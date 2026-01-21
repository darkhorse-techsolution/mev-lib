//! Swap type definitions.

use alloy_primitives::U256;
use core::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use thiserror::Error;

use super::pool::{Pool, PoolId};
use super::token::TokenId;

/// The direction of a swap in a liquidity pool.
///
/// In a standard liquidity pool with two tokens (token0 and token1),
/// a swap can go in either direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    /// Swap from token0 to token1
    ZeroForOne,
    /// Swap from token1 to token0
    OneForZero,
}

impl Direction {
    /// Returns the opposite direction.
    #[must_use]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::ZeroForOne => Self::OneForZero,
            Self::OneForZero => Self::ZeroForOne,
        }
    }

    /// Returns true if this direction is opposite to another.
    #[must_use]
    pub fn is_opposite(&self, other: &Self) -> bool {
        *self == other.opposite()
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroForOne => write!(f, "0→1"),
            Self::OneForZero => write!(f, "1→0"),
        }
    }
}

/// A unique identifier for a swap operation.
///
/// Combines a pool ID with a direction to uniquely identify a swap path.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SwapId {
    /// The pool where the swap occurs
    pub pool_id: PoolId,
    /// The direction of the swap
    pub direction: Direction,
}

impl SwapId {
    /// Creates a new swap ID.
    #[must_use]
    pub const fn new(pool_id: PoolId, direction: Direction) -> Self {
        Self { pool_id, direction }
    }
}

impl Debug for SwapId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SwapId({:?}, {})", self.pool_id, self.direction)
    }
}

impl Display for SwapId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.pool_id, self.direction)
    }
}

/// Error types for swap operations.
#[derive(Debug, Error)]
pub enum SwapError {
    /// Token in and token out cannot be the same
    #[error("token_in and token_out must be different")]
    SameTokens,
    /// Pool has no liquidity
    #[error("pool has no liquidity")]
    NoLiquidity,
}

/// A swap operation in a liquidity pool.
///
/// Represents swapping from one token to another within a specific pool,
/// including the current reserves for quote calculations.
///
/// # Example
///
/// ```rust
/// use mev_lib::types::{Swap, Pool, PoolId, TokenId, Reserves, Direction};
/// use alloy_primitives::{Address, U256};
///
/// let pool = Pool::new(
///     PoolId::new(Address::ZERO),
///     TokenId::new(Address::ZERO),
///     TokenId::new(Address::ZERO),
///     Reserves::new(U256::from(1_000_000u64), U256::from(1_000_000u64)),
/// );
///
/// // Create a swap in the forward direction (token0 -> token1)
/// let swap = Swap::from_pool(&pool, Direction::ZeroForOne);
/// ```
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Swap {
    id: SwapId,
    token_in: TokenId,
    token_out: TokenId,
    reserve_in: U256,
    reserve_out: U256,
}

impl Swap {
    /// Creates a new swap with explicit parameters.
    ///
    /// # Errors
    ///
    /// Returns an error if `token_in` equals `token_out`.
    pub fn new(
        id: SwapId,
        token_in: TokenId,
        token_out: TokenId,
        reserve_in: U256,
        reserve_out: U256,
    ) -> Result<Self, SwapError> {
        if token_in == token_out {
            return Err(SwapError::SameTokens);
        }

        Ok(Self {
            id,
            token_in,
            token_out,
            reserve_in,
            reserve_out,
        })
    }

    /// Creates a swap from a pool in the specified direction.
    #[must_use]
    pub fn from_pool(pool: &Pool, direction: Direction) -> Self {
        let (token_in, token_out, reserve_in, reserve_out) = match direction {
            Direction::ZeroForOne => (pool.token0, pool.token1, pool.reserve0(), pool.reserve1()),
            Direction::OneForZero => (pool.token1, pool.token0, pool.reserve1(), pool.reserve0()),
        };

        Self {
            id: SwapId::new(pool.id, direction),
            token_in,
            token_out,
            reserve_in,
            reserve_out,
        }
    }

    /// Creates a forward swap (token0 -> token1) from a pool.
    #[must_use]
    pub fn forward(pool: &Pool) -> Self {
        Self::from_pool(pool, Direction::ZeroForOne)
    }

    /// Creates a reverse swap (token1 -> token0) from a pool.
    #[must_use]
    pub fn reverse(pool: &Pool) -> Self {
        Self::from_pool(pool, Direction::OneForZero)
    }

    /// Returns the swap's unique identifier.
    #[must_use]
    pub const fn id(&self) -> &SwapId {
        &self.id
    }

    /// Returns the pool ID.
    #[must_use]
    pub const fn pool_id(&self) -> PoolId {
        self.id.pool_id
    }

    /// Returns the swap direction.
    #[must_use]
    pub const fn direction(&self) -> Direction {
        self.id.direction
    }

    /// Returns the input token.
    #[must_use]
    pub const fn token_in(&self) -> TokenId {
        self.token_in
    }

    /// Returns the output token.
    #[must_use]
    pub const fn token_out(&self) -> TokenId {
        self.token_out
    }

    /// Returns the input token reserve.
    #[must_use]
    pub const fn reserve_in(&self) -> U256 {
        self.reserve_in
    }

    /// Returns the output token reserve.
    #[must_use]
    pub const fn reserve_out(&self) -> U256 {
        self.reserve_out
    }

    /// Returns true if this swap has liquidity (non-zero reserves).
    #[must_use]
    pub fn has_liquidity(&self) -> bool {
        !self.reserve_in.is_zero() && !self.reserve_out.is_zero()
    }

    /// Returns true if this swap is the reciprocal of another.
    ///
    /// Two swaps are reciprocal if they use the same pool but opposite directions.
    #[must_use]
    pub fn is_reciprocal(&self, other: &Self) -> bool {
        self.id.pool_id == other.id.pool_id && self.id.direction.is_opposite(&other.id.direction)
    }
}

impl Debug for Swap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Swap({:?}, {} {} -> {} {})",
            self.id, self.reserve_in, self.token_in, self.reserve_out, self.token_out
        )
    }
}

impl Display for Swap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} {} → {} {}",
            self.id, self.reserve_in, self.token_in, self.reserve_out, self.token_out
        )
    }
}

/// Swaps are equal if they have the same ID, token_in, and token_out.
/// Reserves are not considered in equality (allows updating reserves).
impl PartialEq for Swap {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.token_in == other.token_in && self.token_out == other.token_out
    }
}

impl Eq for Swap {}

impl Hash for Swap {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.token_in.hash(state);
        self.token_out.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Reserves;
    use alloy_primitives::Address;

    fn make_test_pool() -> Pool {
        Pool::new(
            PoolId::new(Address::ZERO),
            TokenId::new(Address::repeat_byte(1)),
            TokenId::new(Address::repeat_byte(2)),
            Reserves::new(U256::from(1_000_000u64), U256::from(2_000_000u64)),
        )
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::ZeroForOne.opposite(), Direction::OneForZero);
        assert_eq!(Direction::OneForZero.opposite(), Direction::ZeroForOne);
        assert!(Direction::ZeroForOne.is_opposite(&Direction::OneForZero));
    }

    #[test]
    fn test_swap_from_pool_forward() {
        let pool = make_test_pool();
        let swap = Swap::forward(&pool);

        assert_eq!(swap.direction(), Direction::ZeroForOne);
        assert_eq!(swap.token_in(), pool.token0);
        assert_eq!(swap.token_out(), pool.token1);
        assert_eq!(swap.reserve_in(), pool.reserve0());
        assert_eq!(swap.reserve_out(), pool.reserve1());
    }

    #[test]
    fn test_swap_from_pool_reverse() {
        let pool = make_test_pool();
        let swap = Swap::reverse(&pool);

        assert_eq!(swap.direction(), Direction::OneForZero);
        assert_eq!(swap.token_in(), pool.token1);
        assert_eq!(swap.token_out(), pool.token0);
        assert_eq!(swap.reserve_in(), pool.reserve1());
        assert_eq!(swap.reserve_out(), pool.reserve0());
    }

    #[test]
    fn test_swap_reciprocal() {
        let pool = make_test_pool();
        let forward = Swap::forward(&pool);
        let reverse = Swap::reverse(&pool);

        assert!(forward.is_reciprocal(&reverse));
        assert!(reverse.is_reciprocal(&forward));
        assert!(!forward.is_reciprocal(&forward));
    }

    #[test]
    fn test_swap_same_tokens_error() {
        let token = TokenId::new(Address::ZERO);
        let result = Swap::new(
            SwapId::new(PoolId::new(Address::ZERO), Direction::ZeroForOne),
            token,
            token,
            U256::from(100u64),
            U256::from(200u64),
        );
        assert!(matches!(result, Err(SwapError::SameTokens)));
    }
}
