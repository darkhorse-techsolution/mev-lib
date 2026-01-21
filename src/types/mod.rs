//! Core type definitions for MEV operations.
//!
//! This module provides type-safe abstractions for common MEV concepts:
//! - [`TokenId`]: Unique identifier for ERC-20 tokens
//! - [`PoolId`]: Unique identifier for liquidity pools
//! - [`Pool`]: Represents a liquidity pool with reserves
//! - [`Swap`]: Represents a swap operation in a specific direction
//! - [`Direction`]: The direction of a swap (ZeroForOne or OneForZero)

mod pool;
mod swap;
mod token;

pub use pool::{Pool, PoolId, Reserves};
pub use swap::{Direction, Swap, SwapId};
pub use token::{Token, TokenId};
