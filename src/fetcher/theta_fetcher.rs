use crate::coin::theta::client::ThetaClient;
use crate::types::{BlockData, AppError};
use crate::fetcher::fetcher::BlockFetcher;
use async_trait::async_trait;
use std::sync::Arc;

pub struct ThetaFetcher {
    pub client: Arc<ThetaClient>,
}

#[async_trait]
impl BlockFetcher for ThetaFetcher {
    async fn fetch_block(&self, block_number: u64) -> Result<BlockData, AppError> {
        let block = self.client.fetch_block_by_number(block_number).await
            .map_err(|e| AppError::Client(format!("Failed to fetch THETA block: {}", e)))?;
        Ok(BlockData::Theta(block))
    }
    
    fn chain_name(&self) -> &'static str {
        "THETA"
    }
}
