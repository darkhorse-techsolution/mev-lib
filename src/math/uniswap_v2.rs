//! Uniswap V2 constant product AMM math.
//!
//! Implements the core formulas for Uniswap V2 and compatible DEXs
//! (SushiSwap, PancakeSwap, etc.) using the constant product formula: x * y = k
//!
//! # Fee Handling
//!
//! Uniswap V2 charges a 0.3% fee (30 basis points) on each swap.
//! The fee is applied to the input amount before the swap calculation.
//!
//! # Example
//!
//! ```rust
//! use mev_lib::math::uniswap_v2;
//! use alloy_primitives::U256;
//!
//! let amount_in = U256::from(1_000_000u64);
//! let reserve_in = U256::from(100_000_000_000u64);
//! let reserve_out = U256::from(100_000_000_000u64);
//!
//! // Standard Uniswap V2 fee (30 basis points = 0.3%)
//! let amount_out = uniswap_v2::get_amount_out(amount_in, reserve_in, reserve_out, 30);
//! ```

use alloy_primitives::U256;

use crate::types::Swap;

/// Basis points denominator (10000 = 100%)
const BPS_DENOMINATOR: u64 = 10_000;

/// Default Uniswap V2 fee in basis points (30 = 0.3%)
pub const DEFAULT_FEE_BPS: u32 = 30;

/// Calculates the output amount for a given input amount.
///
/// Uses the constant product formula with fee deduction:
/// ```text
/// amount_out = (amount_in * (10000 - fee_bps) * reserve_out) / (reserve_in * 10000 + amount_in * (10000 - fee_bps))
/// ```
///
/// # Arguments
///
/// * `amount_in` - The amount of input tokens
/// * `reserve_in` - The reserve of the input token in the pool
/// * `reserve_out` - The reserve of the output token in the pool
/// * `fee_bps` - The swap fee in basis points (30 = 0.3%)
///
/// # Returns
///
/// The amount of output tokens received
///
/// # Panics
///
/// Panics if `reserve_in` or `reserve_out` is zero.
///
/// # Example
///
/// ```rust
/// use mev_lib::math::uniswap_v2::get_amount_out;
/// use alloy_primitives::U256;
///
/// let amount_out = get_amount_out(
///     U256::from(1_000u64),      // amount_in
///     U256::from(1_000_000u64),  // reserve_in
///     U256::from(1_000_000u64),  // reserve_out
///     30,                         // 0.3% fee
/// );
///
/// assert_eq!(amount_out, U256::from(996u64)); // slightly less due to fee + slippage
/// ```
#[must_use]
pub fn get_amount_out(amount_in: U256, reserve_in: U256, reserve_out: U256, fee_bps: u32) -> U256 {
    assert!(!reserve_in.is_zero(), "reserve_in must be non-zero");
    assert!(!reserve_out.is_zero(), "reserve_out must be non-zero");

    if amount_in.is_zero() {
        return U256::ZERO;
    }

    let fee_multiplier = U256::from(BPS_DENOMINATOR - u64::from(fee_bps));
    let denominator_multiplier = U256::from(BPS_DENOMINATOR);

    let amount_in_with_fee = amount_in * fee_multiplier;
    let numerator = amount_in_with_fee * reserve_out;
    let denominator = (reserve_in * denominator_multiplier) + amount_in_with_fee;

    numerator / denominator
}

/// Calculates the input amount required for a desired output amount.
///
/// This is the inverse of `get_amount_out`. Useful for calculating exact output swaps.
///
/// # Arguments
///
/// * `amount_out` - The desired amount of output tokens
/// * `reserve_in` - The reserve of the input token in the pool
/// * `reserve_out` - The reserve of the output token in the pool
/// * `fee_bps` - The swap fee in basis points (30 = 0.3%)
///
/// # Returns
///
/// The amount of input tokens required, or `None` if the output amount exceeds available liquidity.
///
/// # Example
///
/// ```rust
/// use mev_lib::math::uniswap_v2::get_amount_in;
/// use alloy_primitives::U256;
///
/// let amount_in = get_amount_in(
///     U256::from(996u64),        // amount_out
///     U256::from(1_000_000u64),  // reserve_in
///     U256::from(1_000_000u64),  // reserve_out
///     30,                         // 0.3% fee
/// );
///
/// // Returns Some(amount_in) needed to get 996 tokens out
/// assert!(amount_in.is_some());
/// ```
#[must_use]
pub fn get_amount_in(
    amount_out: U256,
    reserve_in: U256,
    reserve_out: U256,
    fee_bps: u32,
) -> Option<U256> {
    if reserve_in.is_zero() || reserve_out.is_zero() {
        return None;
    }

    if amount_out.is_zero() {
        return Some(U256::ZERO);
    }

    // Cannot get more out than the reserve
    if amount_out >= reserve_out {
        return None;
    }

    let fee_multiplier = U256::from(BPS_DENOMINATOR - u64::from(fee_bps));
    let denominator_multiplier = U256::from(BPS_DENOMINATOR);

    let numerator = reserve_in * amount_out * denominator_multiplier;
    let denominator = (reserve_out - amount_out) * fee_multiplier;

    // Round up to ensure we get at least amount_out
    Some((numerator / denominator) + U256::from(1u64))
}

/// Calculates the output amount for a swap using the swap's reserves.
///
/// This is a convenience function that extracts reserves from a `Swap` struct.
///
/// # Arguments
///
/// * `swap` - The swap containing reserve information
/// * `amount_in` - The amount of input tokens
/// * `fee_bps` - The swap fee in basis points
///
/// # Returns
///
/// The amount of output tokens received
#[must_use]
pub fn quote_swap(swap: &Swap, amount_in: U256, fee_bps: u32) -> U256 {
    get_amount_out(amount_in, swap.reserve_in(), swap.reserve_out(), fee_bps)
}

/// Calculates the output amount using the default Uniswap V2 fee (0.3%).
///
/// This is a convenience function for the most common case.
#[must_use]
pub fn get_amount_out_default(amount_in: U256, reserve_in: U256, reserve_out: U256) -> U256 {
    get_amount_out(amount_in, reserve_in, reserve_out, DEFAULT_FEE_BPS)
}

/// Calculates the spot price (without slippage) for a swap direction.
///
/// Returns the price as `reserve_out / reserve_in`, representing how many
/// output tokens you get per input token at infinitesimally small trade sizes.
///
/// # Note
///
/// This does not account for fees. For fee-adjusted price, multiply by `(1 - fee)`.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn spot_price(reserve_in: U256, reserve_out: U256) -> f64 {
    if reserve_in.is_zero() {
        return 0.0;
    }

    // Convert to f64 for price calculation
    // This may lose precision for very large reserves, but is acceptable for price display
    let reserve_in_f64: f64 = reserve_in.to_string().parse().unwrap_or(f64::MAX);
    let reserve_out_f64: f64 = reserve_out.to_string().parse().unwrap_or(f64::MAX);

    reserve_out_f64 / reserve_in_f64
}

/// Calculates the price impact of a swap as a percentage.
///
/// Price impact is how much worse the execution price is compared to the spot price.
/// A higher price impact means more slippage.
///
/// # Returns
///
/// Price impact as a decimal (e.g., 0.01 = 1% impact)
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn price_impact(amount_in: U256, reserve_in: U256, reserve_out: U256, fee_bps: u32) -> f64 {
    if amount_in.is_zero() || reserve_in.is_zero() || reserve_out.is_zero() {
        return 0.0;
    }

    let spot = spot_price(reserve_in, reserve_out);
    let amount_out = get_amount_out(amount_in, reserve_in, reserve_out, fee_bps);

    let amount_in_f64: f64 = amount_in.to_string().parse().unwrap_or(f64::MAX);
    let amount_out_f64: f64 = amount_out.to_string().parse().unwrap_or(0.0);

    let execution_price = amount_out_f64 / amount_in_f64;
    let fee_adjusted_spot = spot * (1.0 - f64::from(fee_bps) / f64::from(BPS_DENOMINATOR as u32));

    if fee_adjusted_spot == 0.0 {
        return 0.0;
    }

    1.0 - (execution_price / fee_adjusted_spot)
}

/// Calculates the new reserves after a swap.
///
/// # Arguments
///
/// * `amount_in` - The amount of input tokens
/// * `reserve_in` - The current reserve of the input token
/// * `reserve_out` - The current reserve of the output token
/// * `fee_bps` - The swap fee in basis points
///
/// # Returns
///
/// Tuple of (new_reserve_in, new_reserve_out, amount_out)
#[must_use]
pub fn simulate_swap(
    amount_in: U256,
    reserve_in: U256,
    reserve_out: U256,
    fee_bps: u32,
) -> (U256, U256, U256) {
    let amount_out = get_amount_out(amount_in, reserve_in, reserve_out, fee_bps);

    let new_reserve_in = reserve_in + amount_in;
    let new_reserve_out = reserve_out - amount_out;

    (new_reserve_in, new_reserve_out, amount_out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_amount_out_basic() {
        // 1:1 pool, small trade
        let amount_out = get_amount_out(
            U256::from(1_000u64),
            U256::from(1_000_000u64),
            U256::from(1_000_000u64),
            30,
        );

        // Should get slightly less than 1000 due to fee (0.3%) and slippage
        assert_eq!(amount_out, U256::from(996u64));
    }

    #[test]
    fn test_get_amount_out_larger_trade() {
        // Test with larger trade to see slippage
        let amount_out = get_amount_out(
            U256::from(10_000_000u64),
            U256::from(1_000_000_000u64),
            U256::from(1_000_000_000u64),
            30,
        );

        // Expected: ~9,871,580 (significant slippage at 1% of pool)
        assert_eq!(amount_out, U256::from(9_871_580u64));
    }

    #[test]
    fn test_get_amount_out_drains_pool() {
        // Even with huge input, can't get more than reserve_out - 1
        let amount_out = get_amount_out(
            U256::from(1_000_000_000u64),
            U256::from(1_000u64),
            U256::from(1_000u64),
            30,
        );

        // Maximum output is reserve_out - 1 = 999
        assert_eq!(amount_out, U256::from(999u64));
    }

    #[test]
    fn test_get_amount_out_zero_input() {
        let amount_out = get_amount_out(
            U256::ZERO,
            U256::from(1_000_000u64),
            U256::from(1_000_000u64),
            30,
        );

        assert_eq!(amount_out, U256::ZERO);
    }

    #[test]
    fn test_get_amount_in_basic() {
        // Get the input needed for a specific output
        let amount_in = get_amount_in(
            U256::from(996u64),
            U256::from(1_000_000u64),
            U256::from(1_000_000u64),
            30,
        );

        assert!(amount_in.is_some());
        // Should be close to 1000
        let amount_in = amount_in.expect("should have amount");
        assert!(amount_in >= U256::from(1000u64));
        assert!(amount_in <= U256::from(1010u64));
    }

    #[test]
    fn test_get_amount_in_exceeds_reserve() {
        // Can't get more than the reserve
        let amount_in = get_amount_in(
            U256::from(1_000_001u64),
            U256::from(1_000_000u64),
            U256::from(1_000_000u64),
            30,
        );

        assert!(amount_in.is_none());
    }

    #[test]
    fn test_simulate_swap() {
        let (new_reserve_in, new_reserve_out, amount_out) = simulate_swap(
            U256::from(1_000u64),
            U256::from(1_000_000u64),
            U256::from(1_000_000u64),
            30,
        );

        assert_eq!(new_reserve_in, U256::from(1_001_000u64));
        assert_eq!(new_reserve_out, U256::from(999_004u64));
        assert_eq!(amount_out, U256::from(996u64));
    }

    #[test]
    fn test_different_fee_tiers() {
        let reserve_in = U256::from(1_000_000u64);
        let reserve_out = U256::from(1_000_000u64);
        let amount_in = U256::from(10_000u64);

        // 0.3% fee (standard Uniswap V2)
        let out_30bps = get_amount_out(amount_in, reserve_in, reserve_out, 30);

        // 0.05% fee (like Uniswap V3 lowest tier)
        let out_5bps = get_amount_out(amount_in, reserve_in, reserve_out, 5);

        // 1% fee
        let out_100bps = get_amount_out(amount_in, reserve_in, reserve_out, 100);

        // Lower fee = more output
        assert!(out_5bps > out_30bps);
        assert!(out_30bps > out_100bps);
    }

    #[test]
    fn test_spot_price() {
        // 2:1 pool (2 token0 per token1)
        let price = spot_price(U256::from(2_000_000u64), U256::from(1_000_000u64));
        assert!((price - 0.5).abs() < 0.0001);

        // 1:2 pool
        let price = spot_price(U256::from(1_000_000u64), U256::from(2_000_000u64));
        assert!((price - 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_price_impact() {
        // Small trade = low impact
        let impact_small = price_impact(
            U256::from(1_000u64),
            U256::from(1_000_000_000u64),
            U256::from(1_000_000_000u64),
            30,
        );

        // Large trade = high impact
        let impact_large = price_impact(
            U256::from(100_000_000u64),
            U256::from(1_000_000_000u64),
            U256::from(1_000_000_000u64),
            30,
        );

        assert!(impact_small < 0.01); // < 1% for tiny trade
        assert!(impact_large > 0.05); // > 5% for 10% of pool
        assert!(impact_large > impact_small);
    }
}
