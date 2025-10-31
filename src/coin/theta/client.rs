use reqwest::Client;
use serde_json::json;
use serde::Deserialize;
use crate::coin::coin_trait::BlockchainClient;
use crate::coin::theta::model::ThetaBlock;
use async_trait::async_trait;
use std::io::{Error as IoError, ErrorKind};

#[derive(Clone)]
pub struct ThetaClient {
    client: Client,
    api_url: String,
}

impl ThetaClient {
    pub fn new(api_url: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
        }
    }
    
    pub async fn fetch_block_by_number(&self, block_number: u64) -> Result<ThetaBlock, Box<dyn std::error::Error>> {
        // THETA RPC: eth_getBlockByNumber
        let block_number_hex = format!("0x{:X}", block_number);
        let payload = json!({
            "jsonrpc": "2.0",
            "method": "eth_getBlockByNumber",
            "params": [block_number_hex, true],
            "id": 1
        });

        #[derive(Deserialize)]
        struct RpcResponse {
            result: Option<ThetaBlock>,
        }

        let response: RpcResponse = self.client
            .post(&self.api_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        match response.result {
            Some(block) => Ok(block),
            None => Err(Box::new(IoError::new(ErrorKind::Other, "Theta RPC returned no result")))
        }
    }
}

#[async_trait]
impl BlockchainClient for ThetaClient {
    fn get_http_client(&self) -> &Client {
        &self.client
    }
    
    fn get_api_url(&self) -> &str {
        &self.api_url
    }
}
