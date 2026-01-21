//! Revm-based transaction executor.
//!
//! This module provides the actual simulation execution using Revm.
//! It requires the `simulation` feature flag.

use alloy_primitives::{Address, Bytes, U256};
use revm::{
    primitives::{
        AccountInfo, Bytecode, ExecutionResult, HaltReason, Output,
        TransactTo, TxEnv, CfgEnv, BlockEnv, Env,
    },
    Evm, InMemoryDB,
};
use std::collections::HashMap;

use super::types::{
    ExecutionStatus, SimulationConfig, SimulationLog,
    SimulationResult, SimulationTx, StateOverride,
};

/// A simple in-memory database for simulation.
///
/// This can be extended to support forking from an RPC endpoint.
pub struct SimulationDb {
    inner: InMemoryDB,
}

impl SimulationDb {
    /// Creates a new empty simulation database.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: InMemoryDB::default(),
        }
    }

    /// Sets an account's balance.
    pub fn set_balance(&mut self, address: Address, balance: U256) {
        // Get existing info or create default
        let mut info = match self.inner.accounts.get(&address) {
            Some(account) => account.info.clone(),
            None => AccountInfo::default(),
        };
        info.balance = balance;
        self.inner.insert_account_info(address, info);
    }

    /// Sets an account's code.
    pub fn set_code(&mut self, address: Address, code: Bytes) {
        let info = self.inner.load_account(address)
            .ok()
            .map(|a| a.info.clone())
            .unwrap_or_default();
        self.inner.insert_account_info(
            address,
            AccountInfo {
                code_hash: revm::primitives::keccak256(&code),
                code: Some(Bytecode::new_raw(code)),
                ..info
            },
        );
    }

    /// Sets a storage slot.
    pub fn set_storage(&mut self, address: Address, slot: U256, value: U256) {
        let _ = self.inner.insert_account_storage(address, slot, value);
    }

    /// Applies state overrides to the database.
    pub fn apply_overrides(&mut self, overrides: &HashMap<Address, StateOverride>) {
        for (address, override_) in overrides {
            if let Some(balance) = override_.balance {
                self.set_balance(*address, balance);
            }

            if let Some(ref code) = override_.code {
                self.set_code(*address, code.clone());
            }

            for (slot, value) in &override_.storage {
                let slot_u256 = U256::from_be_bytes(slot.0);
                let value_u256 = U256::from_be_bytes(value.0);
                self.set_storage(*address, slot_u256, value_u256);
            }
        }
    }

    /// Returns the inner database for use with Revm.
    pub fn into_inner(self) -> InMemoryDB {
        self.inner
    }

    /// Returns a reference to the inner database.
    pub fn inner(&self) -> &InMemoryDB {
        &self.inner
    }

    /// Returns a mutable reference to the inner database.
    pub fn inner_mut(&mut self) -> &mut InMemoryDB {
        &mut self.inner
    }
}

impl Default for SimulationDb {
    fn default() -> Self {
        Self::new()
    }
}

/// Simulates a transaction and returns the result.
///
/// # Arguments
///
/// * `tx` - The transaction to simulate
/// * `db` - The state database
/// * `config` - Simulation configuration
///
/// # Returns
///
/// The simulation result including gas used, logs, and execution status.
///
/// # Example
///
/// ```rust,ignore
/// use mev_lib::simulation::{simulate, SimulationDb, SimulationConfig, SimulationTx};
/// use alloy_primitives::{Address, Bytes, U256};
///
/// let mut db = SimulationDb::new();
/// db.set_balance(sender, U256::from(10_000_000_000_000_000_000u128));
///
/// let tx = SimulationTx::new(sender, recipient, calldata);
/// let config = SimulationConfig::new().with_gas_limit(500_000);
///
/// let result = simulate(&tx, db, &config)?;
/// println!("Gas used: {}", result.gas_used);
/// ```
pub fn simulate(
    tx: &SimulationTx,
    db: SimulationDb,
    config: &SimulationConfig,
) -> Result<SimulationResult, SimulationError> {
    // Build environment
    let mut cfg = CfgEnv::default();
    cfg.chain_id = 1; // Mainnet

    let mut block = BlockEnv::default();
    if let Some(number) = config.block_number {
        block.number = U256::from(number);
    }
    if let Some(timestamp) = config.block_timestamp {
        block.timestamp = U256::from(timestamp);
    }
    if let Some(base_fee) = config.base_fee {
        block.basefee = base_fee;
    }
    if let Some(coinbase) = config.coinbase {
        block.coinbase = coinbase;
    }

    let mut tx_env = TxEnv::default();
    tx_env.caller = tx.from;
    tx_env.gas_limit = tx.gas;
    tx_env.gas_price = tx.gas_price;
    tx_env.value = tx.value;
    tx_env.data = tx.input.clone();
    tx_env.transact_to = match tx.to {
        Some(addr) => TransactTo::Call(addr),
        None => TransactTo::Create,
    };
    if let Some(nonce) = tx.nonce {
        tx_env.nonce = Some(nonce);
    }

    // Create EVM instance
    let mut evm = Evm::builder()
        .with_db(db.into_inner())
        .with_env(Box::new(Env {
            cfg,
            block,
            tx: tx_env,
        }))
        .build();

    // Execute transaction
    let result = evm.transact().map_err(|e| SimulationError::Execution(format!("{:?}", e)))?;

    // Convert result
    convert_execution_result(result.result)
}

/// Simulates a transaction with state overrides.
///
/// This is useful for testing scenarios like:
/// - Simulating with infinite balance
/// - Testing against modified contract state
/// - Frontrunning/backrunning simulation
pub fn simulate_with_overrides(
    tx: &SimulationTx,
    mut db: SimulationDb,
    config: &SimulationConfig,
    overrides: &HashMap<Address, StateOverride>,
) -> Result<SimulationResult, SimulationError> {
    db.apply_overrides(overrides);
    simulate(tx, db, config)
}

/// Simulates multiple transactions in sequence.
///
/// This is useful for simulating MEV bundles where transactions
/// must execute in a specific order.
///
/// # Arguments
///
/// * `txs` - Transactions to simulate in order
/// * `db` - The state database (will be modified)
/// * `config` - Simulation configuration
///
/// # Returns
///
/// A vector of simulation results, one per transaction.
pub fn simulate_bundle(
    txs: &[SimulationTx],
    db: SimulationDb,
    config: &SimulationConfig,
) -> Result<Vec<SimulationResult>, SimulationError> {
    let mut results = Vec::with_capacity(txs.len());

    // Build environment
    let mut cfg = CfgEnv::default();
    cfg.chain_id = 1;

    let mut block = BlockEnv::default();
    if let Some(number) = config.block_number {
        block.number = U256::from(number);
    }
    if let Some(timestamp) = config.block_timestamp {
        block.timestamp = U256::from(timestamp);
    }
    if let Some(base_fee) = config.base_fee {
        block.basefee = base_fee;
    }

    let mut inner_db = db.into_inner();

    for tx in txs {
        let mut tx_env = TxEnv::default();
        tx_env.caller = tx.from;
        tx_env.gas_limit = tx.gas;
        tx_env.gas_price = tx.gas_price;
        tx_env.value = tx.value;
        tx_env.data = tx.input.clone();
        tx_env.transact_to = match tx.to {
            Some(addr) => TransactTo::Call(addr),
            None => TransactTo::Create,
        };

        let mut evm = Evm::builder()
            .with_db(&mut inner_db)
            .with_env(Box::new(Env {
                cfg: cfg.clone(),
                block: block.clone(),
                tx: tx_env,
            }))
            .build();

        let result = evm.transact_commit()
            .map_err(|e| SimulationError::Execution(format!("{:?}", e)))?;

        results.push(convert_execution_result(result)?);
    }

    Ok(results)
}

/// Converts a Revm execution result to our SimulationResult.
fn convert_execution_result(result: ExecutionResult) -> Result<SimulationResult, SimulationError> {
    match result {
        ExecutionResult::Success {
            reason: _,
            gas_used,
            gas_refunded,
            logs,
            output,
        } => {
            let (output_data, created_address) = match output {
                Output::Call(data) => (data, None),
                Output::Create(data, addr) => (data, addr),
            };

            Ok(SimulationResult {
                status: ExecutionStatus::Success,
                gas_used,
                gas_refunded,
                output: output_data,
                logs: logs
                    .into_iter()
                    .map(|log| SimulationLog {
                        address: log.address,
                        topics: log.topics().to_vec(),
                        data: log.data.data,
                    })
                    .collect(),
                calls: vec![], // Would need inspector for this
                created_address,
            })
        }
        ExecutionResult::Revert { gas_used, output } => Ok(SimulationResult {
            status: ExecutionStatus::Revert,
            gas_used,
            gas_refunded: 0,
            output,
            logs: vec![],
            calls: vec![],
            created_address: None,
        }),
        ExecutionResult::Halt { reason, gas_used } => {
            let status = match reason {
                HaltReason::OutOfGas(_) => ExecutionStatus::OutOfGas,
                HaltReason::OpcodeNotFound | HaltReason::InvalidFEOpcode => {
                    ExecutionStatus::InvalidOpcode
                }
                other => ExecutionStatus::Halt(format!("{:?}", other)),
            };

            Ok(SimulationResult {
                status,
                gas_used,
                gas_refunded: 0,
                output: Bytes::new(),
                logs: vec![],
                calls: vec![],
                created_address: None,
            })
        }
    }
}

/// Errors that can occur during simulation.
#[derive(Debug, thiserror::Error)]
pub enum SimulationError {
    /// EVM execution error
    #[error("execution error: {0}")]
    Execution(String),
    /// Database error
    #[error("database error: {0}")]
    Database(String),
    /// Invalid transaction
    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_db_balance() {
        let mut db = SimulationDb::new();
        let addr = Address::repeat_byte(0x01);

        db.set_balance(addr, U256::from(1_000_000u64));

        // Verify balance was set
        let account = db.inner_mut().load_account(addr).unwrap();
        assert_eq!(account.info.balance, U256::from(1_000_000u64));
    }

    #[test]
    fn test_simple_transfer_simulation() {
        let mut db = SimulationDb::new();

        let sender = Address::repeat_byte(0x01);
        let recipient = Address::repeat_byte(0x02);

        // Fund the sender
        db.set_balance(sender, U256::from(10_000_000_000_000_000_000u128)); // 10 ETH

        // Create a simple ETH transfer
        let tx = SimulationTx::new(sender, recipient, Bytes::new())
            .with_value(U256::from(1_000_000_000_000_000_000u128)) // 1 ETH
            .with_gas(21_000);

        let config = SimulationConfig::new();

        let result = simulate(&tx, db, &config).expect("simulation should succeed");

        assert!(result.is_success());
        assert_eq!(result.gas_used, 21_000);
    }
}
