/**
* author : HAMA
* date: 2025. 4. 5.
* description:
**/

use reqwest;
use serde::Deserialize;
use log::info;
use crate::coin::coin_trait::BlockchainClient;
use crate::coin::ethereum::model::EthereumBlock;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

#[derive(Clone)]
pub struct EthereumClient {
  client: Client,
  api_url: String,
}

impl EthereumClient {
  pub fn new(api_url: String) -> Self {
    Self {
      client: Client::new(),
      api_url,
    }
  }
  
  pub async fn fetch_block_by_number(&self, block_number: u64) -> Result<EthereumBlock, reqwest::Error> {
    let block_number_hex = format!("0x{:X}", block_number);
    let payload = json!({
            "jsonrpc": "2.0",
            "method": "eth_getBlockByNumber",
            "params": [block_number_hex, true],
            "id": 1
        });
    
    self.fetch_json(&payload).await
  }
}

#[async_trait]
impl BlockchainClient for EthereumClient {
  fn get_http_client(&self) -> &Client {
    &self.client
  }
  
  fn get_api_url(&self) -> &str {
    &self.api_url
  }
}
