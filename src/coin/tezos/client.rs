use reqwest::Client;
use async_trait::async_trait;
use crate::coin::coin_trait::BlockchainClient;
use crate::coin::tezos::model::TezosBlock;

#[derive(Clone)]
pub struct TezosClient {
  client: Client,
  api_url: String,
}

impl TezosClient {
  pub fn new(api_url: String) -> Self {
    Self {
      client: Client::new(),
      api_url,
    }
  }

  pub async fn fetch_block_by_number(&self, block_number: u64) -> Result<TezosBlock, Box<dyn std::error::Error>> {
    let url = format!("{}/chains/main/blocks/{}", self.api_url, block_number);
    let response = self.client.get(&url).send().await?;
    let response_text = response.text().await?;
    log::debug!("[TEZOS] Raw API response: {}", response_text);
    let block: TezosBlock = serde_json::from_str(&response_text)?;
    Ok(block)
  }
}

#[async_trait]
impl BlockchainClient for TezosClient {
  fn get_http_client(&self) -> &Client {
    &self.client
  }

  fn get_api_url(&self) -> &str {
    &self.api_url
  }
}
