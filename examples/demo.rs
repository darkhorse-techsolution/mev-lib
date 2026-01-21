//! Demo example showing mev-lib functionality.
//!
//! Run with: cargo run --example demo

use alloy_primitives::{Address, Bytes, U256};
use mev_lib::{
    decoder::{decode_calldata, DecodedCall},
    math::uniswap_v2,
    types::{Pool, PoolId, Reserves, TokenId},
};

fn main() {
    println!("=== mev-lib Demo ===\n");

    // 1. Types Demo
    demo_types();

    // 2. Math Demo
    demo_math();

    // 3. Decoder Demo
    demo_decoder();

    println!("\n✓ All demos completed successfully!");
}

fn demo_types() {
    println!("--- Types Module ---");

    // Create tokens (WETH and USDC addresses)
    let weth = TokenId::new(Address::repeat_byte(0x01));
    let usdc = TokenId::new(Address::repeat_byte(0x02));

    println!("WETH: {}", weth);
    println!("USDC: {}", usdc);

    // Create a pool
    let pool = Pool::new(
        PoolId::new(Address::repeat_byte(0xAA)),
        weth,
        usdc,
        Reserves::new(
            U256::from(1_000_000_000_000_000_000_000u128), // 1000 WETH
            U256::from(2_000_000_000_000u128),              // 2M USDC (6 decimals)
        ),
    );

    println!("Pool: {} / {}", pool.token0, pool.token1);
    println!("Reserves: {} / {}", pool.reserve0(), pool.reserve1());
    println!();
}

fn demo_math() {
    println!("--- Math Module (Uniswap V2) ---");

    // Pool reserves
    let reserve_in = U256::from(100_000_000_000u64);  // 100B tokens
    let reserve_out = U256::from(100_000_000_000u64); // 100B tokens

    // Swap 1M tokens
    let amount_in = U256::from(1_000_000u64);

    // Calculate output (0.3% fee = 30 basis points)
    let amount_out = uniswap_v2::get_amount_out(amount_in, reserve_in, reserve_out, 30);
    println!("Input:  {} tokens", amount_in);
    println!("Output: {} tokens", amount_out);

    // Calculate spot price
    let spot = uniswap_v2::spot_price(reserve_in, reserve_out);
    println!("Spot Price: {:.6}", spot);

    // Calculate price impact
    let impact = uniswap_v2::price_impact(amount_in, reserve_in, reserve_out, 30);
    println!("Price Impact: {:.4}%", impact * 100.0);

    // Simulate swap (get new reserves and amount out)
    let (new_reserve_in, new_reserve_out, swap_amount_out) =
        uniswap_v2::simulate_swap(amount_in, reserve_in, reserve_out, 30);
    println!("Swap output: {} tokens", swap_amount_out);
    println!("New reserves after swap: {} / {}", new_reserve_in, new_reserve_out);
    println!();
}

fn demo_decoder() {
    println!("--- Decoder Module ---");

    // 1. Decode a Uniswap V2 swapExactTokensForTokens call
    // Selector: 0x38ed1739
    let v2_calldata = build_v2_swap_calldata();
    match decode_calldata(&v2_calldata) {
        Some(DecodedCall::UniswapV2(swap)) => {
            println!("Decoded Uniswap V2 Swap:");
            println!("  Method: {:?}", swap.method);
            println!("  Amount In: {:?}", swap.amount_in);
            println!("  Amount Out Min: {:?}", swap.amount_out_min);
            println!("  Path length: {} tokens", swap.path.len());
            println!("  Deadline: {:?}", swap.deadline);
        }
        other => println!("Unexpected decode result: {:?}", other),
    }

    // 2. Decode an ERC20 transfer
    // Selector: 0xa9059cbb
    let transfer_calldata = build_transfer_calldata();
    match decode_calldata(&transfer_calldata) {
        Some(DecodedCall::Transfer(transfer)) => {
            println!("\nDecoded ERC20 Transfer:");
            println!("  To: {}", transfer.to);
            println!("  Amount: {}", transfer.amount);
        }
        other => println!("Unexpected decode result: {:?}", other),
    }

    // 3. Unknown selector
    let unknown = Bytes::from(vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00]);
    match decode_calldata(&unknown) {
        Some(DecodedCall::Unknown(selector)) => {
            println!("\nUnknown selector: {}", selector);
        }
        other => println!("Unexpected decode result: {:?}", other),
    }
}

/// Build a mock Uniswap V2 swapExactTokensForTokens calldata
fn build_v2_swap_calldata() -> Bytes {
    let mut data = Vec::new();

    // Selector: swapExactTokensForTokens
    data.extend_from_slice(&[0x38, 0xed, 0x17, 0x39]);

    // amountIn (32 bytes)
    let amount_in = U256::from(1_000_000_000_000_000_000u128); // 1 ETH
    data.extend_from_slice(&amount_in.to_be_bytes::<32>());

    // amountOutMin (32 bytes)
    let amount_out_min = U256::from(1_900_000_000u128); // 1900 USDC
    data.extend_from_slice(&amount_out_min.to_be_bytes::<32>());

    // path offset (32 bytes) - points to 160 (0xa0)
    let path_offset = U256::from(160u64);
    data.extend_from_slice(&path_offset.to_be_bytes::<32>());

    // to address (32 bytes, left-padded)
    let to = Address::repeat_byte(0x42);
    let mut to_padded = [0u8; 32];
    to_padded[12..].copy_from_slice(to.as_slice());
    data.extend_from_slice(&to_padded);

    // deadline (32 bytes)
    let deadline = U256::from(1700000000u64);
    data.extend_from_slice(&deadline.to_be_bytes::<32>());

    // path array length (32 bytes)
    let path_len = U256::from(2u64);
    data.extend_from_slice(&path_len.to_be_bytes::<32>());

    // path[0] - WETH
    let weth = Address::repeat_byte(0x01);
    let mut weth_padded = [0u8; 32];
    weth_padded[12..].copy_from_slice(weth.as_slice());
    data.extend_from_slice(&weth_padded);

    // path[1] - USDC
    let usdc = Address::repeat_byte(0x02);
    let mut usdc_padded = [0u8; 32];
    usdc_padded[12..].copy_from_slice(usdc.as_slice());
    data.extend_from_slice(&usdc_padded);

    Bytes::from(data)
}

/// Build a mock ERC20 transfer calldata
fn build_transfer_calldata() -> Bytes {
    let mut data = Vec::new();

    // Selector: transfer(address,uint256)
    data.extend_from_slice(&[0xa9, 0x05, 0x9c, 0xbb]);

    // to address (32 bytes, left-padded)
    let to = Address::repeat_byte(0x99);
    let mut to_padded = [0u8; 32];
    to_padded[12..].copy_from_slice(to.as_slice());
    data.extend_from_slice(&to_padded);

    // amount (32 bytes)
    let amount = U256::from(500_000_000_000u128); // 500k tokens
    data.extend_from_slice(&amount.to_be_bytes::<32>());

    Bytes::from(data)
}
