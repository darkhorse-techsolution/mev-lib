//! Pool type definitions.

use alloy_primitives::{Address, U256};
use core::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};

use super::token::TokenId;

/// A unique identifier for a liquidity pool.
///
/// This is a type-safe wrapper around an Ethereum address, providing
/// compile-time guarantees that pool addresses aren't mixed up with
/// other address types.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoolId(Address);

impl PoolId {
    /// Creates a new `PoolId` from an address.
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

impl From<Address> for PoolId {
    fn from(address: Address) -> Self {
        Self(address)
    }
}

impl From<PoolId> for Address {
    fn from(pool_id: PoolId) -> Self {
        pool_id.0
    }
}

impl Debug for PoolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PoolId({:?})", self.0)
    }
}

impl Display for PoolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Token reserves in a liquidity pool.
///
/// Represents the amounts of token0 and token1 held by the pool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Reserves {
    /// Reserve amount of token0
    pub reserve0: U256,
    /// Reserve amount of token1
    pub reserve1: U256,
}

impl Reserves {
    /// Creates new reserves.
    #[must_use]
    pub const fn new(reserve0: U256, reserve1: U256) -> Self {
        Self { reserve0, reserve1 }
    }

    /// Returns true if both reserves are non-zero.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.reserve0.is_zero() && !self.reserve1.is_zero()
    }
}

/// A liquidity pool containing two tokens.
///
/// Represents an AMM pool (like Uniswap V2) with token pair and reserves.
///
/// # Example
///
/// ```rust
/// use mev_lib::types::{Pool, PoolId, TokenId, Reserves};
/// use alloy_primitives::{Address, U256};
///
/// let pool = Pool::new(
///     PoolId::new(Address::ZERO),
///     TokenId::new(Address::ZERO),  // token0
///     TokenId::new(Address::ZERO),  // token1
///     Reserves::new(
///         U256::from(1_000_000_000u64),
///         U256::from(1_000_000_000u64),
///     ),
/// );
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pool {
    /// The unique identifier for this pool
    pub id: PoolId,
    /// The first token in the pool (token0)
    pub token0: TokenId,
    /// The second token in the pool (token1)
    pub token1: TokenId,
    /// The reserves of both tokens
    pub reserves: Reserves,
}

impl Pool {
    /// Creates a new pool.
    #[must_use]
    pub const fn new(id: PoolId, token0: TokenId, token1: TokenId, reserves: Reserves) -> Self {
        Self {
            id,
            token0,
            token1,
            reserves,
        }
    }

    /// Returns the reserve of token0.
    #[must_use]
    pub const fn reserve0(&self) -> U256 {
        self.reserves.reserve0
    }

    /// Returns the reserve of token1.
    #[must_use]
    pub const fn reserve1(&self) -> U256 {
        self.reserves.reserve1
    }

    /// Returns true if the pool has valid (non-zero) reserves.
    #[must_use]
    pub fn has_liquidity(&self) -> bool {
        self.reserves.is_valid()
    }
}

/// Two pools are equal if they have the same ID (address).
impl PartialEq for Pool {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Pool {}

/// Hash by pool ID only.
impl Hash for Pool {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_id_from_address() {
        let addr = Address::ZERO;
        let pool_id = PoolId::from(addr);
        assert_eq!(pool_id.address(), addr);
    }

    #[test]
    fn test_reserves_validity() {
        let valid = Reserves::new(U256::from(100u64), U256::from(200u64));
        assert!(valid.is_valid());

        let invalid = Reserves::new(U256::ZERO, U256::from(200u64));
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_pool_equality() {
        let pool1 = Pool::new(
            PoolId::new(Address::ZERO),
            TokenId::new(Address::ZERO),
            TokenId::new(Address::ZERO),
            Reserves::new(U256::from(100u64), U256::from(200u64)),
        );

        let pool2 = Pool::new(
            PoolId::new(Address::ZERO),
            TokenId::new(Address::ZERO),
            TokenId::new(Address::ZERO),
            Reserves::new(U256::from(300u64), U256::from(400u64)), // different reserves
        );

        // Pools are equal if same ID, regardless of reserves
        assert_eq!(pool1, pool2);
    }
}
