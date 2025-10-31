use crate::coin::icon::client::IconClient;
use crate::types::{BlockData, AppError};
use crate::fetcher::fetcher::BlockFetcher;
use async_trait::async_trait;
use std::sync::Arc;

pub struct IconFetcher {
    pub client: Arc<IconClient>,
}

#[async_trait]
impl BlockFetcher for IconFetcher {
    async fn fetch_block(&self, block_number: u64) -> Result<BlockData, AppError> {
        let block = self.client.fetch_block_by_number(block_number).await
            .map_err(|e| AppError::Client(format!("Failed to fetch ICON block: {}", e)))?;
        Ok(BlockData::Icon(block))
    }
    
    fn chain_name(&self) -> &'static str {
        "ICON"
    }
}
