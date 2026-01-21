//! Uniswap V3 Router calldata decoder.
//!
//! Decodes swap calls from Uniswap V3 SwapRouter and SwapRouter02.
//!
//! # Path Encoding
//!
//! V3 paths are encoded differently from V2:
//! - V2: `address[]` - array of token addresses
//! - V3: `bytes` - packed encoding of `token, fee, token, fee, token`
//!
//! Each fee is a uint24 (3 bytes), so a single-hop path is:
//! `20 bytes (token) + 3 bytes (fee) + 20 bytes (token) = 43 bytes`
//!
//! # Supported Methods
//!
//! - `exactInputSingle` - single pool swap with exact input
//! - `exactInput` - multi-hop swap with exact input
//! - `exactOutputSingle` - single pool swap with exact output
//! - `exactOutput` - multi-hop swap with exact output

use alloy_primitives::{Address, Bytes, U256};

use super::selector::{uniswap_v3 as selectors, MethodSelector};

/// Decoded Uniswap V3 swap parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniswapV3Swap {
    /// The method that was called
    pub method: UniswapV3Method,
    /// Input token address
    pub token_in: Address,
    /// Output token address
    pub token_out: Address,
    /// Fee tier(s) in hundredths of a bip (e.g., 3000 = 0.3%)
    pub fees: Vec<u32>,
    /// Recipient address
    pub recipient: Address,
    /// Amount of input tokens (for exactInput methods)
    pub amount_in: Option<U256>,
    /// Minimum output tokens expected (for exactInput methods)
    pub amount_out_min: Option<U256>,
    /// Exact output tokens desired (for exactOutput methods)
    pub amount_out: Option<U256>,
    /// Maximum input tokens willing to spend (for exactOutput methods)
    pub amount_in_max: Option<U256>,
    /// Price limit (sqrtPriceLimitX96) - 0 means no limit
    pub sqrt_price_limit: Option<U256>,
    /// Transaction deadline
    pub deadline: Option<U256>,
    /// Raw path bytes (for multi-hop swaps)
    pub raw_path: Option<Bytes>,
}

/// Uniswap V3 swap method types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniswapV3Method {
    /// exactInputSingle - single pool, exact input amount
    ExactInputSingle,
    /// exactInput - multi-hop, exact input amount
    ExactInput,
    /// exactOutputSingle - single pool, exact output amount
    ExactOutputSingle,
    /// exactOutput - multi-hop, exact output amount
    ExactOutput,
}

/// V3 fee tiers in hundredths of a bip.
pub mod fee_tiers {
    /// 0.01% fee tier (1 bip)
    pub const LOWEST: u32 = 100;
    /// 0.05% fee tier (5 bips)
    pub const LOW: u32 = 500;
    /// 0.3% fee tier (30 bips) - most common
    pub const MEDIUM: u32 = 3000;
    /// 1% fee tier (100 bips)
    pub const HIGH: u32 = 10000;
}

impl UniswapV3Swap {
    /// Returns the token path as a vector of addresses.
    ///
    /// For single-hop swaps, returns `[token_in, token_out]`.
    /// For multi-hop swaps, decodes the full path.
    #[must_use]
    pub fn path(&self) -> Vec<Address> {
        if let Some(ref raw) = self.raw_path {
            decode_v3_path(raw)
        } else {
            vec![self.token_in, self.token_out]
        }
    }

    /// Returns true if this is an exact input swap.
    #[must_use]
    pub fn is_exact_input(&self) -> bool {
        matches!(
            self.method,
            UniswapV3Method::ExactInputSingle | UniswapV3Method::ExactInput
        )
    }

    /// Returns true if this is a multi-hop swap.
    #[must_use]
    pub fn is_multi_hop(&self) -> bool {
        matches!(
            self.method,
            UniswapV3Method::ExactInput | UniswapV3Method::ExactOutput
        )
    }

    /// Returns the number of hops in the swap.
    #[must_use]
    pub fn hop_count(&self) -> usize {
        self.fees.len()
    }

    /// Returns the primary fee tier (first hop).
    #[must_use]
    pub fn primary_fee(&self) -> Option<u32> {
        self.fees.first().copied()
    }
}

/// Decodes a V3 path into token addresses.
///
/// Path format: `token (20 bytes) + fee (3 bytes) + token (20 bytes) + ...`
#[must_use]
pub fn decode_v3_path(path: &Bytes) -> Vec<Address> {
    let mut tokens = Vec::new();

    if path.len() < 20 {
        return tokens;
    }

    // First token
    tokens.push(Address::from_slice(&path[0..20]));

    // Each hop is 23 bytes (3 fee + 20 token)
    let mut offset = 20;
    while offset + 23 <= path.len() {
        // Skip fee (3 bytes)
        offset += 3;
        // Read token
        tokens.push(Address::from_slice(&path[offset..offset + 20]));
        offset += 20;
    }

    tokens
}

/// Extracts fee tiers from a V3 path.
#[must_use]
pub fn decode_v3_path_fees(path: &Bytes) -> Vec<u32> {
    let mut fees = Vec::new();

    if path.len() < 43 {
        // Minimum: 20 + 3 + 20
        return fees;
    }

    let mut offset = 20; // Start after first token
    while offset + 3 <= path.len() {
        // Read fee as uint24 (big endian)
        let fee = u32::from(path[offset]) << 16
            | u32::from(path[offset + 1]) << 8
            | u32::from(path[offset + 2]);
        fees.push(fee);
        offset += 23; // fee (3) + token (20)
    }

    fees
}

/// Decodes Uniswap V3 router calldata.
///
/// # Arguments
///
/// * `calldata` - Raw transaction input data
///
/// # Returns
///
/// `Some(UniswapV3Swap)` if the calldata is a valid V3 swap, `None` otherwise.
#[must_use]
pub fn decode_uniswap_v3(calldata: &Bytes) -> Option<UniswapV3Swap> {
    if calldata.len() < 4 {
        return None;
    }

    let selector = MethodSelector::from_bytes(&calldata[..4]);

    match selector {
        selectors::EXACT_INPUT_SINGLE => decode_exact_input_single(calldata),
        selectors::EXACT_INPUT => decode_exact_input(calldata),
        selectors::EXACT_OUTPUT_SINGLE => decode_exact_output_single(calldata),
        selectors::EXACT_OUTPUT => decode_exact_output(calldata),
        _ => None,
    }
}

/// Reads a U256 from data at the given offset.
fn read_u256(data: &[u8], offset: usize) -> Option<U256> {
    if offset + 32 > data.len() {
        return None;
    }
    Some(U256::from_be_slice(&data[offset..offset + 32]))
}

/// Reads an Address from data at the given offset.
fn read_address(data: &[u8], offset: usize) -> Option<Address> {
    if offset + 32 > data.len() {
        return None;
    }
    Some(Address::from_slice(&data[offset + 12..offset + 32]))
}

/// Reads a uint24 (fee) from data at the given offset (in a 32-byte slot).
fn read_uint24(data: &[u8], offset: usize) -> Option<u32> {
    if offset + 32 > data.len() {
        return None;
    }
    // uint24 is right-aligned in the 32-byte slot
    let value = U256::from_be_slice(&data[offset..offset + 32]);
    Some(value.to::<u32>())
}

/// Reads dynamic bytes from data.
fn read_bytes(data: &[u8], bytes_offset: usize) -> Option<Bytes> {
    let length = read_u256(data, bytes_offset)?.to::<usize>();
    if bytes_offset + 32 + length > data.len() {
        return None;
    }
    Some(Bytes::copy_from_slice(
        &data[bytes_offset + 32..bytes_offset + 32 + length],
    ))
}

/// Decodes exactInputSingle struct:
/// (address tokenIn, address tokenOut, uint24 fee, address recipient, uint256 deadline,
///  uint256 amountIn, uint256 amountOutMinimum, uint160 sqrtPriceLimitX96)
fn decode_exact_input_single(calldata: &Bytes) -> Option<UniswapV3Swap> {
    let data = &calldata[4..]; // Skip selector

    // The struct is passed as a tuple, so we read fields sequentially
    // Note: SwapRouter02 has a slightly different struct (no deadline in struct)
    // This handles the original SwapRouter format

    if data.len() < 32 * 8 {
        return None;
    }

    let token_in = read_address(data, 0)?;
    let token_out = read_address(data, 32)?;
    let fee = read_uint24(data, 64)?;
    let recipient = read_address(data, 96)?;
    let deadline = read_u256(data, 128)?;
    let amount_in = read_u256(data, 160)?;
    let amount_out_min = read_u256(data, 192)?;
    let sqrt_price_limit = read_u256(data, 224)?;

    Some(UniswapV3Swap {
        method: UniswapV3Method::ExactInputSingle,
        token_in,
        token_out,
        fees: vec![fee],
        recipient,
        amount_in: Some(amount_in),
        amount_out_min: Some(amount_out_min),
        amount_out: None,
        amount_in_max: None,
        sqrt_price_limit: Some(sqrt_price_limit),
        deadline: Some(deadline),
        raw_path: None,
    })
}

/// Decodes exactInput struct:
/// (bytes path, address recipient, uint256 deadline, uint256 amountIn, uint256 amountOutMinimum)
fn decode_exact_input(calldata: &Bytes) -> Option<UniswapV3Swap> {
    let data = &calldata[4..];

    if data.len() < 32 * 5 {
        return None;
    }

    let path_offset = read_u256(data, 0)?.to::<usize>();
    let recipient = read_address(data, 32)?;
    let deadline = read_u256(data, 64)?;
    let amount_in = read_u256(data, 96)?;
    let amount_out_min = read_u256(data, 128)?;

    let path = read_bytes(data, path_offset)?;
    let tokens = decode_v3_path(&path);
    let fees = decode_v3_path_fees(&path);

    if tokens.len() < 2 {
        return None;
    }

    Some(UniswapV3Swap {
        method: UniswapV3Method::ExactInput,
        token_in: tokens[0],
        token_out: *tokens.last()?,
        fees,
        recipient,
        amount_in: Some(amount_in),
        amount_out_min: Some(amount_out_min),
        amount_out: None,
        amount_in_max: None,
        sqrt_price_limit: None,
        deadline: Some(deadline),
        raw_path: Some(path),
    })
}

/// Decodes exactOutputSingle struct:
/// (address tokenIn, address tokenOut, uint24 fee, address recipient, uint256 deadline,
///  uint256 amountOut, uint256 amountInMaximum, uint160 sqrtPriceLimitX96)
fn decode_exact_output_single(calldata: &Bytes) -> Option<UniswapV3Swap> {
    let data = &calldata[4..];

    if data.len() < 32 * 8 {
        return None;
    }

    let token_in = read_address(data, 0)?;
    let token_out = read_address(data, 32)?;
    let fee = read_uint24(data, 64)?;
    let recipient = read_address(data, 96)?;
    let deadline = read_u256(data, 128)?;
    let amount_out = read_u256(data, 160)?;
    let amount_in_max = read_u256(data, 192)?;
    let sqrt_price_limit = read_u256(data, 224)?;

    Some(UniswapV3Swap {
        method: UniswapV3Method::ExactOutputSingle,
        token_in,
        token_out,
        fees: vec![fee],
        recipient,
        amount_in: None,
        amount_out_min: None,
        amount_out: Some(amount_out),
        amount_in_max: Some(amount_in_max),
        sqrt_price_limit: Some(sqrt_price_limit),
        deadline: Some(deadline),
        raw_path: None,
    })
}

/// Decodes exactOutput struct:
/// (bytes path, address recipient, uint256 deadline, uint256 amountOut, uint256 amountInMaximum)
fn decode_exact_output(calldata: &Bytes) -> Option<UniswapV3Swap> {
    let data = &calldata[4..];

    if data.len() < 32 * 5 {
        return None;
    }

    let path_offset = read_u256(data, 0)?.to::<usize>();
    let recipient = read_address(data, 32)?;
    let deadline = read_u256(data, 64)?;
    let amount_out = read_u256(data, 96)?;
    let amount_in_max = read_u256(data, 128)?;

    let path = read_bytes(data, path_offset)?;
    let tokens = decode_v3_path(&path);
    let fees = decode_v3_path_fees(&path);

    if tokens.len() < 2 {
        return None;
    }

    // Note: For exactOutput, the path is reversed (tokenOut first)
    Some(UniswapV3Swap {
        method: UniswapV3Method::ExactOutput,
        token_in: *tokens.last()?, // Path is reversed for exactOutput
        token_out: tokens[0],
        fees,
        recipient,
        amount_in: None,
        amount_out_min: None,
        amount_out: Some(amount_out),
        amount_in_max: Some(amount_in_max),
        sqrt_price_limit: None,
        deadline: Some(deadline),
        raw_path: Some(path),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_v3_path_single_hop() {
        // token0 (20 bytes) + fee 3000 (3 bytes) + token1 (20 bytes)
        let mut path = Vec::new();
        path.extend_from_slice(&[0xAA; 20]); // token0
        path.extend_from_slice(&[0x00, 0x0B, 0xB8]); // fee 3000
        path.extend_from_slice(&[0xBB; 20]); // token1

        let tokens = decode_v3_path(&Bytes::from(path.clone()));
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Address::repeat_byte(0xAA));
        assert_eq!(tokens[1], Address::repeat_byte(0xBB));

        let fees = decode_v3_path_fees(&Bytes::from(path));
        assert_eq!(fees, vec![3000]);
    }

    #[test]
    fn test_decode_v3_path_multi_hop() {
        // token0 + fee + token1 + fee + token2
        let mut path = Vec::new();
        path.extend_from_slice(&[0xAA; 20]); // token0
        path.extend_from_slice(&[0x00, 0x0B, 0xB8]); // fee 3000
        path.extend_from_slice(&[0xBB; 20]); // token1
        path.extend_from_slice(&[0x00, 0x01, 0xF4]); // fee 500
        path.extend_from_slice(&[0xCC; 20]); // token2

        let tokens = decode_v3_path(&Bytes::from(path.clone()));
        assert_eq!(tokens.len(), 3);

        let fees = decode_v3_path_fees(&Bytes::from(path));
        assert_eq!(fees, vec![3000, 500]);
    }

    #[test]
    fn test_fee_tiers() {
        assert_eq!(fee_tiers::LOWEST, 100);
        assert_eq!(fee_tiers::LOW, 500);
        assert_eq!(fee_tiers::MEDIUM, 3000);
        assert_eq!(fee_tiers::HIGH, 10000);
    }

    #[test]
    fn test_unknown_selector() {
        let calldata = Bytes::from(vec![0x00, 0x00, 0x00, 0x00]);
        assert!(decode_uniswap_v3(&calldata).is_none());
    }
}
