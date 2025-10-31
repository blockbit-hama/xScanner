/**
 * author: HAMA
 * date: 2025. 10. 31.
 * description: ALGORAND blockchain client
 **/

use reqwest::Client;
use async_trait::async_trait;
use crate::coin::coin_trait::BlockchainClient;
use crate::coin::algorand::model::AlgorandBlock;

#[derive(Clone)]
pub struct AlgorandClient {
  client: Client,
  api_url: String,
}

impl AlgorandClient {
  pub fn new(api_url: String) -> Self {
    Self {
      client: Client::new(),
      api_url,
    }
  }

  pub async fn fetch_block_by_number(&self, block_number: u64) -> Result<AlgorandBlock, Box<dyn std::error::Error>> {
    let url = format!("{}/v2/blocks/{}", self.api_url, block_number);

    let response = self.client
      .get(&url)
      .send()
      .await?;

    let response_text = response.text().await?;
    log::debug!("[ALGORAND] Raw API response: {}", response_text);

    let block: AlgorandBlock = serde_json::from_str(&response_text)?;

    Ok(block)
  }
}

#[async_trait]
impl BlockchainClient for AlgorandClient {
  fn get_http_client(&self) -> &Client {
    &self.client
  }

  fn get_api_url(&self) -> &str {
    &self.api_url
  }
}
