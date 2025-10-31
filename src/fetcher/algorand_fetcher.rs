/**
 * filename: algorand_fetcher
 * author: HAMA
 * date: 2025. 10. 31.
 * description: ALGORAND blockchain fetcher
 **/

use crate::coin::algorand::client::AlgorandClient;
use crate::types::{BlockData, AppError};
use crate::fetcher::fetcher::BlockFetcher;

use async_trait::async_trait;
use std::sync::Arc;

pub struct AlgorandFetcher {
  pub client: Arc<AlgorandClient>,
}

#[async_trait]
impl BlockFetcher for AlgorandFetcher {
  async fn fetch_block(&self, block_number: u64) -> Result<BlockData, AppError> {
    let block = self.client.fetch_block_by_number(block_number).await
      .map_err(|e| AppError::Client(format!("Failed to fetch ALGORAND block: {}", e)))?;
    Ok(BlockData::Algorand(block))
  }

  fn chain_name(&self) -> &'static str {
    "ALGORAND"
  }
}
