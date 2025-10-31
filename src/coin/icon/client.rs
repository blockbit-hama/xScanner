use reqwest::Client;
use serde_json::json;
use crate::coin::coin_trait::BlockchainClient;
use crate::coin::icon::model::IconBlock;
use async_trait::async_trait;
use serde::Deserialize;
use std::io::{Error as IoError, ErrorKind};

#[derive(Clone)]
pub struct IconClient {
    client: Client,
    api_url: String,
}

impl IconClient {
    pub fn new(api_url: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
        }
    }
    
    pub async fn fetch_block_by_number(&self, block_number: u64) -> Result<IconBlock, Box<dyn std::error::Error>> {
        // ICON RPC: icx_getBlockByHeight
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "icx_getBlockByHeight",
            "params": {
                "height": format!("0x{:X}", block_number)
            }
        });

        #[derive(Deserialize)]
        struct RpcResponse {
            result: Option<IconBlock>,
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
            None => Err(Box::new(IoError::new(ErrorKind::Other, "ICON RPC returned no result")))
        }
    }
}

#[async_trait]
impl BlockchainClient for IconClient {
    fn get_http_client(&self) -> &Client {
        &self.client
    }
    
    fn get_api_url(&self) -> &str {
        &self.api_url
    }
}
