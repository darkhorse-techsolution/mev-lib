//! Simulation example showing Revm-based transaction execution.
//!
//! Run with: cargo run --example simulate --features simulation

#[cfg(feature = "simulation")]
use alloy_primitives::{Address, Bytes, U256};
#[cfg(feature = "simulation")]
use mev_lib::simulation::{simulate, SimulationConfig, SimulationDb, SimulationTx};

#[cfg(feature = "simulation")]
fn main() {
    println!("=== mev-lib Simulation Demo ===\n");

    // 1. Simple ETH transfer
    demo_eth_transfer();

    println!("\n✓ Simulation demo completed!");
}

#[cfg(feature = "simulation")]
fn demo_eth_transfer() {
    println!("--- Simple ETH Transfer ---");

    let mut db = SimulationDb::new();

    let sender = Address::repeat_byte(0x01);
    let recipient = Address::repeat_byte(0x02);

    // Fund the sender with 10 ETH
    let sender_balance = U256::from(10_000_000_000_000_000_000u128);
    db.set_balance(sender, sender_balance);
    println!("Sender {} funded with {} wei", sender, sender_balance);

    // Create a simple ETH transfer of 1 ETH
    let transfer_amount = U256::from(1_000_000_000_000_000_000u128);
    let tx = SimulationTx::new(sender, recipient, Bytes::new())
        .with_value(transfer_amount)
        .with_gas(21_000);

    println!("Simulating transfer of {} wei to {}", transfer_amount, recipient);

    let config = SimulationConfig::new()
        .with_block_number(18_000_000);

    match simulate(&tx, db, &config) {
        Ok(result) => {
            println!("\nSimulation Result:");
            println!("  Status: {:?}", result.status);
            println!("  Gas Used: {}", result.gas_used);
            println!("  Success: {}", result.is_success());

            if !result.logs.is_empty() {
                println!("  Logs: {} events emitted", result.logs.len());
            }
        }
        Err(e) => {
            println!("Simulation failed: {}", e);
        }
    }
}

#[cfg(not(feature = "simulation"))]
fn main() {
    println!("This example requires the 'simulation' feature.");
    println!("Run with: cargo run --example simulate --features simulation");
}
