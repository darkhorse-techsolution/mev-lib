//! Result analysis utilities for MEV detection.
//!
//! This module provides utilities for analyzing simulation results
//! to detect MEV opportunities and calculate profits.

use alloy_primitives::{Address, U256, I256};

use super::types::{SimulationLog, SimulationResult, event_signatures};

/// Analyzes simulation logs to detect swap events.
#[derive(Debug, Clone)]
pub struct SwapAnalyzer;

impl SwapAnalyzer {
    /// Finds all Uniswap V2 swap events in the logs.
    #[must_use]
    pub fn find_v2_swaps(logs: &[SimulationLog]) -> Vec<UniswapV2SwapEvent> {
        logs.iter()
            .filter(|log| log.matches_signature(event_signatures::UNISWAP_V2_SWAP))
            .filter_map(|log| Self::decode_v2_swap(log))
            .collect()
    }

    /// Finds all Uniswap V3 swap events in the logs.
    #[must_use]
    pub fn find_v3_swaps(logs: &[SimulationLog]) -> Vec<UniswapV3SwapEvent> {
        logs.iter()
            .filter(|log| log.matches_signature(event_signatures::UNISWAP_V3_SWAP))
            .filter_map(|log| Self::decode_v3_swap(log))
            .collect()
    }

    /// Finds all ERC20 transfer events in the logs.
    #[must_use]
    pub fn find_transfers(logs: &[SimulationLog]) -> Vec<Erc20TransferEvent> {
        logs.iter()
            .filter(|log| log.matches_signature(event_signatures::ERC20_TRANSFER))
            .filter_map(|log| Self::decode_transfer(log))
            .collect()
    }

    fn decode_v2_swap(log: &SimulationLog) -> Option<UniswapV2SwapEvent> {
        if log.topics.len() < 3 || log.data.len() < 128 {
            return None;
        }

        // Indexed: sender, to
        let sender = Address::from_slice(&log.topics[1].as_slice()[12..32]);
        let to = Address::from_slice(&log.topics[2].as_slice()[12..32]);

        // Non-indexed: amount0In, amount1In, amount0Out, amount1Out
        let amount0_in = U256::from_be_slice(&log.data[0..32]);
        let amount1_in = U256::from_be_slice(&log.data[32..64]);
        let amount0_out = U256::from_be_slice(&log.data[64..96]);
        let amount1_out = U256::from_be_slice(&log.data[96..128]);

        Some(UniswapV2SwapEvent {
            pool: log.address,
            sender,
            to,
            amount0_in,
            amount1_in,
            amount0_out,
            amount1_out,
        })
    }

    fn decode_v3_swap(log: &SimulationLog) -> Option<UniswapV3SwapEvent> {
        if log.topics.len() < 3 || log.data.len() < 160 {
            return None;
        }

        // Indexed: sender, recipient
        let sender = Address::from_slice(&log.topics[1].as_slice()[12..32]);
        let recipient = Address::from_slice(&log.topics[2].as_slice()[12..32]);

        // Non-indexed: amount0, amount1, sqrtPriceX96, liquidity, tick
        let amount0 = I256::from_be_bytes::<32>(log.data[0..32].try_into().ok()?);
        let amount1 = I256::from_be_bytes::<32>(log.data[32..64].try_into().ok()?);
        let sqrt_price_x96 = U256::from_be_slice(&log.data[64..96]);
        let liquidity = U256::from_be_slice(&log.data[96..128]).to::<u128>();
        let tick = I256::from_be_bytes::<32>(log.data[128..160].try_into().ok()?).as_i32();

        Some(UniswapV3SwapEvent {
            pool: log.address,
            sender,
            recipient,
            amount0,
            amount1,
            sqrt_price_x96,
            liquidity,
            tick,
        })
    }

    fn decode_transfer(log: &SimulationLog) -> Option<Erc20TransferEvent> {
        if log.topics.len() < 3 || log.data.len() < 32 {
            return None;
        }

        let from = Address::from_slice(&log.topics[1].as_slice()[12..32]);
        let to = Address::from_slice(&log.topics[2].as_slice()[12..32]);
        let amount = U256::from_be_slice(&log.data[0..32]);

        Some(Erc20TransferEvent {
            token: log.address,
            from,
            to,
            amount,
        })
    }
}

/// Decoded Uniswap V2 Swap event.
#[derive(Debug, Clone)]
pub struct UniswapV2SwapEvent {
    /// Pool address
    pub pool: Address,
    /// Sender address
    pub sender: Address,
    /// Recipient address
    pub to: Address,
    /// Amount of token0 input
    pub amount0_in: U256,
    /// Amount of token1 input
    pub amount1_in: U256,
    /// Amount of token0 output
    pub amount0_out: U256,
    /// Amount of token1 output
    pub amount1_out: U256,
}

impl UniswapV2SwapEvent {
    /// Returns true if this is a token0 -> token1 swap.
    #[must_use]
    pub fn is_zero_for_one(&self) -> bool {
        !self.amount0_in.is_zero() && !self.amount1_out.is_zero()
    }

    /// Returns the input amount.
    #[must_use]
    pub fn amount_in(&self) -> U256 {
        if !self.amount0_in.is_zero() {
            self.amount0_in
        } else {
            self.amount1_in
        }
    }

    /// Returns the output amount.
    #[must_use]
    pub fn amount_out(&self) -> U256 {
        if !self.amount0_out.is_zero() {
            self.amount0_out
        } else {
            self.amount1_out
        }
    }
}

/// Decoded Uniswap V3 Swap event.
#[derive(Debug, Clone)]
pub struct UniswapV3SwapEvent {
    /// Pool address
    pub pool: Address,
    /// Sender address
    pub sender: Address,
    /// Recipient address
    pub recipient: Address,
    /// Amount of token0 (positive = in, negative = out)
    pub amount0: I256,
    /// Amount of token1 (positive = in, negative = out)
    pub amount1: I256,
    /// New sqrt price after swap
    pub sqrt_price_x96: U256,
    /// New liquidity after swap
    pub liquidity: u128,
    /// New tick after swap
    pub tick: i32,
}

impl UniswapV3SwapEvent {
    /// Returns true if this is a token0 -> token1 swap.
    #[must_use]
    pub fn is_zero_for_one(&self) -> bool {
        self.amount0.is_positive() && self.amount1.is_negative()
    }

    /// Returns the input amount (always positive).
    #[must_use]
    pub fn amount_in(&self) -> U256 {
        if self.amount0.is_positive() {
            self.amount0.into_raw()
        } else {
            self.amount1.into_raw()
        }
    }

    /// Returns the output amount (always positive).
    #[must_use]
    pub fn amount_out(&self) -> U256 {
        if self.amount0.is_negative() {
            (-self.amount0).into_raw()
        } else {
            (-self.amount1).into_raw()
        }
    }
}

/// Decoded ERC20 Transfer event.
#[derive(Debug, Clone)]
pub struct Erc20TransferEvent {
    /// Token address
    pub token: Address,
    /// Sender address
    pub from: Address,
    /// Recipient address
    pub to: Address,
    /// Transfer amount
    pub amount: U256,
}

/// Calculates profit from simulation results.
#[derive(Debug)]
pub struct ProfitCalculator;

impl ProfitCalculator {
    /// Calculates net profit from a simulation, accounting for gas costs.
    ///
    /// # Arguments
    ///
    /// * `result` - The simulation result
    /// * `token_profits` - Map of token address to profit amount
    /// * `gas_price` - Gas price in wei
    /// * `token_prices` - Map of token address to ETH price (in wei per token)
    ///
    /// # Returns
    ///
    /// Net profit in wei (ETH)
    #[must_use]
    pub fn calculate_net_profit(
        result: &SimulationResult,
        token_profits: &[(Address, I256)],
        gas_price: U256,
        token_prices: &[(Address, U256)],
    ) -> I256 {
        // Calculate gas cost
        let gas_cost = gas_price * U256::from(result.gas_used);

        // Calculate token profits in ETH
        let mut total_profit = I256::ZERO;
        for (token, profit) in token_profits {
            // Find token price
            if let Some((_, price)) = token_prices.iter().find(|(t, _)| t == token) {
                // profit_in_eth = profit * price
                let profit_in_eth = *profit * I256::from_raw(*price);
                total_profit = total_profit + profit_in_eth;
            }
        }

        // Subtract gas cost
        total_profit - I256::from_raw(gas_cost)
    }

    /// Detects coinbase transfer (MEV bribe) from internal calls.
    #[must_use]
    pub fn detect_coinbase_bribe(
        result: &SimulationResult,
        coinbase: Address,
    ) -> Option<U256> {
        // Look for transfers to coinbase in logs
        let transfers = SwapAnalyzer::find_transfers(&result.logs);

        for transfer in transfers {
            if transfer.to == coinbase {
                return Some(transfer.amount);
            }
        }

        // Could also check internal calls if available
        for call in &result.calls {
            if call.to == coinbase && !call.value.is_zero() {
                return Some(call.value);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::types::SimulationLog;

    #[test]
    fn test_v2_swap_direction() {
        let swap = UniswapV2SwapEvent {
            pool: Address::ZERO,
            sender: Address::ZERO,
            to: Address::ZERO,
            amount0_in: U256::from(1000u64),
            amount1_in: U256::ZERO,
            amount0_out: U256::ZERO,
            amount1_out: U256::from(990u64),
        };

        assert!(swap.is_zero_for_one());
        assert_eq!(swap.amount_in(), U256::from(1000u64));
        assert_eq!(swap.amount_out(), U256::from(990u64));
    }
}
