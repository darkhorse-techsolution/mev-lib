# Building an MEV Library in Rust: A Complete Guide

*A comprehensive educational series for understanding and building MEV tools*

---

## Table of Contents

1. [Understanding MEV and Why Rust](#article-1-understanding-mev-and-why-rust)
2. [Decoding Ethereum Transactions](#article-2-decoding-ethereum-transactions)
3. [AMM Math Deep Dive](#article-3-amm-math-deep-dive)
4. [Local Transaction Simulation](#article-4-local-transaction-simulation)
5. [Arbitrage Detection](#article-5-arbitrage-detection)
6. [Building a Complete MEV Bot](#article-6-building-a-complete-mev-bot)

---

# Article 1: Understanding MEV and Why Rust?

## What is MEV?

**MEV (Maximal Extractable Value)** represents the maximum value that can be extracted from block production beyond the standard block reward and gas fees. This value comes from the ability to:

- **Include** specific transactions
- **Exclude** specific transactions
- **Reorder** transactions within a block

### Historical Context

```
Timeline of MEV
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  2019          2020           2021           2022           2023+          │
│    │             │              │              │              │             │
│    ▼             ▼              ▼              ▼              ▼             │
│ ┌─────┐     ┌─────────┐   ┌──────────┐  ┌──────────┐  ┌────────────┐      │
│ │Flash│     │Flashbots│   │MEV-Boost │  │  The     │  │ MEV-Share  │      │
│ │Loans│     │ Launch  │   │ Launch   │  │ Merge    │  │ & OFAs     │      │
│ │Begin│     │         │   │          │  │ (PoS)    │  │            │      │
│ └─────┘     └─────────┘   └──────────┘  └──────────┘  └────────────┘      │
│                                                                             │
│  "Miner              Private         Proposer-      "Maximal         User  │
│   Extractable        Mempools        Builder        Extractable     Protect│
│   Value" coined      Emerge          Separation     Value"          -ion   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## The MEV Ecosystem

### Key Players

```
MEV Ecosystem Participants
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  ┌───────────────┐                                                         │
│  │    USERS      │  Submit transactions to swap, lend, borrow              │
│  └───────┬───────┘                                                         │
│          │                                                                  │
│          ▼                                                                  │
│  ┌───────────────┐     ┌───────────────┐                                   │
│  │   MEMPOOL     │────▶│  SEARCHERS    │  Monitor mempool for MEV          │
│  │  (Public)     │     │  (MEV Bots)   │  opportunities                    │
│  └───────────────┘     └───────┬───────┘                                   │
│                                │                                            │
│                                ▼                                            │
│                        ┌───────────────┐                                   │
│                        │   BUILDERS    │  Construct optimal blocks         │
│                        │               │  from transactions + bundles      │
│                        └───────┬───────┘                                   │
│                                │                                            │
│                                ▼                                            │
│                        ┌───────────────┐                                   │
│                        │  VALIDATORS   │  Propose blocks to the network    │
│                        │  (Proposers)  │  Receive MEV rewards              │
│                        └───────┬───────┘                                   │
│                                │                                            │
│                                ▼                                            │
│                        ┌───────────────┐                                   │
│                        │  BLOCKCHAIN   │  Final, immutable state           │
│                        └───────────────┘                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Types of MEV Opportunities

### 1. Arbitrage

**What**: Exploit price differences for the same asset across different venues.

**Example**: ETH costs $2000 on Uniswap but $2010 on SushiSwap
- Buy 1 ETH on Uniswap for $2000
- Sell 1 ETH on SushiSwap for $2010
- Profit: $10 (minus gas)

```
Arbitrage Flow
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ┌─────────┐         ┌─────────┐         ┌─────────┐                      │
│   │  Start  │         │  DEX A  │         │  DEX B  │                      │
│   │  100    │   Buy   │  Price: │  Sell   │  Price: │                      │
│   │  USDC   │ ──────▶ │  $2000  │ ──────▶ │  $2010  │                      │
│   └─────────┘         │  /ETH   │         │  /ETH   │                      │
│                       └─────────┘         └────┬────┘                      │
│                                                │                            │
│                                                ▼                            │
│                                          ┌─────────┐                       │
│                                          │  End    │                       │
│                                          │  100.50 │  Profit: $0.50        │
│                                          │  USDC   │  (after gas)          │
│                                          └─────────┘                       │
│                                                                             │
│   Atomic Execution: Both swaps in single transaction = no risk!            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2. Sandwich Attack

**What**: Front-run and back-run a victim's large swap to profit from price movement.

**Why it works**: Large swaps move prices. If you know a large swap is coming, you can:
1. Buy before (front-run) - push price up
2. Let victim buy at higher price
3. Sell after (back-run) - profit from the price increase

```
Sandwich Attack Anatomy
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  Block N                                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  Tx 1: ATTACKER FRONT-RUN                                          │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │  Buy 10 ETH @ $2000                                         │   │   │
│  │  │  Price moves: $2000 → $2005                                 │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  │                              │                                      │   │
│  │                              ▼                                      │   │
│  │  Tx 2: VICTIM SWAP                                                 │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │  Buy 100 ETH @ $2005 (expected $2000!)                      │   │   │
│  │  │  Price moves: $2005 → $2050                                 │   │   │
│  │  │  Victim loses: ~$500 to slippage                            │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  │                              │                                      │   │
│  │                              ▼                                      │   │
│  │  Tx 3: ATTACKER BACK-RUN                                           │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │  Sell 10 ETH @ $2050                                        │   │   │
│  │  │  Profit: 10 × ($2050 - $2000) = $500                        │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Attacker Profit: ~$500 (minus gas)                                        │
│  Victim Loss: ~$500 (worse execution price)                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3. Liquidation

**What**: Liquidate undercollateralized positions on lending protocols.

**How it works**:
1. User borrows $800 against $1000 collateral (80% LTV)
2. Collateral price drops, now worth $900
3. Position becomes liquidatable (>80% LTV)
4. Liquidator repays part of debt, receives collateral + bonus

```
Liquidation Example
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  BEFORE (Healthy Position)              AFTER (Liquidatable)               │
│  ┌─────────────────────────┐           ┌─────────────────────────┐         │
│  │ Collateral: 1 ETH       │           │ Collateral: 1 ETH       │         │
│  │ Value: $1000            │   Price   │ Value: $900             │         │
│  │ Debt: $800 USDC         │   Drops   │ Debt: $800 USDC         │         │
│  │ LTV: 80% ✓              │ ────────▶ │ LTV: 88.9% ✗            │         │
│  │ Health: Good            │           │ Health: LIQUIDATABLE    │         │
│  └─────────────────────────┘           └─────────────────────────┘         │
│                                                                             │
│  LIQUIDATION PROCESS:                                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  Liquidator pays: $400 USDC (50% of debt)                          │   │
│  │  Liquidator receives: $400 + 5% bonus = $420 worth of ETH          │   │
│  │  Liquidator profit: $20                                            │   │
│  │                                                                     │   │
│  │  Remaining position:                                                │   │
│  │  Collateral: ~0.53 ETH ($480)                                      │   │
│  │  Debt: $400 USDC                                                   │   │
│  │  LTV: 83% (still risky but better)                                 │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4. Just-In-Time (JIT) Liquidity

**What**: Provide liquidity right before a large swap, capture fees, remove liquidity after.

**Advanced technique** used in Uniswap V3 with concentrated liquidity.

## Why Rust for MEV?

### Performance Comparison

```
Language Performance for MEV (Relative)
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  Task: Simulate 1000 swaps                                                 │
│                                                                             │
│  Rust      ████████████████████████████████████████  100ms                 │
│  Go        ████████████████████████████████████████████████  180ms         │
│  Python    ████████████████████████████████████████████████████████  800ms │
│  JavaScript████████████████████████████████████████████████████  600ms     │
│                                                                             │
│  Task: Decode 10,000 transactions                                          │
│                                                                             │
│  Rust      ██████████████████████████  50ms                                │
│  Go        ████████████████████████████████████  90ms                      │
│  Python    ████████████████████████████████████████████████████████  500ms │
│                                                                             │
│  WHY RUST WINS:                                                            │
│  • Zero-cost abstractions                                                  │
│  • No garbage collection pauses                                            │
│  • Predictable performance                                                 │
│  • Memory safety without runtime overhead                                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### The Rust MEV Ecosystem

| Project | Description | Stars |
|---------|-------------|-------|
| **Reth** | Rust Ethereum client | 4k+ |
| **Revm** | Rust EVM implementation | 2k+ |
| **Alloy** | Ethereum library | 1k+ |
| **Artemis** | MEV bot framework | 2k+ |
| **Foundry** | Development toolkit | 8k+ |

### Type Safety Prevents Costly Bugs

```rust
// Rust prevents mixing up token addresses at compile time!

struct TokenId(Address);  // Wrapper type for tokens
struct PoolId(Address);   // Wrapper type for pools

fn swap(pool: PoolId, token_in: TokenId, amount: U256) {
    // Can't accidentally pass a TokenId where PoolId is expected
}

// This won't compile:
// swap(token_address, pool_address, amount);  // ERROR!

// This will compile:
swap(pool_id, token_id, amount);  // Correct!
```

---

# Article 2: Decoding Ethereum Transactions

## Understanding Transaction Anatomy

Every Ethereum transaction has these fields:

```
Ethereum Transaction Structure
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  Transaction {                                                              │
│    nonce:      12,                        // Sender's tx count             │
│    gas_price:  20_000_000_000,            // 20 gwei                       │
│    gas_limit:  200_000,                   // Max gas to use                │
│    to:         0x7a250d...Router,         // Uniswap Router                │
│    value:      1_000_000_000_000_000_000, // 1 ETH                         │
│    data:       0x7ff36ab5...              // THE CALLDATA (our focus!)     │
│    v, r, s:    ...                        // Signature                     │
│  }                                                                          │
│                                                                             │
│  The `data` field contains:                                                │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  [4 bytes]        [remaining bytes]                                 │   │
│  │  Function         ABI-encoded parameters                            │   │
│  │  Selector                                                           │   │
│  │                                                                     │   │
│  │  0x7ff36ab5       0000000000000000000000000000000000...             │   │
│  │  ▲                ▲                                                 │   │
│  │  │                │                                                 │   │
│  │  keccak256(       Parameters packed in 32-byte slots                │   │
│  │  "swapExact       according to ABI specification                    │   │
│  │  ETHForTokens                                                       │   │
│  │  (...)")[:4]                                                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Function Selectors Explained

The selector is computed from the function signature:

```
Selector Computation
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  Function: swapExactTokensForTokens(                                       │
│              uint256 amountIn,                                             │
│              uint256 amountOutMin,                                         │
│              address[] path,                                               │
│              address to,                                                   │
│              uint256 deadline                                              │
│            )                                                               │
│                                                                             │
│  Step 1: Create canonical signature (no spaces, no param names)            │
│  ────────────────────────────────────────────────────────────              │
│  "swapExactTokensForTokens(uint256,uint256,address[],address,uint256)"     │
│                                                                             │
│  Step 2: Compute keccak256 hash                                            │
│  ────────────────────────────────────────────────────────────              │
│  keccak256("swapExact...") =                                               │
│  0x38ed1739274d7c5d4a6e37d95e8d2b7e3a1b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f... │
│                                                                             │
│  Step 3: Take first 4 bytes                                                │
│  ────────────────────────────────────────────────────────────              │
│  Selector = 0x38ed1739                                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## ABI Encoding Deep Dive

### Static vs Dynamic Types

```
ABI Encoding Rules
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  STATIC TYPES (fixed size, encoded in-place):                              │
│  ─────────────────────────────────────────────                              │
│  • uint256, int256     → 32 bytes                                          │
│  • address             → 32 bytes (20 bytes right-padded)                  │
│  • bool                → 32 bytes (0 or 1)                                 │
│  • bytes32             → 32 bytes                                          │
│                                                                             │
│  DYNAMIC TYPES (variable size, use offset pointer):                        │
│  ─────────────────────────────────────────────                              │
│  • bytes               → offset + length + data                            │
│  • string              → offset + length + data                            │
│  • T[] (dynamic array) → offset + length + elements                        │
│                                                                             │
│  EXAMPLE: swapExactTokensForTokens(1000, 900, [WETH, USDC], 0xABC, 9999)   │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                             │
│  Offset  │ Content                │ Description                            │
│  ────────┼────────────────────────┼───────────────────────────────────────  │
│  0x00    │ 0x38ed1739             │ Selector                               │
│  0x04    │ 0x00...03e8            │ amountIn = 1000                        │
│  0x24    │ 0x00...0384            │ amountOutMin = 900                     │
│  0x44    │ 0x00...00a0            │ path offset = 160 (0xa0)               │
│  0x64    │ 0x00...0ABC            │ to = 0xABC...                          │
│  0x84    │ 0x00...270f            │ deadline = 9999                        │
│  0xa4    │ 0x00...0002            │ path.length = 2                        │
│  0xc4    │ 0x00...WETH            │ path[0] = WETH address                 │
│  0xe4    │ 0x00...USDC            │ path[1] = USDC address                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Uniswap V2 Router Methods

All 9 swap methods you need to decode:

```
Uniswap V2 Swap Methods
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  EXACT INPUT (you specify input amount):                                   │
│  ─────────────────────────────────────────────                              │
│  ┌──────────────────────────────────────┬────────────┬──────────────────┐  │
│  │ Method                               │ Selector   │ Input → Output   │  │
│  ├──────────────────────────────────────┼────────────┼──────────────────┤  │
│  │ swapExactTokensForTokens             │ 0x38ed1739 │ Token → Token    │  │
│  │ swapExactETHForTokens                │ 0x7ff36ab5 │ ETH → Token      │  │
│  │ swapExactTokensForETH                │ 0x18cbafe5 │ Token → ETH      │  │
│  └──────────────────────────────────────┴────────────┴──────────────────┘  │
│                                                                             │
│  EXACT OUTPUT (you specify output amount):                                 │
│  ─────────────────────────────────────────────                              │
│  ┌──────────────────────────────────────┬────────────┬──────────────────┐  │
│  │ Method                               │ Selector   │ Input → Output   │  │
│  ├──────────────────────────────────────┼────────────┼──────────────────┤  │
│  │ swapTokensForExactTokens             │ 0x8803dbee │ Token → Token    │  │
│  │ swapETHForExactTokens                │ 0xfb3bdb41 │ ETH → Token      │  │
│  │ swapTokensForExactETH                │ 0x4a25d94a │ Token → ETH      │  │
│  └──────────────────────────────────────┴────────────┴──────────────────┘  │
│                                                                             │
│  FEE-ON-TRANSFER SUPPORT (for tokens that take fees):                      │
│  ─────────────────────────────────────────────                              │
│  ┌──────────────────────────────────────────────────────┬────────────┐     │
│  │ Method                                               │ Selector   │     │
│  ├──────────────────────────────────────────────────────┼────────────┤     │
│  │ swapExactTokensForTokensSupportingFeeOnTransfer...   │ 0x5c11d795 │     │
│  │ swapExactETHForTokensSupportingFeeOnTransfer...      │ 0xb6f9de95 │     │
│  │ swapExactTokensForETHSupportingFeeOnTransfer...      │ 0x791ac947 │     │
│  └──────────────────────────────────────────────────────┴────────────┘     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Uniswap V3 Path Encoding

V3 uses a completely different encoding for swap paths:

```
V3 Path Encoding
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  V2 Path: address[] = [tokenA, tokenB, tokenC]                             │
│  • Simple array of addresses                                               │
│  • Fee is always 0.3%                                                      │
│                                                                             │
│  V3 Path: bytes = tokenA + fee1 + tokenB + fee2 + tokenC                   │
│  • Packed encoding with fees between tokens                                │
│  • Multiple fee tiers possible                                             │
│                                                                             │
│  ENCODING STRUCTURE:                                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  Single Hop: WETH → USDC (0.3% fee)                                │   │
│  │  ┌────────────────────┬───────────┬────────────────────┐           │   │
│  │  │  WETH Address      │   3000    │  USDC Address      │           │   │
│  │  │  (20 bytes)        │ (3 bytes) │  (20 bytes)        │           │   │
│  │  │  0xC02a...         │  0x0BB8   │  0xA0b8...         │           │   │
│  │  └────────────────────┴───────────┴────────────────────┘           │   │
│  │  Total: 43 bytes                                                   │   │
│  │                                                                     │   │
│  │  Multi Hop: WETH → USDC (0.05%) → DAI (0.01%)                      │   │
│  │  ┌────────────┬───────┬────────────┬───────┬────────────┐          │   │
│  │  │   WETH     │  500  │   USDC     │  100  │   DAI      │          │   │
│  │  │  (20 B)    │ (3 B) │  (20 B)    │ (3 B) │  (20 B)    │          │   │
│  │  └────────────┴───────┴────────────┴───────┴────────────┘          │   │
│  │  Total: 66 bytes                                                   │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  FEE TIERS (in hundredths of a basis point):                               │
│  • 100   = 0.01% (stablecoin pairs like USDC/USDT)                         │
│  • 500   = 0.05% (stable pairs like ETH/stETH)                             │
│  • 3000  = 0.30% (most common, standard pairs)                             │
│  • 10000 = 1.00% (exotic/low liquidity pairs)                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Decoding Code Example

```rust
use mev_lib::decoder::{decode_calldata, DecodedCall};
use alloy_primitives::Bytes;

/// Analyze a pending transaction from the mempool
fn analyze_pending_tx(calldata: &Bytes, value: U256) -> Option<SwapInfo> {
    let decoded = decode_calldata(calldata)?;

    match decoded {
        DecodedCall::UniswapV2(swap) => {
            // Extract key MEV-relevant information
            let token_in = swap.token_in()?;
            let token_out = swap.token_out()?;
            let amount_in = swap.amount_in.or(Some(value))?; // ETH swaps use msg.value
            let min_out = swap.amount_out_min?;

            // Calculate slippage tolerance (MEV opportunity indicator!)
            // High slippage = potential sandwich target
            let slippage_bps = calculate_slippage(amount_in, min_out);

            println!("V2 Swap Detected:");
            println!("  {} → {}", token_in, token_out);
            println!("  Amount: {} (slippage tolerance: {} bps)", amount_in, slippage_bps);

            Some(SwapInfo {
                protocol: Protocol::UniswapV2,
                token_in,
                token_out,
                amount_in,
                min_amount_out: min_out,
                slippage_bps,
            })
        }

        DecodedCall::UniswapV3(swap) => {
            println!("V3 Swap Detected:");
            println!("  {} → {}", swap.token_in, swap.token_out);
            println!("  Fee tier: {} bps", swap.primary_fee().unwrap_or(0));

            // V3 swaps often have tighter slippage - less MEV opportunity
            Some(SwapInfo {
                protocol: Protocol::UniswapV3,
                token_in: swap.token_in,
                token_out: swap.token_out,
                amount_in: swap.amount_in?,
                min_amount_out: swap.amount_out_min?,
                slippage_bps: calculate_slippage(swap.amount_in?, swap.amount_out_min?),
            })
        }

        _ => None, // Not a swap we care about
    }
}
```

---

# Article 3: AMM Math Deep Dive

## The Constant Product Formula

Uniswap V2's core invariant: **x × y = k**

```
Understanding x * y = k
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  A liquidity pool with two tokens:                                         │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     LIQUIDITY POOL                                  │   │
│  │  ┌─────────────────────────┬─────────────────────────┐              │   │
│  │  │       Token X           │       Token Y           │              │   │
│  │  │    (e.g., WETH)         │    (e.g., USDC)         │              │   │
│  │  │                         │                         │              │   │
│  │  │    Reserve: 100 ETH     │    Reserve: 200,000 USDC│              │   │
│  │  │                         │                         │              │   │
│  │  └─────────────────────────┴─────────────────────────┘              │   │
│  │                                                                     │   │
│  │  Invariant: x × y = k                                               │   │
│  │  100 × 200,000 = 20,000,000                                         │   │
│  │                                                                     │   │
│  │  Implied Price: 200,000 / 100 = $2,000 per ETH                      │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  WHAT HAPPENS WHEN YOU SWAP:                                               │
│                                                                             │
│  You want to buy ETH with 2,000 USDC:                                      │
│                                                                             │
│  Before: x=100, y=200,000, k=20,000,000                                    │
│  After:  y'=202,000, x'=k/y'=20,000,000/202,000 = 99.0099                  │
│  You receive: 100 - 99.0099 = 0.9901 ETH                                   │
│                                                                             │
│  Expected at spot price: 2000/2000 = 1 ETH                                 │
│  Actual received: 0.9901 ETH                                               │
│  Slippage: ~1%                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## The Swap Formula with Fees

```
Uniswap V2 Swap Formula
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  Given:                                                                     │
│  • amountIn    = tokens you're selling                                     │
│  • reserveIn   = pool reserve of input token                               │
│  • reserveOut  = pool reserve of output token                              │
│  • fee         = 0.3% (represented as 997/1000)                            │
│                                                                             │
│  Formula:                                                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │              amountIn × 997 × reserveOut                            │   │
│  │  amountOut = ─────────────────────────────────                      │   │
│  │              reserveIn × 1000 + amountIn × 997                      │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Why 997? Because 0.3% fee means 99.7% passes through.                     │
│  997/1000 = 0.997 = 99.7%                                                  │
│                                                                             │
│  STEP BY STEP EXAMPLE:                                                      │
│  ─────────────────────                                                      │
│  Swap 1,000 USDC for ETH                                                   │
│  Pool: 100 ETH / 200,000 USDC                                              │
│                                                                             │
│  amountIn = 1,000                                                          │
│  reserveIn = 200,000                                                       │
│  reserveOut = 100                                                          │
│                                                                             │
│  numerator = 1,000 × 997 × 100 = 99,700,000                                │
│  denominator = 200,000 × 1000 + 1,000 × 997 = 200,997,000                  │
│                                                                             │
│  amountOut = 99,700,000 / 200,997,000 = 0.4960 ETH                         │
│                                                                             │
│  At spot price ($2000/ETH): would get 0.5 ETH                              │
│  Actual: 0.4960 ETH                                                        │
│  Loss to slippage + fees: 0.8%                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Price Impact Visualization

```
How Trade Size Affects Price
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  Pool: 1000 ETH / 2,000,000 USDC ($2000/ETH)                               │
│                                                                             │
│  Trade Size    │  ETH Received  │  Effective Price  │  Price Impact        │
│  ──────────────┼────────────────┼───────────────────┼──────────────────    │
│  $1,000        │  0.4995 ETH    │  $2,002           │  0.1%                │
│  $10,000       │  4.9751 ETH    │  $2,010           │  0.5%                │
│  $100,000      │  47.619 ETH    │  $2,100           │  5.0%                │
│  $500,000      │  200.00 ETH    │  $2,500           │  25.0%               │
│  $1,000,000    │  333.33 ETH    │  $3,000           │  50.0%               │
│                                                                             │
│                                                                             │
│  Price Impact Graph:                                                        │
│                                                                             │
│  Impact │                                                                  │
│   50% ──┤                                              ●                   │
│         │                                                                  │
│   40% ──┤                                                                  │
│         │                                                                  │
│   30% ──┤                                                                  │
│         │                                    ●                             │
│   20% ──┤                                                                  │
│         │                                                                  │
│   10% ──┤                          ●                                       │
│         │                                                                  │
│    5% ──┤                 ●                                                │
│         │        ●                                                         │
│    0% ──┼────●───────────────────────────────────────────▶ Trade Size     │
│         $1K    $10K     $100K    $500K   $1M                               │
│                                                                             │
│  KEY INSIGHT: Price impact grows FASTER than trade size!                   │
│  This is why large trades are sandwich targets.                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Implementing the Math in Rust

```rust
use alloy_primitives::U256;

/// Basis points denominator (10000 = 100%)
const BPS_DENOMINATOR: u64 = 10_000;

/// Calculate output amount for a Uniswap V2 swap
///
/// # Formula
/// amount_out = (amount_in × (10000 - fee_bps) × reserve_out)
///            / (reserve_in × 10000 + amount_in × (10000 - fee_bps))
pub fn get_amount_out(
    amount_in: U256,
    reserve_in: U256,
    reserve_out: U256,
    fee_bps: u32,  // 30 = 0.3%
) -> U256 {
    // Sanity checks
    assert!(!reserve_in.is_zero(), "reserve_in must be non-zero");
    assert!(!reserve_out.is_zero(), "reserve_out must be non-zero");

    if amount_in.is_zero() {
        return U256::ZERO;
    }

    // Calculate fee multiplier (e.g., 30 bps fee → 9970 multiplier)
    let fee_multiplier = U256::from(BPS_DENOMINATOR - u64::from(fee_bps));
    let denominator_multiplier = U256::from(BPS_DENOMINATOR);

    // Apply formula
    let amount_in_with_fee = amount_in * fee_multiplier;
    let numerator = amount_in_with_fee * reserve_out;
    let denominator = (reserve_in * denominator_multiplier) + amount_in_with_fee;

    numerator / denominator
}

/// Calculate the price impact of a trade as a percentage
pub fn price_impact(
    amount_in: U256,
    reserve_in: U256,
    reserve_out: U256,
    fee_bps: u32,
) -> f64 {
    // Spot price (no trade)
    let spot = reserve_out.to::<f64>() / reserve_in.to::<f64>();

    // Execution price (with trade)
    let amount_out = get_amount_out(amount_in, reserve_in, reserve_out, fee_bps);
    let execution = amount_out.to::<f64>() / amount_in.to::<f64>();

    // Adjust for fee
    let fee_adjusted_spot = spot * (1.0 - f64::from(fee_bps) / 10000.0);

    // Impact = how much worse execution is vs fee-adjusted spot
    1.0 - (execution / fee_adjusted_spot)
}
```

---

# Article 4: Local Transaction Simulation

## Why Local Simulation is Essential

```
Benefits of Local Simulation
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  WITHOUT SIMULATION:                      WITH SIMULATION:                  │
│  ────────────────────                     ──────────────────                │
│                                                                             │
│  • Send tx to mempool                     • Test locally first             │
│  • Wait for inclusion (~12s)              • Instant results (<100ms)       │
│  • Pay gas even if tx fails               • Zero cost to test              │
│  • Reveal strategy to competitors         • Complete privacy               │
│  • Hope it works                          • Know exactly what happens      │
│                                                                             │
│  SIMULATION ENABLES:                                                        │
│  ────────────────────                                                       │
│                                                                             │
│  1. PROFIT CALCULATION                                                      │
│     Execute your arbitrage locally, verify profit > gas cost               │
│                                                                             │
│  2. SANDWICH ANALYSIS                                                       │
│     Simulate: front-run → victim → back-run                                │
│     Calculate exact profit before committing                               │
│                                                                             │
│  3. REVERT DETECTION                                                        │
│     Know if tx will fail BEFORE paying gas                                 │
│     Understand WHY it fails (decode revert reason)                         │
│                                                                             │
│  4. GAS ESTIMATION                                                          │
│     Accurate gas usage, not estimates                                      │
│     Optimize calldata for gas efficiency                                   │
│                                                                             │
│  5. STATE INSPECTION                                                        │
│     See all storage changes                                                │
│     Track token balances                                                   │
│     Analyze internal calls                                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Revm Architecture

```
Revm Internal Structure
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                              ┌─────────────────┐                           │
│                              │      Evm        │                           │
│                              │    (Entry)      │                           │
│                              └────────┬────────┘                           │
│                                       │                                     │
│          ┌────────────────────────────┼────────────────────────────┐       │
│          │                            │                            │       │
│          ▼                            ▼                            ▼       │
│  ┌───────────────┐          ┌───────────────┐          ┌───────────────┐  │
│  │    Context    │          │  Interpreter  │          │   Database    │  │
│  │               │          │               │          │               │  │
│  │ • Block env   │          │ • Execute     │          │ • Accounts    │  │
│  │ • Tx env      │          │   opcodes     │          │ • Storage     │  │
│  │ • Cfg env     │          │ • Gas meter   │          │ • Bytecode    │  │
│  │               │          │ • Stack/Mem   │          │               │  │
│  └───────────────┘          └───────────────┘          └───────────────┘  │
│                                                                             │
│  EXECUTION FLOW:                                                            │
│  ───────────────                                                            │
│                                                                             │
│  1. Load sender account from Database                                       │
│  2. Check balance >= value + gas_limit × gas_price                         │
│  3. Increment sender nonce                                                  │
│  4. Transfer value to recipient                                             │
│  5. If contract call:                                                       │
│     a. Load contract bytecode                                              │
│     b. Create Interpreter                                                  │
│     c. Execute opcodes one by one                                          │
│     d. Handle internal calls (CALL, DELEGATECALL, etc.)                    │
│  6. Calculate gas used, apply refunds                                       │
│  7. Return result + logs + state changes                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Simulation Code Deep Dive

```rust
use mev_lib::simulation::{
    SimulationDb, SimulationTx, SimulationConfig,
    simulate, SimulationResult, SwapAnalyzer
};
use alloy_primitives::{Address, Bytes, U256};

/// Complete example: Simulate a sandwich attack
async fn simulate_sandwich(
    victim_tx: &Transaction,
    pool_address: Address,
    my_address: Address,
) -> Result<SandwichResult, SimulationError> {

    // 1. Create fresh database
    let mut db = SimulationDb::new();

    // 2. Fund our account generously (for testing)
    db.set_balance(my_address, U256::from(1000) * U256::from(10).pow(U256::from(18)));

    // 3. Also need to fund the victim (simulate their state)
    db.set_balance(victim_tx.from, victim_tx.value + estimate_gas_cost(victim_tx));

    // 4. Build FRONT-RUN transaction
    let front_run_amount = calculate_optimal_front_run(victim_tx);
    let front_run = SimulationTx::new(
        my_address,
        pool_address,
        encode_swap(front_run_amount, Direction::Buy)
    ).with_gas(200_000);

    // 5. Build BACK-RUN transaction
    let back_run = SimulationTx::new(
        my_address,
        pool_address,
        encode_swap(front_run_amount, Direction::Sell)
    ).with_gas(200_000);

    // 6. Build victim transaction from their calldata
    let victim_sim = SimulationTx::new(
        victim_tx.from,
        victim_tx.to,
        victim_tx.input.clone()
    ).with_value(victim_tx.value)
     .with_gas(victim_tx.gas_limit);

    // 7. Simulate as a BUNDLE (all 3 transactions in sequence)
    let bundle = vec![front_run, victim_sim, back_run];
    let results = simulate_bundle(&bundle, db, &SimulationConfig::default())?;

    // 8. Analyze results
    let [front_result, victim_result, back_result] = results.as_slice() else {
        return Err(SimulationError::InvalidBundle);
    };

    // 9. All must succeed for sandwich to work
    if !front_result.is_success() || !victim_result.is_success() || !back_result.is_success() {
        return Ok(SandwichResult::NotProfitable {
            reason: "One or more transactions would revert".into()
        });
    }

    // 10. Calculate profit
    let gas_cost = calculate_gas_cost(
        front_result.gas_used + back_result.gas_used,
        current_gas_price()
    );

    let tokens_out = extract_tokens_received(&back_result.logs);
    let tokens_in = front_run_amount;
    let gross_profit = tokens_out.saturating_sub(tokens_in);
    let net_profit = gross_profit.saturating_sub(gas_cost);

    if net_profit > U256::ZERO {
        Ok(SandwichResult::Profitable {
            gross_profit,
            gas_cost,
            net_profit,
            front_run_tx: front_result,
            back_run_tx: back_result,
        })
    } else {
        Ok(SandwichResult::NotProfitable {
            reason: format!("Gross {} - Gas {} = Loss", gross_profit, gas_cost)
        })
    }
}
```

## Analyzing Simulation Results

```
Understanding Simulation Output
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  SimulationResult {                                                         │
│                                                                             │
│    status: Success | Revert | OutOfGas | Halt                              │
│    ──────────────────────────────────────────────                           │
│    • Success: Transaction completed normally                               │
│    • Revert: require() failed, transaction rolled back                     │
│    • OutOfGas: Ran out of gas during execution                             │
│    • Halt: Invalid opcode or other fatal error                             │
│                                                                             │
│    gas_used: 152847                                                         │
│    ────────────────                                                         │
│    Actual gas consumed. Use for accurate cost calculation:                 │
│    cost = gas_used × gas_price                                             │
│                                                                             │
│    logs: [Log, Log, ...]                                                    │
│    ─────────────────────                                                    │
│    Events emitted during execution. Critical for MEV:                      │
│    • Swap events → tokens exchanged                                        │
│    • Transfer events → token movements                                     │
│    • Sync events → new reserves (for price updates)                        │
│                                                                             │
│    output: Bytes                                                            │
│    ─────────────                                                            │
│    Return data from the call:                                              │
│    • On success: function return value                                     │
│    • On revert: error message (often ABI-encoded)                          │
│                                                                             │
│  }                                                                          │
│                                                                             │
│  DECODING SWAP EVENTS:                                                      │
│  ─────────────────────                                                      │
│                                                                             │
│  Uniswap V2 Swap Event:                                                    │
│  event Swap(                                                                │
│    address indexed sender,                                                 │
│    uint256 amount0In,                                                      │
│    uint256 amount1In,                                                      │
│    uint256 amount0Out,                                                     │
│    uint256 amount1Out,                                                     │
│    address indexed to                                                      │
│  );                                                                         │
│                                                                             │
│  From this event you can determine:                                        │
│  • Which direction the swap went (amount0In > 0 or amount1In > 0)          │
│  • Exact amounts exchanged                                                 │
│  • Effective exchange rate                                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

# Article 5: Arbitrage Detection

## Graph-Based Market Representation

```
Modeling DEXs as a Graph
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  CONCEPT:                                                                   │
│  • Each TOKEN is a NODE                                                    │
│  • Each POOL is TWO EDGES (one per direction)                              │
│  • Edge WEIGHT is the exchange rate                                        │
│                                                                             │
│  EXAMPLE MARKET:                                                            │
│                                                                             │
│                    ┌─────────────────────────────────┐                     │
│                    │          Uniswap V2             │                     │
│                    │     ETH/USDC: $2000/ETH         │                     │
│                    └─────────────────────────────────┘                     │
│                              │                                              │
│              ┌───────────────┴───────────────┐                             │
│              │                               │                             │
│              ▼                               ▼                             │
│         ┌────────┐                     ┌────────┐                          │
│         │  ETH   │◀────────────────────│  USDC  │                          │
│         │        │  rate: 1/2000       │        │                          │
│         │        │────────────────────▶│        │                          │
│         └────────┘  rate: 2000         └────────┘                          │
│              │                               │                             │
│              │                               │                             │
│     SushiSwap│                               │ Curve                       │
│   rate: 2010 │                               │ rate: 1.001                 │
│              │                               │                             │
│              ▼                               ▼                             │
│         ┌────────┐                     ┌────────┐                          │
│         │  USDT  │◀────────────────────│  DAI   │                          │
│         │        │  Curve: 0.999       │        │                          │
│         └────────┘                     └────────┘                          │
│                                                                             │
│  FINDING ARBITRAGE = FINDING PROFITABLE CYCLES                             │
│                                                                             │
│  Cycle: ETH → USDC → DAI → USDT → ETH                                      │
│  Rate:  2000 × 1.001 × 0.999 × (1/2010)                                    │
│       = 2000 × 1.001 × 0.999 / 2010                                        │
│       = 0.995 (NOT profitable, lose 0.5%)                                  │
│                                                                             │
│  Reverse: ETH → USDT → DAI → USDC → ETH                                    │
│  Rate:    2010 × (1/0.999) × (1/1.001) × (1/2000)                          │
│        =  2010 / 0.999 / 1.001 / 2000                                      │
│        =  1.005 (PROFITABLE! gain 0.5%)                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Cycle Detection Algorithm

```
DFS-Based Cycle Detection
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  ALGORITHM:                                                                 │
│  ──────────                                                                 │
│  1. For each token as starting point                                       │
│  2. DFS with max depth (usually 3-4 hops)                                  │
│  3. Track visited edges to avoid duplicates                                │
│  4. When we return to start token = cycle found!                           │
│  5. Calculate cycle profitability                                          │
│                                                                             │
│  PSEUDOCODE:                                                                │
│  ───────────                                                                │
│                                                                             │
│  function findCycles(graph, maxDepth):                                      │
│      cycles = []                                                            │
│                                                                             │
│      for each token in graph.tokens:                                        │
│          dfs(token, [token], [], cycles, maxDepth)                         │
│                                                                             │
│      return cycles.filter(c => c.isProfitable())                           │
│                                                                             │
│  function dfs(current, path, edges, cycles, depth):                        │
│      if depth == 0:                                                         │
│          return                                                             │
│                                                                             │
│      for each edge from current:                                            │
│          if edge.to == path[0] and len(path) >= 2:                         │
│              // Found a cycle back to start!                               │
│              cycles.add(Cycle(path, edges + [edge]))                       │
│          else if edge.to not in path:                                      │
│              // Continue exploring                                         │
│              dfs(edge.to, path + [edge.to], edges + [edge], cycles, depth-1)│
│                                                                             │
│  OPTIMIZATION - LOG RATES:                                                  │
│  ─────────────────────────                                                  │
│  Instead of multiplying rates, ADD log(rates)                              │
│                                                                             │
│  Profitable if: rate1 × rate2 × rate3 > 1                                  │
│  Equivalent:    log(rate1) + log(rate2) + log(rate3) > 0                   │
│                                                                             │
│  Benefits:                                                                  │
│  • Faster (addition vs multiplication)                                     │
│  • More numerically stable                                                 │
│  • Can precompute log(rate) for each edge                                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Finding Optimal Input Amount

```
Binary Search for Maximum Profit
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  PROBLEM:                                                                   │
│  We found a profitable cycle, but how much should we trade?                │
│                                                                             │
│  • Too little: profit doesn't cover gas                                    │
│  • Too much: slippage eats all profit                                      │
│  • Just right: maximum profit                                              │
│                                                                             │
│  PROFIT CURVE:                                                              │
│                                                                             │
│  Profit │                                                                  │
│         │              ●●●                                                 │
│         │           ●●●   ●●●                                              │
│         │         ●●         ●●                                            │
│         │       ●●             ●●                                          │
│         │     ●●                 ●●                                        │
│         │   ●●                     ●●                                      │
│      0 ─┼──●────────────────────────●●●──────────▶ Amount In               │
│         │                              ●●●                                 │
│         │                                 ●●●                              │
│   Loss  │                                    ●●●                           │
│         │                                                                  │
│         0      Optimal                 Max                                 │
│                Amount                  Reserves                            │
│                                                                             │
│  BINARY SEARCH ALGORITHM:                                                   │
│  ────────────────────────                                                   │
│                                                                             │
│  low = 0                                                                    │
│  high = max_reasonable_amount (e.g., 10% of smallest reserve)              │
│                                                                             │
│  for i in 0..100:  // 100 iterations for precision                         │
│      mid = (low + high) / 2                                                │
│                                                                             │
│      profit_at_mid = simulate_cycle(mid)                                   │
│      profit_at_mid_plus = simulate_cycle(mid + delta)                      │
│                                                                             │
│      if profit_at_mid_plus > profit_at_mid:                                │
│          low = mid  // Optimum is higher                                   │
│      else:                                                                  │
│          high = mid  // Optimum is lower or here                           │
│                                                                             │
│  return mid                                                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Complete Arbitrage Implementation

```rust
use mev_lib::prelude::*;
use mev_lib::math::uniswap_v2;

/// Market graph for arbitrage detection
pub struct World {
    tokens: Vec<TokenId>,
    pools: Vec<Pool>,
    /// Adjacency list: token_index -> [(swap_index, direction), ...]
    graph: Vec<Vec<(usize, Direction)>>,
}

/// A trading cycle
pub struct Cycle {
    swaps: Vec<(usize, Direction)>,  // Pool index + direction
    log_rate: f64,                    // Precomputed for fast filtering
}

impl World {
    /// Find all cycles up to max_depth hops
    pub fn find_cycles(&self, max_depth: usize) -> Vec<Cycle> {
        let mut cycles = Vec::new();

        for start_token in 0..self.tokens.len() {
            self.dfs(
                start_token,
                start_token,
                Vec::new(),
                0.0,  // cumulative log rate
                max_depth,
                &mut cycles,
            );
        }

        cycles
    }

    fn dfs(
        &self,
        start: usize,
        current: usize,
        path: Vec<(usize, Direction)>,
        log_rate: f64,
        depth: usize,
        cycles: &mut Vec<Cycle>,
    ) {
        if depth == 0 {
            return;
        }

        for &(pool_idx, direction) in &self.graph[current] {
            let pool = &self.pools[pool_idx];
            let next_token = pool.other_token(current, direction);
            let edge_log_rate = pool.log_rate(direction);

            // Check if we've completed a cycle
            if next_token == start && path.len() >= 1 {
                let mut cycle_path = path.clone();
                cycle_path.push((pool_idx, direction));

                cycles.push(Cycle {
                    swaps: cycle_path,
                    log_rate: log_rate + edge_log_rate,
                });
            } else if !path.iter().any(|(p, _)| *p == pool_idx) {
                // Continue DFS (don't revisit same pool)
                let mut new_path = path.clone();
                new_path.push((pool_idx, direction));

                self.dfs(
                    start,
                    next_token,
                    new_path,
                    log_rate + edge_log_rate,
                    depth - 1,
                    cycles,
                );
            }
        }
    }

    /// Find profitable cycles and optimize amounts
    pub fn find_profitable_opportunities(&self) -> Vec<Opportunity> {
        self.find_cycles(3)
            .into_iter()
            .filter(|c| c.log_rate > 0.0)  // Only profitable cycles
            .filter_map(|cycle| {
                let (optimal_amount, profit) = self.optimize_amount(&cycle)?;

                // Check if profit > gas cost
                let gas_cost = estimate_gas_cost(&cycle);
                if profit > gas_cost {
                    Some(Opportunity {
                        cycle,
                        amount_in: optimal_amount,
                        expected_profit: profit - gas_cost,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
```

---

# Article 6: Building a Complete MEV Bot

## Complete Bot Architecture

```
MEV Bot System Design
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                         ┌─────────────────────┐                            │
│                         │    Mempool Feed     │                            │
│                         │   (WebSocket/IPC)   │                            │
│                         └──────────┬──────────┘                            │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        TRANSACTION PROCESSOR                         │   │
│  │                                                                     │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │   │
│  │  │   Decoder   │  │   Filter    │  │  Classifier │                 │   │
│  │  │             │  │             │  │             │                 │   │
│  │  │ Parse       │  │ Check if    │  │ Arbitrage?  │                 │   │
│  │  │ calldata    │  │ interesting │  │ Sandwich?   │                 │   │
│  │  │             │  │ (is swap?)  │  │ Liquidation?│                 │   │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                 │   │
│  │         │                │                │                         │   │
│  └─────────┼────────────────┼────────────────┼─────────────────────────┘   │
│            │                │                │                              │
│            ▼                ▼                ▼                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                       STRATEGY ENGINE                                │   │
│  │                                                                     │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐     │   │
│  │  │   Arbitrage     │  │    Sandwich     │  │   Liquidation   │     │   │
│  │  │   Strategy      │  │    Strategy     │  │   Strategy      │     │   │
│  │  │                 │  │                 │  │                 │     │   │
│  │  │ • Find cycles   │  │ • Calculate     │  │ • Monitor       │     │   │
│  │  │ • Optimize amt  │  │   front/back    │  │   health        │     │   │
│  │  │ • Check profit  │  │ • Simulate      │  │ • Simulate      │     │   │
│  │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘     │   │
│  │           │                    │                    │               │   │
│  └───────────┼────────────────────┼────────────────────┼───────────────┘   │
│              │                    │                    │                    │
│              ▼                    ▼                    ▼                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         SIMULATOR (Revm)                             │   │
│  │                                                                     │   │
│  │  • Fork current state                                               │   │
│  │  • Execute transactions locally                                     │   │
│  │  • Verify profitability                                             │   │
│  │  • Calculate exact gas                                              │   │
│  │                                                                     │   │
│  └──────────────────────────────────┬──────────────────────────────────┘   │
│                                     │                                       │
│                    Only if profitable│                                      │
│                                     ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        BUNDLE BUILDER                                │   │
│  │                                                                     │   │
│  │  • Create Flashbots bundle                                          │   │
│  │  • Sign transactions                                                │   │
│  │  • Set appropriate gas price / priority fee                         │   │
│  │                                                                     │   │
│  └──────────────────────────────────┬──────────────────────────────────┘   │
│                                     │                                       │
│                                     ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      SUBMISSION ENGINE                               │   │
│  │                                                                     │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │   │
│  │  │  Flashbots  │  │  MEV-Share  │  │   Direct    │                 │   │
│  │  │   Relay     │  │             │  │   to Block  │                 │   │
│  │  │             │  │             │  │   Builder   │                 │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘                 │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Event Loop Pattern

```rust
/// Main bot event loop
async fn run_bot(config: BotConfig) -> Result<()> {
    // Initialize components
    let provider = create_provider(&config.rpc_url).await?;
    let mempool = MempoolSubscriber::new(&provider).await?;
    let simulator = Simulator::new(&provider).await?;
    let bundle_client = FlashbotsClient::new(&config.flashbots_key)?;

    // State
    let mut market = Market::new();

    // Subscribe to new blocks for state updates
    let mut blocks = provider.subscribe_blocks().await?;

    // Subscribe to pending transactions
    let mut pending_txs = mempool.subscribe().await?;

    loop {
        tokio::select! {
            // New block arrived - update market state
            Some(block) = blocks.next() => {
                market.update_from_block(&block).await?;
            }

            // New pending transaction
            Some(tx) = pending_txs.next() => {
                // 1. Try to decode
                let Some(decoded) = decode_calldata(&tx.input) else {
                    continue;
                };

                // 2. Check if it's interesting
                if !is_interesting_swap(&decoded) {
                    continue;
                }

                // 3. Run strategies in parallel
                let opportunities = tokio::join!(
                    check_arbitrage(&market, &tx, &decoded),
                    check_sandwich(&market, &tx, &decoded),
                );

                // 4. Execute best opportunity
                if let Some(best) = select_best_opportunity(opportunities) {
                    let bundle = best.to_bundle()?;

                    // 5. Final simulation check
                    if simulator.verify_profitable(&bundle).await? {
                        // 6. Submit!
                        let result = bundle_client.submit(bundle).await;
                        log_result(&result);
                    }
                }
            }
        }
    }
}
```

---

## Recommended Images

### Diagrams to Create

| Diagram | Tool | Description |
|---------|------|-------------|
| MEV Flow | Excalidraw | Transaction lifecycle with MEV extraction points |
| Sandwich Attack | Mermaid | Sequence diagram of front-run/victim/back-run |
| AMM Curve | Desmos | Interactive x*y=k curve with trade visualization |
| Market Graph | D3.js | Token nodes with exchange rate edges |
| Bot Architecture | Draw.io | Component diagram of full MEV bot |

### Screenshots

| Screenshot | Source | Purpose |
|------------|--------|---------|
| Etherscan TX | etherscan.io | Real swap transaction calldata |
| Flashbots Dashboard | flashbots.net | MEV extraction statistics |
| Block with MEV | etherscan.io | Show sandwich in real block |

### External Images (with attribution)

| Image | Source | License |
|-------|--------|---------|
| Ethereum Logo | ethereum.org | CC BY |
| Uniswap Logo | uniswap.org | Brand guidelines |
| Rust Logo | rust-lang.org | CC BY |

---

*This documentation provides comprehensive educational content for Medium articles about MEV and the mev-lib library. Each article section includes code examples, ASCII diagrams, and clear explanations suitable for developers learning about MEV.*
