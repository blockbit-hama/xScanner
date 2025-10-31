/**
 * filename: aion_fetcher
 * author: HAMA
 * date: 2025. 10. 31.
 * description: AION blockchain fetcher
 **/

use crate::coin::aion::client::AionClient;
use crate::types::{BlockData, AppError};
use crate::fetcher::fetcher::BlockFetcher;

use async_trait::async_trait;
use std::sync::Arc;

pub struct AionFetcher {
  pub client: Arc<AionClient>,
}

#[async_trait]
impl BlockFetcher for AionFetcher {
  async fn fetch_block(&self, block_number: u64) -> Result<BlockData, AppError> {
    let block = self.client.fetch_block_by_number(block_number).await
      .map_err(|e| AppError::Client(format!("Failed to fetch AION block: {}", e)))?;
    Ok(BlockData::Aion(block))
  }

  fn chain_name(&self) -> &'static str {
    "AION"
  }
}
