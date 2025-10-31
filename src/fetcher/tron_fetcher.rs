use crate::coin::tron::client::TronClient;
use crate::types::{BlockData, AppError};
use crate::fetcher::fetcher::BlockFetcher;
use async_trait::async_trait;
use std::sync::Arc;

pub struct TronFetcher {
    pub client: Arc<TronClient>,
}

#[async_trait]
impl BlockFetcher for TronFetcher {
    async fn fetch_block(&self, block_number: u64) -> Result<BlockData, AppError> {
        let block = self.client.fetch_block_by_number(block_number).await?;
        Ok(BlockData::Tron(block))
    }
    
    fn chain_name(&self) -> &'static str {
        "TRON"
    }
}
