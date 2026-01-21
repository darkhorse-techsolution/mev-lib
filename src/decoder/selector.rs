//! Method selector utilities.
//!
//! A method selector is the first 4 bytes of the Keccak-256 hash of a function signature.
//! This module provides utilities for working with selectors.

use alloy_primitives::FixedBytes;
use core::fmt::{self, Display};

/// A 4-byte method selector.
///
/// The selector is derived from the first 4 bytes of keccak256(function_signature).
///
/// # Example
///
/// ```rust
/// use mev_lib::decoder::MethodSelector;
///
/// // swapExactTokensForTokens selector
/// let selector = MethodSelector::new([0x38, 0xed, 0x17, 0x39]);
/// assert_eq!(selector.to_string(), "0x38ed1739");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MethodSelector(pub FixedBytes<4>);

impl MethodSelector {
    /// Creates a new selector from raw bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 4]) -> Self {
        Self(FixedBytes::new(bytes))
    }

    /// Creates a selector from a byte slice.
    ///
    /// # Panics
    ///
    /// Panics if the slice is less than 4 bytes.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 4];
        arr.copy_from_slice(&bytes[..4]);
        Self::new(arr)
    }

    /// Returns the raw bytes of the selector.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 4] {
        self.0.as_ref()
    }

    /// Returns the selector as a u32 for fast comparison.
    #[must_use]
    pub fn as_u32(&self) -> u32 {
        u32::from_be_bytes(*self.as_bytes())
    }
}

impl Display for MethodSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.as_bytes()))
    }
}

impl From<[u8; 4]> for MethodSelector {
    fn from(bytes: [u8; 4]) -> Self {
        Self::new(bytes)
    }
}

impl From<u32> for MethodSelector {
    fn from(value: u32) -> Self {
        Self::new(value.to_be_bytes())
    }
}

// ============================================================================
// Well-known selectors
// ============================================================================

/// Uniswap V2 Router selectors
pub mod uniswap_v2 {
    use super::MethodSelector;

    /// swapExactTokensForTokens(uint256,uint256,address[],address,uint256)
    pub const SWAP_EXACT_TOKENS_FOR_TOKENS: MethodSelector =
        MethodSelector::new([0x38, 0xed, 0x17, 0x39]);

    /// swapTokensForExactTokens(uint256,uint256,address[],address,uint256)
    pub const SWAP_TOKENS_FOR_EXACT_TOKENS: MethodSelector =
        MethodSelector::new([0x88, 0x03, 0xdb, 0xee]);

    /// swapExactETHForTokens(uint256,address[],address,uint256)
    pub const SWAP_EXACT_ETH_FOR_TOKENS: MethodSelector =
        MethodSelector::new([0x7f, 0xf3, 0x6a, 0xb5]);

    /// swapTokensForExactETH(uint256,uint256,address[],address,uint256)
    pub const SWAP_TOKENS_FOR_EXACT_ETH: MethodSelector =
        MethodSelector::new([0x4a, 0x25, 0xd9, 0x4a]);

    /// swapExactTokensForETH(uint256,uint256,address[],address,uint256)
    pub const SWAP_EXACT_TOKENS_FOR_ETH: MethodSelector =
        MethodSelector::new([0x18, 0xcb, 0xaf, 0xe5]);

    /// swapETHForExactTokens(uint256,address[],address,uint256)
    pub const SWAP_ETH_FOR_EXACT_TOKENS: MethodSelector =
        MethodSelector::new([0xfb, 0x3b, 0xdb, 0x41]);

    /// swapExactTokensForTokensSupportingFeeOnTransferTokens
    pub const SWAP_EXACT_TOKENS_FOR_TOKENS_SUPPORTING_FEE: MethodSelector =
        MethodSelector::new([0x5c, 0x11, 0xd7, 0x95]);

    /// swapExactETHForTokensSupportingFeeOnTransferTokens
    pub const SWAP_EXACT_ETH_FOR_TOKENS_SUPPORTING_FEE: MethodSelector =
        MethodSelector::new([0xb6, 0xf9, 0xde, 0x95]);

    /// swapExactTokensForETHSupportingFeeOnTransferTokens
    pub const SWAP_EXACT_TOKENS_FOR_ETH_SUPPORTING_FEE: MethodSelector =
        MethodSelector::new([0x79, 0x1a, 0xc9, 0x47]);

    /// Returns true if the selector is a V2 swap method.
    #[must_use]
    pub fn is_swap(selector: &MethodSelector) -> bool {
        matches!(
            *selector,
            SWAP_EXACT_TOKENS_FOR_TOKENS
                | SWAP_TOKENS_FOR_EXACT_TOKENS
                | SWAP_EXACT_ETH_FOR_TOKENS
                | SWAP_TOKENS_FOR_EXACT_ETH
                | SWAP_EXACT_TOKENS_FOR_ETH
                | SWAP_ETH_FOR_EXACT_TOKENS
                | SWAP_EXACT_TOKENS_FOR_TOKENS_SUPPORTING_FEE
                | SWAP_EXACT_ETH_FOR_TOKENS_SUPPORTING_FEE
                | SWAP_EXACT_TOKENS_FOR_ETH_SUPPORTING_FEE
        )
    }
}

/// Uniswap V3 Router selectors
pub mod uniswap_v3 {
    use super::MethodSelector;

    /// exactInputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))
    pub const EXACT_INPUT_SINGLE: MethodSelector = MethodSelector::new([0x41, 0x4b, 0xf3, 0x89]);

    /// exactInput((bytes,address,uint256,uint256,uint256))
    pub const EXACT_INPUT: MethodSelector = MethodSelector::new([0xc0, 0x4b, 0x8d, 0x59]);

    /// exactOutputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))
    pub const EXACT_OUTPUT_SINGLE: MethodSelector = MethodSelector::new([0xdb, 0x3e, 0x21, 0x98]);

    /// exactOutput((bytes,address,uint256,uint256,uint256))
    pub const EXACT_OUTPUT: MethodSelector = MethodSelector::new([0xf2, 0x8c, 0x05, 0x98]);

    /// multicall(uint256,bytes[])
    pub const MULTICALL: MethodSelector = MethodSelector::new([0x5a, 0xe4, 0x01, 0xdc]);

    /// multicall(bytes[])
    pub const MULTICALL_NO_DEADLINE: MethodSelector = MethodSelector::new([0xac, 0x96, 0x50, 0xd8]);

    /// Returns true if the selector is a V3 swap method.
    #[must_use]
    pub fn is_swap(selector: &MethodSelector) -> bool {
        matches!(
            *selector,
            EXACT_INPUT_SINGLE | EXACT_INPUT | EXACT_OUTPUT_SINGLE | EXACT_OUTPUT
        )
    }
}

/// ERC20 selectors
pub mod erc20 {
    use super::MethodSelector;

    /// transfer(address,uint256)
    pub const TRANSFER: MethodSelector = MethodSelector::new([0xa9, 0x05, 0x9c, 0xbb]);

    /// transferFrom(address,address,uint256)
    pub const TRANSFER_FROM: MethodSelector = MethodSelector::new([0x23, 0xb8, 0x72, 0xdd]);

    /// approve(address,uint256)
    pub const APPROVE: MethodSelector = MethodSelector::new([0x09, 0x5e, 0xa7, 0xb3]);

    /// balanceOf(address)
    pub const BALANCE_OF: MethodSelector = MethodSelector::new([0x70, 0xa0, 0x82, 0x31]);

    /// allowance(address,address)
    pub const ALLOWANCE: MethodSelector = MethodSelector::new([0xdd, 0x62, 0xed, 0x3e]);
}

/// Identifies a method selector and returns its human-readable name.
///
/// # Example
///
/// ```rust
/// use mev_lib::decoder::{MethodSelector, identify_selector};
///
/// let selector = MethodSelector::new([0x38, 0xed, 0x17, 0x39]);
/// assert_eq!(identify_selector(&selector), Some("swapExactTokensForTokens"));
/// ```
#[must_use]
pub fn identify_selector(selector: &MethodSelector) -> Option<&'static str> {
    match *selector {
        // Uniswap V2
        uniswap_v2::SWAP_EXACT_TOKENS_FOR_TOKENS => Some("swapExactTokensForTokens"),
        uniswap_v2::SWAP_TOKENS_FOR_EXACT_TOKENS => Some("swapTokensForExactTokens"),
        uniswap_v2::SWAP_EXACT_ETH_FOR_TOKENS => Some("swapExactETHForTokens"),
        uniswap_v2::SWAP_TOKENS_FOR_EXACT_ETH => Some("swapTokensForExactETH"),
        uniswap_v2::SWAP_EXACT_TOKENS_FOR_ETH => Some("swapExactTokensForETH"),
        uniswap_v2::SWAP_ETH_FOR_EXACT_TOKENS => Some("swapETHForExactTokens"),
        uniswap_v2::SWAP_EXACT_TOKENS_FOR_TOKENS_SUPPORTING_FEE => {
            Some("swapExactTokensForTokensSupportingFeeOnTransferTokens")
        }
        uniswap_v2::SWAP_EXACT_ETH_FOR_TOKENS_SUPPORTING_FEE => {
            Some("swapExactETHForTokensSupportingFeeOnTransferTokens")
        }
        uniswap_v2::SWAP_EXACT_TOKENS_FOR_ETH_SUPPORTING_FEE => {
            Some("swapExactTokensForETHSupportingFeeOnTransferTokens")
        }

        // Uniswap V3
        uniswap_v3::EXACT_INPUT_SINGLE => Some("exactInputSingle"),
        uniswap_v3::EXACT_INPUT => Some("exactInput"),
        uniswap_v3::EXACT_OUTPUT_SINGLE => Some("exactOutputSingle"),
        uniswap_v3::EXACT_OUTPUT => Some("exactOutput"),
        uniswap_v3::MULTICALL => Some("multicall"),
        uniswap_v3::MULTICALL_NO_DEADLINE => Some("multicall"),

        // ERC20
        erc20::TRANSFER => Some("transfer"),
        erc20::TRANSFER_FROM => Some("transferFrom"),
        erc20::APPROVE => Some("approve"),
        erc20::BALANCE_OF => Some("balanceOf"),
        erc20::ALLOWANCE => Some("allowance"),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_from_bytes() {
        let selector = MethodSelector::from_bytes(&[0x38, 0xed, 0x17, 0x39]);
        assert_eq!(selector, uniswap_v2::SWAP_EXACT_TOKENS_FOR_TOKENS);
    }

    #[test]
    fn test_selector_display() {
        let selector = uniswap_v2::SWAP_EXACT_TOKENS_FOR_TOKENS;
        assert_eq!(selector.to_string(), "0x38ed1739");
    }

    #[test]
    fn test_identify_v2_swap() {
        let selector = uniswap_v2::SWAP_EXACT_TOKENS_FOR_TOKENS;
        assert_eq!(
            identify_selector(&selector),
            Some("swapExactTokensForTokens")
        );
        assert!(uniswap_v2::is_swap(&selector));
    }

    #[test]
    fn test_identify_v3_swap() {
        let selector = uniswap_v3::EXACT_INPUT_SINGLE;
        assert_eq!(identify_selector(&selector), Some("exactInputSingle"));
        assert!(uniswap_v3::is_swap(&selector));
    }

    #[test]
    fn test_identify_erc20() {
        assert_eq!(identify_selector(&erc20::TRANSFER), Some("transfer"));
        assert_eq!(
            identify_selector(&erc20::TRANSFER_FROM),
            Some("transferFrom")
        );
    }

    #[test]
    fn test_unknown_selector() {
        let selector = MethodSelector::new([0x00, 0x00, 0x00, 0x00]);
        assert_eq!(identify_selector(&selector), None);
    }
}
