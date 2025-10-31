use crate::coin::gxchain::client::GxchainClient;
use crate::types::{BlockData, AppError};
use crate::fetcher::fetcher::BlockFetcher;
use async_trait::async_trait;
use std::sync::Arc;

pub struct GxchainFetcher {
  pub client: Arc<GxchainClient>,
}

#[async_trait]
impl BlockFetcher for GxchainFetcher {
  async fn fetch_block(&self, block_number: u64) -> Result<BlockData, AppError> {
    let block = self.client.fetch_block_by_number(block_number).await
      .map_err(|e| AppError::Client(format!("Failed to fetch GXCHAIN block: {}", e)))?;
    Ok(BlockData::Gxchain(block))
  }

  fn chain_name(&self) -> &'static str {
    "GXCHAIN"
  }
}
