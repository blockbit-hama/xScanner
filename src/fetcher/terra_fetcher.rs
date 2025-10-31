use crate::coin::terra::client::TerraClient;
use crate::types::{BlockData, AppError};
use crate::fetcher::fetcher::BlockFetcher;
use async_trait::async_trait;
use std::sync::Arc;

pub struct TerraFetcher {
  pub client: Arc<TerraClient>,
}

#[async_trait]
impl BlockFetcher for TerraFetcher {
  async fn fetch_block(&self, block_number: u64) -> Result<BlockData, AppError> {
    let block = self.client.fetch_block_by_number(block_number).await
      .map_err(|e| AppError::Client(format!("Failed to fetch TERRA block: {}", e)))?;
    Ok(BlockData::Terra(block))
  }

  fn chain_name(&self) -> &'static str {
    "TERRA"
  }
}
