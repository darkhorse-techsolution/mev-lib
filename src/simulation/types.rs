//! Simulation types that don't require the revm feature.

use alloy_primitives::{Address, Bytes, B256, U256};

/// Configuration for transaction simulation.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Block number to simulate at (for state access)
    pub block_number: Option<u64>,
    /// Block timestamp override
    pub block_timestamp: Option<u64>,
    /// Gas limit for the simulation
    pub gas_limit: u64,
    /// Gas price to use for cost calculation
    pub gas_price: U256,
    /// Base fee override (EIP-1559)
    pub base_fee: Option<U256>,
    /// Coinbase address (block builder)
    pub coinbase: Option<Address>,
    /// Whether to trace internal calls
    pub trace_calls: bool,
    /// Whether to trace storage changes
    pub trace_storage: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            block_number: None,
            block_timestamp: None,
            gas_limit: 30_000_000, // Standard block gas limit
            gas_price: U256::from(20_000_000_000u64), // 20 gwei
            base_fee: None,
            coinbase: None,
            trace_calls: false,
            trace_storage: false,
        }
    }
}

impl SimulationConfig {
    /// Creates a new simulation config with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the block number for state access.
    #[must_use]
    pub const fn with_block_number(mut self, block: u64) -> Self {
        self.block_number = Some(block);
        self
    }

    /// Sets the block timestamp.
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.block_timestamp = Some(timestamp);
        self
    }

    /// Sets the gas limit.
    #[must_use]
    pub const fn with_gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = limit;
        self
    }

    /// Sets the gas price.
    #[must_use]
    pub const fn with_gas_price(mut self, price: U256) -> Self {
        self.gas_price = price;
        self
    }

    /// Sets the base fee.
    #[must_use]
    pub const fn with_base_fee(mut self, fee: U256) -> Self {
        self.base_fee = Some(fee);
        self
    }

    /// Sets the coinbase address.
    #[must_use]
    pub const fn with_coinbase(mut self, coinbase: Address) -> Self {
        self.coinbase = Some(coinbase);
        self
    }

    /// Enables call tracing.
    #[must_use]
    pub const fn with_call_tracing(mut self) -> Self {
        self.trace_calls = true;
        self
    }

    /// Enables storage tracing.
    #[must_use]
    pub const fn with_storage_tracing(mut self) -> Self {
        self.trace_storage = true;
        self
    }
}

/// A pending transaction to simulate.
#[derive(Debug, Clone)]
pub struct SimulationTx {
    /// Sender address
    pub from: Address,
    /// Recipient address (None for contract creation)
    pub to: Option<Address>,
    /// Transaction value in wei
    pub value: U256,
    /// Transaction input data
    pub input: Bytes,
    /// Gas limit
    pub gas: u64,
    /// Gas price (legacy) or max fee per gas (EIP-1559)
    pub gas_price: U256,
    /// Max priority fee per gas (EIP-1559)
    pub max_priority_fee: Option<U256>,
    /// Transaction nonce
    pub nonce: Option<u64>,
}

impl SimulationTx {
    /// Creates a new simulation transaction.
    #[must_use]
    pub fn new(from: Address, to: Address, input: Bytes) -> Self {
        Self {
            from,
            to: Some(to),
            value: U256::ZERO,
            input,
            gas: 500_000,
            gas_price: U256::from(20_000_000_000u64),
            max_priority_fee: None,
            nonce: None,
        }
    }

    /// Creates a contract creation transaction.
    #[must_use]
    pub fn create(from: Address, bytecode: Bytes) -> Self {
        Self {
            from,
            to: None,
            value: U256::ZERO,
            input: bytecode,
            gas: 3_000_000,
            gas_price: U256::from(20_000_000_000u64),
            max_priority_fee: None,
            nonce: None,
        }
    }

    /// Sets the transaction value.
    #[must_use]
    pub const fn with_value(mut self, value: U256) -> Self {
        self.value = value;
        self
    }

    /// Sets the gas limit.
    #[must_use]
    pub const fn with_gas(mut self, gas: u64) -> Self {
        self.gas = gas;
        self
    }

    /// Sets the gas price.
    #[must_use]
    pub const fn with_gas_price(mut self, price: U256) -> Self {
        self.gas_price = price;
        self
    }

    /// Sets the nonce.
    #[must_use]
    pub const fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }
}

/// Result of a transaction simulation.
#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// Execution status
    pub status: ExecutionStatus,
    /// Gas used by the transaction
    pub gas_used: u64,
    /// Gas refunded
    pub gas_refunded: u64,
    /// Output data (return value or revert reason)
    pub output: Bytes,
    /// Emitted logs
    pub logs: Vec<SimulationLog>,
    /// Internal calls (if tracing enabled)
    pub calls: Vec<InternalCall>,
    /// Created contract address (if contract creation)
    pub created_address: Option<Address>,
}

impl SimulationResult {
    /// Returns true if the transaction succeeded.
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self.status, ExecutionStatus::Success)
    }

    /// Returns true if the transaction reverted.
    #[must_use]
    pub fn is_revert(&self) -> bool {
        matches!(self.status, ExecutionStatus::Revert)
    }

    /// Calculates the gas cost in wei.
    #[must_use]
    pub fn gas_cost(&self, gas_price: U256) -> U256 {
        gas_price * U256::from(self.gas_used)
    }

    /// Attempts to decode the revert reason as a string.
    #[must_use]
    pub fn revert_reason(&self) -> Option<String> {
        if !self.is_revert() || self.output.len() < 68 {
            return None;
        }

        // Check for Error(string) selector: 0x08c379a0
        if self.output[0..4] != [0x08, 0xc3, 0x79, 0xa0] {
            return None;
        }

        // Decode string from ABI encoding
        // Skip selector (4) + offset (32) + length offset
        let length_start = 36;
        if self.output.len() < length_start + 32 {
            return None;
        }

        let length = U256::from_be_slice(&self.output[length_start..length_start + 32]);
        let string_start = length_start + 32;
        let string_end = string_start + length.to::<usize>();

        if self.output.len() < string_end {
            return None;
        }

        String::from_utf8(self.output[string_start..string_end].to_vec()).ok()
    }
}

/// Transaction execution status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Transaction executed successfully
    Success,
    /// Transaction reverted
    Revert,
    /// Transaction ran out of gas
    OutOfGas,
    /// Invalid opcode encountered
    InvalidOpcode,
    /// Other halt reason
    Halt(String),
}

/// An emitted log during simulation.
#[derive(Debug, Clone)]
pub struct SimulationLog {
    /// Contract that emitted the log
    pub address: Address,
    /// Log topics (indexed parameters)
    pub topics: Vec<B256>,
    /// Log data (non-indexed parameters)
    pub data: Bytes,
}

impl SimulationLog {
    /// Returns the event signature (first topic).
    #[must_use]
    pub fn event_signature(&self) -> Option<B256> {
        self.topics.first().copied()
    }

    /// Returns true if this log matches the given event signature.
    #[must_use]
    pub fn matches_signature(&self, signature: B256) -> bool {
        self.event_signature() == Some(signature)
    }
}

/// An internal call during transaction execution.
#[derive(Debug, Clone)]
pub struct InternalCall {
    /// Call type
    pub call_type: CallType,
    /// Caller address
    pub from: Address,
    /// Callee address
    pub to: Address,
    /// Value transferred
    pub value: U256,
    /// Input data
    pub input: Bytes,
    /// Gas provided
    pub gas: u64,
    /// Gas used
    pub gas_used: u64,
    /// Output data
    pub output: Bytes,
    /// Whether the call succeeded
    pub success: bool,
    /// Nested calls
    pub calls: Vec<InternalCall>,
}

/// Type of internal call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallType {
    /// Regular call (CALL opcode)
    Call,
    /// Static call (STATICCALL opcode)
    StaticCall,
    /// Delegate call (DELEGATECALL opcode)
    DelegateCall,
    /// Create (CREATE opcode)
    Create,
    /// Create2 (CREATE2 opcode)
    Create2,
}

/// State override for simulation.
///
/// Allows modifying account state before simulation.
#[derive(Debug, Clone, Default)]
pub struct StateOverride {
    /// Override account balance
    pub balance: Option<U256>,
    /// Override account nonce
    pub nonce: Option<u64>,
    /// Override account code
    pub code: Option<Bytes>,
    /// Override specific storage slots
    pub storage: Vec<(B256, B256)>,
}

impl StateOverride {
    /// Creates a new empty state override.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the balance override.
    #[must_use]
    pub const fn with_balance(mut self, balance: U256) -> Self {
        self.balance = Some(balance);
        self
    }

    /// Sets the nonce override.
    #[must_use]
    pub const fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Sets the code override.
    #[must_use]
    pub fn with_code(mut self, code: Bytes) -> Self {
        self.code = Some(code);
        self
    }

    /// Adds a storage slot override.
    #[must_use]
    pub fn with_storage(mut self, slot: B256, value: B256) -> Self {
        self.storage.push((slot, value));
        self
    }
}

/// Well-known event signatures for MEV detection.
pub mod event_signatures {
    use alloy_primitives::b256;

    /// Uniswap V2 Swap event
    /// `Swap(address indexed sender, uint256 amount0In, uint256 amount1In, uint256 amount0Out, uint256 amount1Out, address indexed to)`
    pub const UNISWAP_V2_SWAP: alloy_primitives::B256 =
        b256!("d78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822");

    /// Uniswap V3 Swap event
    /// `Swap(address indexed sender, address indexed recipient, int256 amount0, int256 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick)`
    pub const UNISWAP_V3_SWAP: alloy_primitives::B256 =
        b256!("c42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67");

    /// ERC20 Transfer event
    /// `Transfer(address indexed from, address indexed to, uint256 value)`
    pub const ERC20_TRANSFER: alloy_primitives::B256 =
        b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

    /// ERC20 Approval event
    /// `Approval(address indexed owner, address indexed spender, uint256 value)`
    pub const ERC20_APPROVAL: alloy_primitives::B256 =
        b256!("8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_config_builder() {
        let config = SimulationConfig::new()
            .with_block_number(18_000_000)
            .with_gas_limit(500_000)
            .with_call_tracing();

        assert_eq!(config.block_number, Some(18_000_000));
        assert_eq!(config.gas_limit, 500_000);
        assert!(config.trace_calls);
    }

    #[test]
    fn test_simulation_tx_builder() {
        let from = Address::repeat_byte(0x01);
        let to = Address::repeat_byte(0x02);
        let input = Bytes::from(vec![0x38, 0xed, 0x17, 0x39]);

        let tx = SimulationTx::new(from, to, input.clone())
            .with_value(U256::from(1_000_000_000_000_000_000u128))
            .with_gas(100_000);

        assert_eq!(tx.from, from);
        assert_eq!(tx.to, Some(to));
        assert_eq!(tx.value, U256::from(1_000_000_000_000_000_000u128));
        assert_eq!(tx.gas, 100_000);
    }

    #[test]
    fn test_state_override_builder() {
        let override_ = StateOverride::new()
            .with_balance(U256::from(1_000_000u64))
            .with_nonce(5);

        assert_eq!(override_.balance, Some(U256::from(1_000_000u64)));
        assert_eq!(override_.nonce, Some(5));
    }
}
