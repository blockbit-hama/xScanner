use reqwest::Client;
use serde_json::json;
use async_trait::async_trait;
use crate::coin::coin_trait::BlockchainClient;
use crate::coin::quark::model::QuarkBlock;

#[derive(Clone)]
pub struct QuarkClient {
  client: Client,
  api_url: String,
}

impl QuarkClient {
  pub fn new(api_url: String) -> Self {
    Self {
      client: Client::new(),
      api_url,
    }
  }

  pub async fn fetch_block_by_number(&self, block_number: u64) -> Result<QuarkBlock, Box<dyn std::error::Error>> {
    let block_number_hex = format!("0x{:x}", block_number);
    let payload = json!({
      "jsonrpc": "2.0",
      "method": "eth_getBlockByNumber",
      "params": [block_number_hex, true],
      "id": 1
    });

    let response = self.client.post(&self.api_url).json(&payload).send().await?;
    let response_text = response.text().await?;
    log::debug!("[QUARK] Raw API response: {}", response_text);
    let block: QuarkBlock = serde_json::from_str(&response_text)?;

    if block.result.is_none() {
      return Err("Block not yet created (result is null)".into());
    }

    Ok(block)
  }
}

#[async_trait]
impl BlockchainClient for QuarkClient {
  fn get_http_client(&self) -> &Client {
    &self.client
  }

  fn get_api_url(&self) -> &str {
    &self.api_url
  }
}
