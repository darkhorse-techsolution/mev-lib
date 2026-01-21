//! ERC20 token calldata decoder.
//!
//! Decodes common ERC20 methods: transfer, transferFrom, approve.
//!
//! # Example
//!
//! ```rust,ignore
//! use mev_lib::decoder::{decode_erc20_transfer, Erc20Transfer};
//! use alloy_primitives::Bytes;
//!
//! let calldata: Bytes = /* ... */;
//! if let Some(transfer) = decode_erc20_transfer(&calldata) {
//!     println!("Transfer {} tokens to {}", transfer.amount, transfer.to);
//! }
//! ```

use alloy_primitives::{Address, Bytes, U256};

use super::selector::{erc20 as selectors, MethodSelector};

/// Decoded ERC20 transfer call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Erc20Transfer {
    /// Recipient address
    pub to: Address,
    /// Amount of tokens
    pub amount: U256,
}

/// Decoded ERC20 transferFrom call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Erc20TransferFrom {
    /// Source address (tokens are taken from here)
    pub from: Address,
    /// Recipient address
    pub to: Address,
    /// Amount of tokens
    pub amount: U256,
}

/// Decoded ERC20 approve call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Erc20Approve {
    /// Spender address (allowed to spend tokens)
    pub spender: Address,
    /// Approved amount
    pub amount: U256,
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

/// Decodes an ERC20 transfer call.
///
/// # Signature
///
/// `transfer(address to, uint256 amount)`
///
/// # Arguments
///
/// * `calldata` - Raw transaction input data
///
/// # Returns
///
/// `Some(Erc20Transfer)` if the calldata is a valid transfer call, `None` otherwise.
#[must_use]
pub fn decode_erc20_transfer(calldata: &Bytes) -> Option<Erc20Transfer> {
    if calldata.len() < 4 + 64 {
        return None;
    }

    let selector = MethodSelector::from_bytes(&calldata[..4]);
    if selector != selectors::TRANSFER {
        return None;
    }

    let data = &calldata[4..];
    let to = read_address(data, 0)?;
    let amount = read_u256(data, 32)?;

    Some(Erc20Transfer { to, amount })
}

/// Decodes an ERC20 transferFrom call.
///
/// # Signature
///
/// `transferFrom(address from, address to, uint256 amount)`
///
/// # Arguments
///
/// * `calldata` - Raw transaction input data
///
/// # Returns
///
/// `Some(Erc20TransferFrom)` if the calldata is a valid transferFrom call, `None` otherwise.
#[must_use]
pub fn decode_erc20_transfer_from(calldata: &Bytes) -> Option<Erc20TransferFrom> {
    if calldata.len() < 4 + 96 {
        return None;
    }

    let selector = MethodSelector::from_bytes(&calldata[..4]);
    if selector != selectors::TRANSFER_FROM {
        return None;
    }

    let data = &calldata[4..];
    let from = read_address(data, 0)?;
    let to = read_address(data, 32)?;
    let amount = read_u256(data, 64)?;

    Some(Erc20TransferFrom { from, to, amount })
}

/// Decodes an ERC20 approve call.
///
/// # Signature
///
/// `approve(address spender, uint256 amount)`
///
/// # Arguments
///
/// * `calldata` - Raw transaction input data
///
/// # Returns
///
/// `Some(Erc20Approve)` if the calldata is a valid approve call, `None` otherwise.
#[must_use]
pub fn decode_erc20_approve(calldata: &Bytes) -> Option<Erc20Approve> {
    if calldata.len() < 4 + 64 {
        return None;
    }

    let selector = MethodSelector::from_bytes(&calldata[..4]);
    if selector != selectors::APPROVE {
        return None;
    }

    let data = &calldata[4..];
    let spender = read_address(data, 0)?;
    let amount = read_u256(data, 32)?;

    Some(Erc20Approve { spender, amount })
}

/// Checks if the calldata is an ERC20 transfer or transferFrom.
#[must_use]
pub fn is_token_transfer(calldata: &Bytes) -> bool {
    if calldata.len() < 4 {
        return false;
    }

    let selector = MethodSelector::from_bytes(&calldata[..4]);
    selector == selectors::TRANSFER || selector == selectors::TRANSFER_FROM
}

/// Checks if the amount is a max approval (type(uint256).max).
#[must_use]
pub fn is_max_approval(amount: U256) -> bool {
    amount == U256::MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_transfer() {
        let mut calldata = Vec::new();

        // Selector: transfer
        calldata.extend_from_slice(&[0xa9, 0x05, 0x9c, 0xbb]);

        // to address
        let to = Address::repeat_byte(0x42);
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(to.as_slice());

        // amount: 1000
        calldata.extend_from_slice(&U256::from(1000u64).to_be_bytes::<32>());

        let bytes = Bytes::from(calldata);
        let transfer = decode_erc20_transfer(&bytes).expect("should decode");

        assert_eq!(transfer.to, to);
        assert_eq!(transfer.amount, U256::from(1000u64));
    }

    #[test]
    fn test_decode_transfer_from() {
        let mut calldata = Vec::new();

        // Selector: transferFrom
        calldata.extend_from_slice(&[0x23, 0xb8, 0x72, 0xdd]);

        // from address
        let from = Address::repeat_byte(0x11);
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(from.as_slice());

        // to address
        let to = Address::repeat_byte(0x22);
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(to.as_slice());

        // amount: 5000
        calldata.extend_from_slice(&U256::from(5000u64).to_be_bytes::<32>());

        let bytes = Bytes::from(calldata);
        let transfer = decode_erc20_transfer_from(&bytes).expect("should decode");

        assert_eq!(transfer.from, from);
        assert_eq!(transfer.to, to);
        assert_eq!(transfer.amount, U256::from(5000u64));
    }

    #[test]
    fn test_decode_approve() {
        let mut calldata = Vec::new();

        // Selector: approve
        calldata.extend_from_slice(&[0x09, 0x5e, 0xa7, 0xb3]);

        // spender address
        let spender = Address::repeat_byte(0x33);
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(spender.as_slice());

        // amount: max
        calldata.extend_from_slice(&U256::MAX.to_be_bytes::<32>());

        let bytes = Bytes::from(calldata);
        let approve = decode_erc20_approve(&bytes).expect("should decode");

        assert_eq!(approve.spender, spender);
        assert!(is_max_approval(approve.amount));
    }

    #[test]
    fn test_is_token_transfer() {
        // Transfer selector
        let transfer = Bytes::from(vec![0xa9, 0x05, 0x9c, 0xbb, 0x00]);
        assert!(is_token_transfer(&transfer));

        // TransferFrom selector
        let transfer_from = Bytes::from(vec![0x23, 0xb8, 0x72, 0xdd, 0x00]);
        assert!(is_token_transfer(&transfer_from));

        // Approve selector (not a transfer)
        let approve = Bytes::from(vec![0x09, 0x5e, 0xa7, 0xb3, 0x00]);
        assert!(!is_token_transfer(&approve));
    }
}
