//! Calldata decoding utilities for DEX transactions.
//!
//! This module provides functions to decode and parse calldata from various
//! DEX protocols, making it easy to understand what a transaction is doing.
//!
//! # Supported Protocols
//!
//! - **Uniswap V2**: Router swaps (swapExactTokensForTokens, etc.)
//! - **Uniswap V3**: SwapRouter and SwapRouter02
//! - **ERC20**: transfer, transferFrom, approve
//!
//! # Example
//!
//! ```rust,ignore
//! use mev_lib::decoder::{decode_calldata, DecodedCall};
//!
//! let calldata = hex::decode("38ed1739...").unwrap();
//! match decode_calldata(&calldata) {
//!     Some(DecodedCall::UniswapV2(swap)) => {
//!         println!("V2 Swap: {} -> {}", swap.path[0], swap.path[1]);
//!     }
//!     Some(DecodedCall::UniswapV3(swap)) => {
//!         println!("V3 Swap: {} tokens", swap.amount_in);
//!     }
//!     _ => println!("Unknown call"),
//! }
//! ```

mod erc20;
mod selector;
mod uniswap_v2;
mod uniswap_v3;

pub use erc20::*;
pub use selector::{identify_selector, MethodSelector};
pub use uniswap_v2::*;
pub use uniswap_v3::*;

/// Well-known method selectors organized by protocol.
pub mod selectors {
    pub use super::selector::erc20 as erc20_selectors;
    pub use super::selector::uniswap_v2 as v2_selectors;
    pub use super::selector::uniswap_v3 as v3_selectors;
}

use alloy_primitives::{Address, Bytes, U256};

/// A decoded DEX call with parsed parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedCall {
    /// Uniswap V2 style swap
    UniswapV2(UniswapV2Swap),
    /// Uniswap V3 style swap
    UniswapV3(UniswapV3Swap),
    /// ERC20 transfer
    Transfer(Erc20Transfer),
    /// ERC20 transferFrom
    TransferFrom(Erc20TransferFrom),
    /// ERC20 approve
    Approve(Erc20Approve),
    /// Unknown call with raw selector
    Unknown(MethodSelector),
}

/// Attempts to decode calldata into a known call type.
///
/// # Arguments
///
/// * `calldata` - Raw transaction input data
///
/// # Returns
///
/// `Some(DecodedCall)` if the calldata matches a known pattern, `None` if too short.
///
/// # Example
///
/// ```rust
/// use mev_lib::decoder::decode_calldata;
/// use alloy_primitives::Bytes;
///
/// let calldata = Bytes::from(vec![0x38, 0xed, 0x17, 0x39]); // swapExactTokensForTokens selector
/// let decoded = decode_calldata(&calldata);
/// ```
#[must_use]
pub fn decode_calldata(calldata: &Bytes) -> Option<DecodedCall> {
    if calldata.len() < 4 {
        return None;
    }

    let selector = MethodSelector::from_bytes(&calldata[..4]);

    // Try Uniswap V2
    if let Some(swap) = decode_uniswap_v2(calldata) {
        return Some(DecodedCall::UniswapV2(swap));
    }

    // Try Uniswap V3
    if let Some(swap) = decode_uniswap_v3(calldata) {
        return Some(DecodedCall::UniswapV3(swap));
    }

    // Try ERC20
    if let Some(transfer) = decode_erc20_transfer(calldata) {
        return Some(DecodedCall::Transfer(transfer));
    }

    if let Some(transfer_from) = decode_erc20_transfer_from(calldata) {
        return Some(DecodedCall::TransferFrom(transfer_from));
    }

    if let Some(approve) = decode_erc20_approve(calldata) {
        return Some(DecodedCall::Approve(approve));
    }

    // Unknown but valid selector
    Some(DecodedCall::Unknown(selector))
}

/// Extracts the swap path from any supported DEX call.
///
/// This is useful when you just need to know the token path without
/// caring about the specific protocol.
#[must_use]
pub fn extract_swap_path(call: &DecodedCall) -> Option<Vec<Address>> {
    match call {
        DecodedCall::UniswapV2(swap) => Some(swap.path.clone()),
        DecodedCall::UniswapV3(swap) => Some(swap.path()),
        _ => None,
    }
}

/// Checks if a decoded call is a swap operation.
#[must_use]
pub fn is_swap(call: &DecodedCall) -> bool {
    matches!(call, DecodedCall::UniswapV2(_) | DecodedCall::UniswapV3(_))
}

/// Extracts the input amount from a swap call.
#[must_use]
pub fn get_swap_amount_in(call: &DecodedCall) -> Option<U256> {
    match call {
        DecodedCall::UniswapV2(swap) => swap.amount_in,
        DecodedCall::UniswapV3(swap) => swap.amount_in,
        _ => None,
    }
}

/// Extracts the minimum output amount from a swap call.
#[must_use]
pub fn get_swap_amount_out_min(call: &DecodedCall) -> Option<U256> {
    match call {
        DecodedCall::UniswapV2(swap) => swap.amount_out_min,
        DecodedCall::UniswapV3(swap) => swap.amount_out_min,
        _ => None,
    }
}
