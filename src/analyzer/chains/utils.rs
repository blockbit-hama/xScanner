use rust_decimal::Decimal;

/// Wei(Hex)를 Decimal로 변환 (Ethereum, AION, QUARK, THETA 등)
pub fn parse_wei_to_decimal(wei_hex: &str) -> Result<Decimal, String> {
    let wei_str = if wei_hex.starts_with("0x") || wei_hex.starts_with("0X") {
        &wei_hex[2..]
    } else {
        wei_hex
    };

    let wei = u128::from_str_radix(wei_str, 16)
        .map_err(|e| format!("Failed to parse wei: {}", e))?;

    // Wei를 ETH로 변환 (1 ETH = 10^18 Wei)
    let eth = Decimal::from(wei) / Decimal::from(1_000_000_000_000_000_000u128);

    Ok(eth)
}

/// Satoshi를 BTC로 변환
pub fn satoshi_to_btc(satoshi: u64) -> Decimal {
    Decimal::from(satoshi) / Decimal::from(100_000_000u64)
}

/// SUN을 TRX로 변환
pub fn sun_to_trx(sun: u64) -> Decimal {
    Decimal::from(sun) / Decimal::from(1_000_000u64)
}

/// Loop을 ICX로 변환
pub fn loop_to_icx(loop_value: u64) -> Decimal {
    Decimal::from(loop_value) / Decimal::from(1_000_000_000_000_000_000u64)
}

/// MicroAlgo를 ALGO로 변환
pub fn microalgo_to_algo(microalgo: u64) -> Decimal {
    Decimal::from(microalgo) / Decimal::from(1_000_000u64)
}
