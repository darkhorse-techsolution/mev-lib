//! Uniswap V2 Router calldata decoder.
//!
//! Decodes swap calls from Uniswap V2 and compatible routers (SushiSwap, PancakeSwap, etc.).
//!
//! # Supported Methods
//!
//! - `swapExactTokensForTokens`
//! - `swapTokensForExactTokens`
//! - `swapExactETHForTokens`
//! - `swapTokensForExactETH`
//! - `swapExactTokensForETH`
//! - `swapETHForExactTokens`
//! - Fee-on-transfer variants
//!
//! # Example
//!
//! ```rust,ignore
//! use mev_lib::decoder::decode_uniswap_v2;
//! use alloy_primitives::Bytes;
//!
//! let calldata: Bytes = /* ... */;
//! if let Some(swap) = decode_uniswap_v2(&calldata) {
//!     println!("Swap path: {:?}", swap.path);
//!     println!("Amount in: {:?}", swap.amount_in);
//! }
//! ```

use alloy_primitives::{Address, Bytes, U256};

use super::selector::{uniswap_v2 as selectors, MethodSelector};

/// Decoded Uniswap V2 swap parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniswapV2Swap {
    /// The method that was called
    pub method: UniswapV2Method,
    /// Amount of input tokens (for exactInput methods)
    pub amount_in: Option<U256>,
    /// Minimum output tokens expected (for exactInput methods)
    pub amount_out_min: Option<U256>,
    /// Exact output tokens desired (for exactOutput methods)
    pub amount_out: Option<U256>,
    /// Maximum input tokens willing to spend (for exactOutput methods)
    pub amount_in_max: Option<U256>,
    /// Token swap path
    pub path: Vec<Address>,
    /// Recipient address
    pub to: Address,
    /// Transaction deadline (Unix timestamp)
    pub deadline: U256,
}

/// Uniswap V2 swap method types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniswapV2Method {
    /// swapExactTokensForTokens - exact input, minimum output
    SwapExactTokensForTokens,
    /// swapTokensForExactTokens - maximum input, exact output
    SwapTokensForExactTokens,
    /// swapExactETHForTokens - exact ETH input, minimum token output
    SwapExactETHForTokens,
    /// swapTokensForExactETH - maximum token input, exact ETH output
    SwapTokensForExactETH,
    /// swapExactTokensForETH - exact token input, minimum ETH output
    SwapExactTokensForETH,
    /// swapETHForExactTokens - maximum ETH input, exact token output
    SwapETHForExactTokens,
    /// swapExactTokensForTokensSupportingFeeOnTransferTokens
    SwapExactTokensForTokensFee,
    /// swapExactETHForTokensSupportingFeeOnTransferTokens
    SwapExactETHForTokensFee,
    /// swapExactTokensForETHSupportingFeeOnTransferTokens
    SwapExactTokensForETHFee,
}

impl UniswapV2Swap {
    /// Returns the input token (first in path).
    #[must_use]
    pub fn token_in(&self) -> Option<Address> {
        self.path.first().copied()
    }

    /// Returns the output token (last in path).
    #[must_use]
    pub fn token_out(&self) -> Option<Address> {
        self.path.last().copied()
    }

    /// Returns true if this is an exact input swap.
    #[must_use]
    pub fn is_exact_input(&self) -> bool {
        matches!(
            self.method,
            UniswapV2Method::SwapExactTokensForTokens
                | UniswapV2Method::SwapExactETHForTokens
                | UniswapV2Method::SwapExactTokensForETH
                | UniswapV2Method::SwapExactTokensForTokensFee
                | UniswapV2Method::SwapExactETHForTokensFee
                | UniswapV2Method::SwapExactTokensForETHFee
        )
    }

    /// Returns true if this swap involves native ETH.
    #[must_use]
    pub fn involves_eth(&self) -> bool {
        matches!(
            self.method,
            UniswapV2Method::SwapExactETHForTokens
                | UniswapV2Method::SwapTokensForExactETH
                | UniswapV2Method::SwapExactTokensForETH
                | UniswapV2Method::SwapETHForExactTokens
                | UniswapV2Method::SwapExactETHForTokensFee
                | UniswapV2Method::SwapExactTokensForETHFee
        )
    }

    /// Returns true if this is a fee-on-transfer compatible method.
    #[must_use]
    pub fn supports_fee_on_transfer(&self) -> bool {
        matches!(
            self.method,
            UniswapV2Method::SwapExactTokensForTokensFee
                | UniswapV2Method::SwapExactETHForTokensFee
                | UniswapV2Method::SwapExactTokensForETHFee
        )
    }

    /// Returns the number of hops in the swap.
    #[must_use]
    pub fn hop_count(&self) -> usize {
        if self.path.is_empty() {
            0
        } else {
            self.path.len() - 1
        }
    }
}

/// Decodes Uniswap V2 router calldata.
///
/// # Arguments
///
/// * `calldata` - Raw transaction input data
///
/// # Returns
///
/// `Some(UniswapV2Swap)` if the calldata is a valid V2 swap, `None` otherwise.
#[must_use]
pub fn decode_uniswap_v2(calldata: &Bytes) -> Option<UniswapV2Swap> {
    if calldata.len() < 4 {
        return None;
    }

    let selector = MethodSelector::from_bytes(&calldata[..4]);

    match selector {
        selectors::SWAP_EXACT_TOKENS_FOR_TOKENS => {
            decode_swap_exact_tokens_for_tokens(calldata, UniswapV2Method::SwapExactTokensForTokens)
        }
        selectors::SWAP_TOKENS_FOR_EXACT_TOKENS => {
            decode_swap_tokens_for_exact_tokens(calldata, UniswapV2Method::SwapTokensForExactTokens)
        }
        selectors::SWAP_EXACT_ETH_FOR_TOKENS => {
            decode_swap_exact_eth_for_tokens(calldata, UniswapV2Method::SwapExactETHForTokens)
        }
        selectors::SWAP_TOKENS_FOR_EXACT_ETH => {
            decode_swap_tokens_for_exact_eth(calldata, UniswapV2Method::SwapTokensForExactETH)
        }
        selectors::SWAP_EXACT_TOKENS_FOR_ETH => {
            decode_swap_exact_tokens_for_eth(calldata, UniswapV2Method::SwapExactTokensForETH)
        }
        selectors::SWAP_ETH_FOR_EXACT_TOKENS => {
            decode_swap_eth_for_exact_tokens(calldata, UniswapV2Method::SwapETHForExactTokens)
        }
        selectors::SWAP_EXACT_TOKENS_FOR_TOKENS_SUPPORTING_FEE => {
            decode_swap_exact_tokens_for_tokens(calldata, UniswapV2Method::SwapExactTokensForTokensFee)
        }
        selectors::SWAP_EXACT_ETH_FOR_TOKENS_SUPPORTING_FEE => {
            decode_swap_exact_eth_for_tokens(calldata, UniswapV2Method::SwapExactETHForTokensFee)
        }
        selectors::SWAP_EXACT_TOKENS_FOR_ETH_SUPPORTING_FEE => {
            decode_swap_exact_tokens_for_eth(calldata, UniswapV2Method::SwapExactTokensForETHFee)
        }
        _ => None,
    }
}

/// Reads a U256 from calldata at the given offset.
fn read_u256(data: &[u8], offset: usize) -> Option<U256> {
    if offset + 32 > data.len() {
        return None;
    }
    Some(U256::from_be_slice(&data[offset..offset + 32]))
}

/// Reads an Address from calldata at the given offset.
fn read_address(data: &[u8], offset: usize) -> Option<Address> {
    if offset + 32 > data.len() {
        return None;
    }
    // Address is in the last 20 bytes of the 32-byte slot
    Some(Address::from_slice(&data[offset + 12..offset + 32]))
}

/// Reads a dynamic address array from calldata.
fn read_address_array(data: &[u8], array_offset: usize) -> Option<Vec<Address>> {
    // First read the length
    let length = read_u256(data, array_offset)?.to::<usize>();

    if length > 10 {
        // Sanity check: paths shouldn't be too long
        return None;
    }

    let mut addresses = Vec::with_capacity(length);
    for i in 0..length {
        let addr = read_address(data, array_offset + 32 + i * 32)?;
        addresses.push(addr);
    }

    Some(addresses)
}

/// Decodes swapExactTokensForTokens(uint256 amountIn, uint256 amountOutMin, address[] path, address to, uint256 deadline)
fn decode_swap_exact_tokens_for_tokens(
    calldata: &Bytes,
    method: UniswapV2Method,
) -> Option<UniswapV2Swap> {
    if calldata.len() < 4 + 32 * 5 {
        return None;
    }

    let data = &calldata[4..]; // Skip selector

    let amount_in = read_u256(data, 0)?;
    let amount_out_min = read_u256(data, 32)?;
    let path_offset = read_u256(data, 64)?.to::<usize>();
    let to = read_address(data, 96)?;
    let deadline = read_u256(data, 128)?;

    let path = read_address_array(data, path_offset)?;

    Some(UniswapV2Swap {
        method,
        amount_in: Some(amount_in),
        amount_out_min: Some(amount_out_min),
        amount_out: None,
        amount_in_max: None,
        path,
        to,
        deadline,
    })
}

/// Decodes swapTokensForExactTokens(uint256 amountOut, uint256 amountInMax, address[] path, address to, uint256 deadline)
fn decode_swap_tokens_for_exact_tokens(
    calldata: &Bytes,
    method: UniswapV2Method,
) -> Option<UniswapV2Swap> {
    if calldata.len() < 4 + 32 * 5 {
        return None;
    }

    let data = &calldata[4..];

    let amount_out = read_u256(data, 0)?;
    let amount_in_max = read_u256(data, 32)?;
    let path_offset = read_u256(data, 64)?.to::<usize>();
    let to = read_address(data, 96)?;
    let deadline = read_u256(data, 128)?;

    let path = read_address_array(data, path_offset)?;

    Some(UniswapV2Swap {
        method,
        amount_in: None,
        amount_out_min: None,
        amount_out: Some(amount_out),
        amount_in_max: Some(amount_in_max),
        path,
        to,
        deadline,
    })
}

/// Decodes swapExactETHForTokens(uint256 amountOutMin, address[] path, address to, uint256 deadline)
fn decode_swap_exact_eth_for_tokens(
    calldata: &Bytes,
    method: UniswapV2Method,
) -> Option<UniswapV2Swap> {
    if calldata.len() < 4 + 32 * 4 {
        return None;
    }

    let data = &calldata[4..];

    let amount_out_min = read_u256(data, 0)?;
    let path_offset = read_u256(data, 32)?.to::<usize>();
    let to = read_address(data, 64)?;
    let deadline = read_u256(data, 96)?;

    let path = read_address_array(data, path_offset)?;

    // amount_in comes from msg.value, not calldata
    Some(UniswapV2Swap {
        method,
        amount_in: None, // From msg.value
        amount_out_min: Some(amount_out_min),
        amount_out: None,
        amount_in_max: None,
        path,
        to,
        deadline,
    })
}

/// Decodes swapTokensForExactETH(uint256 amountOut, uint256 amountInMax, address[] path, address to, uint256 deadline)
fn decode_swap_tokens_for_exact_eth(
    calldata: &Bytes,
    method: UniswapV2Method,
) -> Option<UniswapV2Swap> {
    decode_swap_tokens_for_exact_tokens(calldata, method)
}

/// Decodes swapExactTokensForETH(uint256 amountIn, uint256 amountOutMin, address[] path, address to, uint256 deadline)
fn decode_swap_exact_tokens_for_eth(
    calldata: &Bytes,
    method: UniswapV2Method,
) -> Option<UniswapV2Swap> {
    decode_swap_exact_tokens_for_tokens(calldata, method)
}

/// Decodes swapETHForExactTokens(uint256 amountOut, address[] path, address to, uint256 deadline)
fn decode_swap_eth_for_exact_tokens(
    calldata: &Bytes,
    method: UniswapV2Method,
) -> Option<UniswapV2Swap> {
    if calldata.len() < 4 + 32 * 4 {
        return None;
    }

    let data = &calldata[4..];

    let amount_out = read_u256(data, 0)?;
    let path_offset = read_u256(data, 32)?.to::<usize>();
    let to = read_address(data, 64)?;
    let deadline = read_u256(data, 96)?;

    let path = read_address_array(data, path_offset)?;

    Some(UniswapV2Swap {
        method,
        amount_in: None, // From msg.value
        amount_out_min: None,
        amount_out: Some(amount_out),
        amount_in_max: None, // From msg.value
        path,
        to,
        deadline,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real calldata from a swapExactTokensForTokens transaction
    #[test]
    fn test_decode_swap_exact_tokens_for_tokens() {
        // Simplified test with manually constructed calldata
        let mut calldata = Vec::new();

        // Selector: swapExactTokensForTokens
        calldata.extend_from_slice(&[0x38, 0xed, 0x17, 0x39]);

        // amountIn: 1000000 (1e6)
        calldata.extend_from_slice(&U256::from(1_000_000u64).to_be_bytes::<32>());

        // amountOutMin: 990000
        calldata.extend_from_slice(&U256::from(990_000u64).to_be_bytes::<32>());

        // path offset: 160 (0xa0)
        calldata.extend_from_slice(&U256::from(160u64).to_be_bytes::<32>());

        // to: 0x1234...
        let to_addr = Address::repeat_byte(0x12);
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(to_addr.as_slice());

        // deadline: 1700000000
        calldata.extend_from_slice(&U256::from(1_700_000_000u64).to_be_bytes::<32>());

        // path array (at offset 160)
        // length: 2
        calldata.extend_from_slice(&U256::from(2u64).to_be_bytes::<32>());

        // token0
        let token0 = Address::repeat_byte(0xAA);
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(token0.as_slice());

        // token1
        let token1 = Address::repeat_byte(0xBB);
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(token1.as_slice());

        let bytes = Bytes::from(calldata);
        let swap = decode_uniswap_v2(&bytes).expect("should decode");

        assert_eq!(swap.method, UniswapV2Method::SwapExactTokensForTokens);
        assert_eq!(swap.amount_in, Some(U256::from(1_000_000u64)));
        assert_eq!(swap.amount_out_min, Some(U256::from(990_000u64)));
        assert_eq!(swap.path.len(), 2);
        assert_eq!(swap.path[0], token0);
        assert_eq!(swap.path[1], token1);
        assert_eq!(swap.to, to_addr);
        assert!(swap.is_exact_input());
        assert!(!swap.involves_eth());
        assert_eq!(swap.hop_count(), 1);
    }

    #[test]
    fn test_decode_unknown_selector() {
        let calldata = Bytes::from(vec![0x00, 0x00, 0x00, 0x00]);
        assert!(decode_uniswap_v2(&calldata).is_none());
    }

    #[test]
    fn test_decode_too_short() {
        let calldata = Bytes::from(vec![0x38, 0xed, 0x17]);
        assert!(decode_uniswap_v2(&calldata).is_none());
    }
}
