/**
* filename : coin_trait
* author : HAMA
* date: 2025. 4. 7.
* description: 
**/

use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;

#[async_trait]
pub trait BlockchainClient: Clone + Send + Sync {
  fn get_http_client(&self) -> &Client;
  fn get_api_url(&self) -> &str;
  
  async fn fetch_json<T>(&self, url_or_payload: &serde_json::Value) -> Result<T, reqwest::Error>
  where
    T: DeserializeOwned + Send,
  {
    let client = self.get_http_client();
    let url = self.get_api_url();
    
    let response = client
      .post(url)
      .header("Content-Type", "application/json")
      .json(url_or_payload)
      .send()
      .await?
      .json::<T>()
      .await?;
    
    Ok(response)
  }
}
