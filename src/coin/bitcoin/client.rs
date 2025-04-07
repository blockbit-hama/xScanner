/**
* author : HAMA
* date: 2025. 4. 5.
* description:
**/

use reqwest::{Client, Error};
use serde::Deserialize;

use crate::coin::coin_trait::BlockchainClient;
use crate::coin::bitcoin::model::BitcoinBlock;
use async_trait::async_trait;
use serde::de::DeserializeOwned;


#[derive(Clone)]
pub struct BitcoinClient {
  client: Client,
  api_url: String,
}

impl BitcoinClient {
  pub fn new(api_url: String) -> Self {
    Self {
      client: Client::new(),
      api_url,
    }
  }
  
  pub async fn fetch_block_by_number(&self, block_number: u64) -> Result<BitcoinBlock, reqwest::Error> {
    let url = format!("{}{}", self.api_url, block_number);
    self.fetch_json_url(&url).await
  }
  
  async fn fetch_json_url<T>(&self, url: &str) -> Result<T, reqwest::Error>
  where
    T: DeserializeOwned,
  {
    self.client.get(url).send().await?.json::<T>().await
  }
}

#[async_trait]
impl BlockchainClient for BitcoinClient {
  fn get_http_client(&self) -> &Client {
    &self.client
  }
  
  fn get_api_url(&self) -> &str {
    &self.api_url
  }
}
