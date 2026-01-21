//! AMM math implementations.
//!
//! This module provides mathematical formulas for various AMM protocols:
//! - [`uniswap_v2`]: Constant product formula (x * y = k)
//! - `uniswap_v3`: Concentrated liquidity (coming soon)
//! - `curve`: StableSwap invariant (coming soon)
//! - `balancer`: Weighted pools (coming soon)

pub mod uniswap_v2;

// Re-export common functions at module level
pub use uniswap_v2::{get_amount_in, get_amount_out};
