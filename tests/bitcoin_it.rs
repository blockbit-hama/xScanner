
use xScaner::coin::bitcoin::client::BitcoinClient;

#[tokio::test]
pub async fn bitcoin_test() {
  let block_number = 654596;
  let client = BitcoinClient::new("https://blockchain.info/rawblock/".to_string());
  let response_result = client.fetch_block_by_number(block_number).await;
  
  match response_result {
    Ok(response) => {
      for transaction in response.tx {
        for out in transaction.out {
          if let Some(address) = &out.addr {
            if is_valid_bitcoin_address(address) {
              println!("Valid address: {}", address);
            } else {
              println!("⚠️ Warning: Invalid Bitcoin address detected: {}", address);
            }
          }
        }
      }
    }
    Err(err) => {
      eprintln!("Failed to fetch block data: {:?}", err);
    }
  }
}

/// 비트코인 주소가 올바른 형식인지 검증하는 함수
fn is_valid_bitcoin_address(address: &str) -> bool {
  let bech32_prefixes = ["bc1", "tb1", "bc1p", "tb1p"];
  let base58_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
  
  if address.starts_with("1") || address.starts_with("3") {
    // Base58Check 형식 (P2PKH, P2SH)
    address.len() >= 26 && address.len() <= 35 && address.chars().all(|c| base58_chars.contains(c))
  } else if bech32_prefixes.iter().any(|&prefix| address.starts_with(prefix)) {
    // Bech32 형식 (P2WPKH, P2WSH, Taproot)
    address.len() >= 14 && address.len() <= 90
  } else {
    false
  }
}
