use reqwest::Client;
use serde_json::json;
use crate::coin::coin_trait::BlockchainClient;
use crate::coin::tron::model::TronBlock;
use async_trait::async_trait;

#[derive(Clone)]
pub struct TronClient {
    client: Client,
    api_url: String,
}

impl TronClient {
    pub fn new(api_url: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
        }
    }
    
    pub async fn fetch_block_by_number(&self, block_number: u64) -> Result<TronBlock, reqwest::Error> {
        // TRON API: /wallet/getblockbynum
        let url = format!("{}/wallet/getblockbynum", self.api_url);
        let payload = json!({
            "num": block_number
        });
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?
            .json::<TronBlock>()
            .await?;
        
        Ok(response)
    }
}

#[async_trait]
impl BlockchainClient for TronClient {
    fn get_http_client(&self) -> &Client {
        &self.client
    }
    
    fn get_api_url(&self) -> &str {
        &self.api_url
    }
}
