# mev-lib

A comprehensive **MEV (Maximal Extractable Value)** utilities library for Ethereum, written in Rust.

[![Crates.io](https://img.shields.io/crates/v/mev-lib.svg)](https://crates.io/crates/mev-lib)
[![Documentation](https://docs.rs/mev-lib/badge.svg)](https://docs.rs/mev-lib)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

`mev-lib` provides essential building blocks for developing MEV bots and analysis tools on Ethereum and EVM-compatible chains. The library focuses on:

- **Type Safety**: Compile-time guarantees with `TokenId`, `PoolId`, `SwapId`
- **Performance**: Optimized AMM math calculations
- **Modularity**: Use only what you need
- **Education**: Well-documented code for learning

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              mev-lib                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   types/    │  │   math/     │  │  decoder/   │  │ simulation/ │       │
│  │             │  │             │  │             │  │             │       │
│  │ • TokenId   │  │ • V2 Math   │  │ • V2 Decode │  │ • Revm      │       │
│  │ • PoolId    │  │ • V3 Math   │  │ • V3 Decode │  │ • Fork      │       │
│  │ • Swap      │  │ • Curve     │  │ • ERC20     │  │ • Trace     │       │
│  │ • Direction │  │ • Balancer  │  │ • Multicall │  │ • Analyze   │       │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mev-lib = "0.1"

# With simulation support (adds revm dependency)
mev-lib = { version = "0.1", features = ["simulation"] }
```

## Quick Start

### AMM Math - Calculate Swap Outputs

```rust
use mev_lib::math::uniswap_v2;
use alloy_primitives::U256;

// Uniswap V2 constant product formula
let amount_in = U256::from(1_000_000u64);      // 1M tokens
let reserve_in = U256::from(100_000_000_000u64);  // 100B
let reserve_out = U256::from(100_000_000_000u64); // 100B

let amount_out = uniswap_v2::get_amount_out(
    amount_in,
    reserve_in,
    reserve_out,
    30,  // 0.3% fee in basis points
);

println!("Output: {} tokens", amount_out);
```

### Decode Swap Transactions

```rust
use mev_lib::decoder::{decode_calldata, DecodedCall};
use alloy_primitives::Bytes;

let calldata: Bytes = /* transaction input data */;

match decode_calldata(&calldata) {
    Some(DecodedCall::UniswapV2(swap)) => {
        println!("V2 Swap detected!");
        println!("  Path: {:?}", swap.path);
        println!("  Amount In: {:?}", swap.amount_in);
        println!("  Min Out: {:?}", swap.amount_out_min);
    }
    Some(DecodedCall::UniswapV3(swap)) => {
        println!("V3 Swap detected!");
        println!("  Fee tier: {} bps", swap.primary_fee().unwrap_or(0));
    }
    Some(DecodedCall::Transfer(transfer)) => {
        println!("ERC20 Transfer: {} to {}", transfer.amount, transfer.to);
    }
    _ => println!("Unknown transaction type"),
}
```

### Simulate Transactions Locally

```rust
use mev_lib::simulation::{SimulationDb, SimulationTx, SimulationConfig, simulate};
use alloy_primitives::{Address, Bytes, U256};

// Create simulation database
let mut db = SimulationDb::new();

// Fund the sender
let sender = Address::repeat_byte(0x01);
db.set_balance(sender, U256::from(10_000_000_000_000_000_000u128)); // 10 ETH

// Create transaction
let tx = SimulationTx::new(sender, recipient, calldata)
    .with_value(U256::ZERO)
    .with_gas(500_000);

// Configure simulation
let config = SimulationConfig::new()
    .with_block_number(18_000_000)
    .with_call_tracing();

// Execute
let result = simulate(&tx, db, &config)?;

if result.is_success() {
    println!("Gas used: {}", result.gas_used);
    println!("Logs emitted: {}", result.logs.len());
} else {
    println!("Reverted: {:?}", result.revert_reason());
}
```

## Modules

### `types/` - Core Primitives

Type-safe abstractions for DeFi concepts:

| Type | Description |
|------|-------------|
| `TokenId` | Wrapper around `Address` for tokens |
| `PoolId` | Wrapper around `Address` for liquidity pools |
| `Pool` | Pool with token pair and reserves |
| `Swap` | Swap operation with direction |
| `Direction` | `ZeroForOne` or `OneForZero` |

### `math/` - AMM Formulas

Precise math implementations for DEX protocols:

| Function | Description |
|----------|-------------|
| `get_amount_out` | Calculate output for given input |
| `get_amount_in` | Calculate required input for desired output |
| `spot_price` | Get current price without slippage |
| `price_impact` | Calculate price impact percentage |
| `simulate_swap` | Get new reserves after swap |

### `decoder/` - Calldata Parsing

Decode transaction input data:

| Protocol | Methods |
|----------|---------|
| Uniswap V2 | All swap variants (9 methods) |
| Uniswap V3 | exactInput, exactOutput, etc. |
| ERC20 | transfer, transferFrom, approve |

### `simulation/` - Local Execution

Simulate transactions using Revm:

| Feature | Description |
|---------|-------------|
| `simulate()` | Execute single transaction |
| `simulate_bundle()` | Execute multiple transactions in sequence |
| `SimulationDb` | In-memory state database |
| `StateOverride` | Modify state before simulation |

## How MEV Works

### Transaction Flow

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  User    │────▶│ Mempool  │────▶│  Block   │────▶│  Chain   │
│  Submits │     │ (Public) │     │ Builder  │     │  State   │
│    Tx    │     │          │     │          │     │          │
└──────────┘     └──────────┘     └──────────┘     └──────────┘
                      │
                      ▼
              ┌──────────────┐
              │   MEV Bot    │
              │              │
              │ • Monitor    │
              │ • Decode     │
              │ • Simulate   │
              │ • Extract    │
              └──────────────┘
```

### Common MEV Strategies

1. **Arbitrage**: Exploit price differences across DEXs
2. **Sandwich**: Front-run and back-run victim swaps
3. **Liquidation**: Liquidate undercollateralized positions
4. **Backrun**: Execute after specific transactions

## MEV Detection Pipeline

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        MEV Detection Pipeline                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐            │
│   │ Mempool │───▶│ Decode  │───▶│Simulate │───▶│ Profit  │            │
│   │ Monitor │    │ Calldata│    │  (Revm) │    │  Calc   │            │
│   └─────────┘    └─────────┘    └─────────┘    └─────────┘            │
│                                                                         │
│   Example Flow:                                                         │
│                                                                         │
│   1. Pending tx arrives: 0x38ed1739...                                 │
│   2. Decode: swapExactTokensForTokens(1000 USDC → ETH)                │
│   3. Simulate locally with Revm                                        │
│   4. Check profitability after gas                                     │
│   5. Submit bundle if profitable                                       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Uniswap V2 Math

The constant product formula: `x * y = k`

```
                    Amount Out Calculation
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│   amount_out = (amount_in × fee × reserve_out)                         │
│                ─────────────────────────────────                        │
│                (reserve_in × 1000) + (amount_in × fee)                 │
│                                                                         │
│   where fee = 997 (0.3% fee means 99.7% passes through)                │
│                                                                         │
│   Example:                                                              │
│   • amount_in = 1,000,000                                              │
│   • reserve_in = 1,000,000,000                                         │
│   • reserve_out = 1,000,000,000                                        │
│   • fee = 997                                                          │
│                                                                         │
│   amount_out = (1,000,000 × 997 × 1,000,000,000)                       │
│                ────────────────────────────────────                     │
│                (1,000,000,000 × 1000) + (1,000,000 × 997)              │
│                                                                         │
│              = 996,005 tokens (with slippage)                          │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Uniswap V3 Path Encoding

V3 uses packed path encoding:

```
V3 Path Format
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│   Single Hop:                                                           │
│   ┌──────────────┬─────────┬──────────────┐                            │
│   │   Token A    │   Fee   │   Token B    │                            │
│   │  (20 bytes)  │(3 bytes)│  (20 bytes)  │                            │
│   └──────────────┴─────────┴──────────────┘                            │
│                                                                         │
│   Multi Hop (A → B → C):                                               │
│   ┌──────────┬─────┬──────────┬─────┬──────────┐                       │
│   │ Token A  │Fee 1│ Token B  │Fee 2│ Token C  │                       │
│   │(20 bytes)│ (3) │(20 bytes)│ (3) │(20 bytes)│                       │
│   └──────────┴─────┴──────────┴─────┴──────────┘                       │
│                                                                         │
│   Fee Tiers (in hundredths of a bip):                                  │
│   • 100   = 0.01%  (stablecoin pairs)                                  │
│   • 500   = 0.05%  (stable pairs)                                      │
│   • 3000  = 0.30%  (most pairs)                                        │
│   • 10000 = 1.00%  (exotic pairs)                                      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Revm Simulation

Local transaction simulation without broadcasting:

```
Simulation Architecture
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│  ┌─────────────┐                                                       │
│  │    RPC      │──── Fork State ────┐                                  │
│  │  (Mainnet)  │                    │                                  │
│  └─────────────┘                    ▼                                  │
│                           ┌─────────────────┐                          │
│                           │  SimulationDb   │                          │
│                           │                 │                          │
│                           │ • Accounts      │                          │
│                           │ • Storage       │                          │
│                           │ • Code          │                          │
│                           └────────┬────────┘                          │
│                                    │                                   │
│                                    ▼                                   │
│  ┌─────────────┐          ┌─────────────────┐                          │
│  │ Transaction │─────────▶│      Revm       │                          │
│  │             │          │                 │                          │
│  │ • from      │          │ • Execute       │                          │
│  │ • to        │          │ • No commit     │                          │
│  │ • calldata  │          │ • Return result │                          │
│  │ • value     │          │                 │                          │
│  └─────────────┘          └────────┬────────┘                          │
│                                    │                                   │
│                                    ▼                                   │
│                         ┌─────────────────────┐                        │
│                         │  SimulationResult   │                        │
│                         │                     │                        │
│                         │ • gas_used          │                        │
│                         │ • logs              │                        │
│                         │ • output            │                        │
│                         │ • status            │                        │
│                         └─────────────────────┘                        │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Feature Flags

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `default` | Core types, math, decoder | alloy-primitives |
| `simulation` | Revm-based simulation | revm |
| `serde` | Serialization support | serde |

## Examples

See the `examples/` directory for complete examples:

- `simple_arb.rs` - Basic arbitrage detection
- `decode_tx.rs` - Transaction decoding
- `simulate.rs` - Local simulation

## Roadmap

- [x] Phase 1: Core types and Uniswap V2 math
- [x] Phase 2: Decoder module (V2, V3, ERC20)
- [x] Phase 3: Simulation module (Revm)
- [ ] Phase 4: Arbitrage detection (graph-based)
- [ ] Phase 5: Bundle building (Flashbots)
- [ ] Phase 6: Mempool monitoring

## References

### Libraries & Tools

- [Revm](https://github.com/bluealloy/revm) - Rust EVM implementation
- [Alloy](https://github.com/alloy-rs/alloy) - Ethereum library
- [Artemis](https://github.com/paradigmxyz/artemis) - MEV bot framework
- [Reth](https://github.com/paradigmxyz/reth) - Rust Ethereum client

### Learning Resources

- [Flashbots Docs](https://docs.flashbots.net/)
- [MEV Wiki](https://github.com/flashbots/mev-research)
- [Uniswap V2 Whitepaper](https://uniswap.org/whitepaper.pdf)
- [Uniswap V3 Whitepaper](https://uniswap.org/whitepaper-v3.pdf)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please read the contributing guidelines first.
