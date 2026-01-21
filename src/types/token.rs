//! Token type definitions.

use alloy_primitives::Address;
use core::fmt::{self, Debug, Display};

/// A unique identifier for an ERC-20 token.
///
/// This is a type-safe wrapper around an Ethereum address, providing
/// compile-time guarantees that token addresses aren't mixed up with
/// other address types (like pool addresses).
///
/// # Example
///
/// ```rust
/// use mev_lib::types::TokenId;
/// use alloy_primitives::Address;
///
/// let weth = TokenId::new(Address::ZERO); // Use actual WETH address in production
/// ```
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TokenId(Address);

impl TokenId {
    /// Creates a new `TokenId` from an address.
    #[must_use]
    pub const fn new(address: Address) -> Self {
        Self(address)
    }

    /// Returns the underlying address.
    #[must_use]
    pub const fn address(&self) -> Address {
        self.0
    }
}

impl From<Address> for TokenId {
    fn from(address: Address) -> Self {
        Self(address)
    }
}

impl From<TokenId> for Address {
    fn from(token_id: TokenId) -> Self {
        token_id.0
    }
}

impl Debug for TokenId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TokenId({:?})", self.0)
    }
}

impl Display for TokenId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents an ERC-20 token with its identifier.
///
/// This struct wraps a `TokenId` and can be extended with additional
/// metadata like symbol, decimals, etc.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Token {
    id: TokenId,
}

impl Token {
    /// Creates a new token with the given ID.
    #[must_use]
    pub const fn new(id: TokenId) -> Self {
        Self { id }
    }

    /// Returns the token's ID.
    #[must_use]
    pub const fn id(&self) -> TokenId {
        self.id
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token({:?})", self.id)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_id_from_address() {
        let addr = Address::ZERO;
        let token_id = TokenId::from(addr);
        assert_eq!(token_id.address(), addr);
    }

    #[test]
    fn test_token_id_equality() {
        let addr1 = Address::ZERO;
        let addr2 = Address::ZERO;
        let token1 = TokenId::from(addr1);
        let token2 = TokenId::from(addr2);
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_token_creation() {
        let token_id = TokenId::new(Address::ZERO);
        let token = Token::new(token_id);
        assert_eq!(token.id(), token_id);
    }
}
