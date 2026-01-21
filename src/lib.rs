//! # mev-lib
//!
//! A comprehensive MEV (Maximal Extractable Value) utilities library for Ethereum.
//!
//! This library provides core primitives and utilities for building MEV bots,
//! including type-safe abstractions for tokens, pools, and swaps, as well as
//! AMM math implementations for various DEX protocols.
//!
//! ## Modules
//!
//! - **[`types`]**: Type-safe primitives (`TokenId`, `PoolId`, `SwapId`)
//! - **[`math`]**: AMM math formulas (Uniswap V2, V3)
//! - **[`decoder`]**: Calldata decoding (Uniswap V2/V3, ERC20)
//! - **[`simulation`]**: Local transaction simulation via Revm
//!
//! ## Example
//!
//! ```rust
//! use mev_lib::prelude::*;
//! use mev_lib::math::uniswap_v2;
//! use alloy_primitives::U256;
//!
//! // Calculate swap output using Uniswap V2 formula
//! let amount_in = U256::from(1_000_000u64);  // 1M tokens
//! let reserve_in = U256::from(100_000_000_000u64);  // 100B reserve
//! let reserve_out = U256::from(100_000_000_000u64); // 100B reserve
//!
//! let amount_out = uniswap_v2::get_amount_out(amount_in, reserve_in, reserve_out, 30);
//! ```
//!
//! ## Decoding Swap Calldata
//!
//! ```rust,ignore
//! use mev_lib::decoder::{decode_calldata, DecodedCall};
//!
//! let calldata = tx.input();
//! match decode_calldata(&calldata) {
//!     Some(DecodedCall::UniswapV2(swap)) => {
//!         println!("V2 swap: {:?} -> {:?}", swap.token_in(), swap.token_out());
//!     }
//!     Some(DecodedCall::UniswapV3(swap)) => {
//!         println!("V3 swap with {} bps fee", swap.primary_fee().unwrap_or(0));
//!     }
//!     _ => {}
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

pub mod decoder;
pub mod math;
pub mod simulation;
pub mod types;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::types::{
        Direction, Pool, PoolId, Reserves, Swap, SwapId, Token, TokenId,
    };

    pub use crate::decoder::{
        decode_calldata, DecodedCall, MethodSelector, UniswapV2Method, UniswapV2Swap,
        UniswapV3Method, UniswapV3Swap,
    };
}
