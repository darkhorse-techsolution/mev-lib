//! Transaction simulation utilities using Revm.
//!
//! This module provides local EVM execution capabilities for simulating
//! transactions without broadcasting them to the network. This is essential
//! for MEV detection and profit calculation.
//!
//! # Features
//!
//! - **Local execution**: Simulate transactions using Revm
//! - **State forking**: Fork state from any RPC endpoint
//! - **Call tracing**: Inspect internal calls and state changes
//! - **Gas estimation**: Accurate gas estimation via simulation
//!
//! # Feature Flag
//!
//! This module requires the `simulation` feature:
//!
//! ```toml
//! [dependencies]
//! mev-lib = { version = "0.1", features = ["simulation"] }
//! ```
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      Transaction Simulation                      │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                  │
//! │  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐  │
//! │  │ Pending  │───▶│ Decoder  │───▶│ Simulate │───▶│ Analyze  │  │
//! │  │    Tx    │    │          │    │  (Revm)  │    │  Result  │  │
//! │  └──────────┘    └──────────┘    └──────────┘    └──────────┘  │
//! │                        │               │               │        │
//! │                        ▼               ▼               ▼        │
//! │                   ┌─────────┐    ┌─────────┐    ┌─────────┐    │
//! │                   │ Swap    │    │ State   │    │ Profit  │    │
//! │                   │ Params  │    │ Changes │    │ Calc    │    │
//! │                   └─────────┘    └─────────┘    └─────────┘    │
//! │                                                                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use mev_lib::simulation::{SimulationConfig, simulate_transaction};
//!
//! // Create simulation config
//! let config = SimulationConfig::new()
//!     .with_block_number(18_000_000)
//!     .with_gas_limit(500_000);
//!
//! // Simulate transaction
//! let result = simulate_transaction(&tx, &state, &config)?;
//!
//! match result.status {
//!     ExecutionStatus::Success => {
//!         println!("Gas used: {}", result.gas_used);
//!         println!("Logs: {:?}", result.logs);
//!     }
//!     ExecutionStatus::Revert(reason) => {
//!         println!("Reverted: {}", reason);
//!     }
//! }
//! ```

#[cfg(feature = "simulation")]
mod executor;
#[cfg(feature = "simulation")]
mod result;

#[cfg(feature = "simulation")]
pub use executor::*;
#[cfg(feature = "simulation")]
pub use result::*;

// Re-export types that are always available (no revm dependency)
mod types;
pub use types::*;
