use reqwest::Client;
use async_trait::async_trait;
use crate::coin::coin_trait::BlockchainClient;
use crate::coin::terra::model::TerraBlock;

#[derive(Clone)]
pub struct TerraClient {
  client: Client,
  api_url: String,
}

impl TerraClient {
  pub fn new(api_url: String) -> Self {
    Self {
      client: Client::new(),
      api_url,
    }
  }

  pub async fn fetch_block_by_number(&self, block_number: u64) -> Result<TerraBlock, Box<dyn std::error::Error>> {
    let url = format!("{}/block?height={}", self.api_url, block_number);
    let response = self.client.get(&url).send().await?;
    let response_text = response.text().await?;
    log::debug!("[TERRA] Raw API response: {}", response_text);
    let block: TerraBlock = serde_json::from_str(&response_text)?;
    Ok(block)
  }
}

#[async_trait]
impl BlockchainClient for TerraClient {
  fn get_http_client(&self) -> &Client {
    &self.client
  }

  fn get_api_url(&self) -> &str {
    &self.api_url
  }
}
