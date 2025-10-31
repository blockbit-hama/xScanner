use crate::coin::quark::client::QuarkClient;
use crate::types::{BlockData, AppError};
use crate::fetcher::fetcher::BlockFetcher;
use async_trait::async_trait;
use std::sync::Arc;

pub struct QuarkFetcher {
  pub client: Arc<QuarkClient>,
}

#[async_trait]
impl BlockFetcher for QuarkFetcher {
  async fn fetch_block(&self, block_number: u64) -> Result<BlockData, AppError> {
    let block = self.client.fetch_block_by_number(block_number).await
      .map_err(|e| AppError::Client(format!("Failed to fetch QUARK block: {}", e)))?;
    Ok(BlockData::Quark(block))
  }

  fn chain_name(&self) -> &'static str {
    "QUARK"
  }
}
