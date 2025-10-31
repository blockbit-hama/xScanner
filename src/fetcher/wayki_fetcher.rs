use crate::coin::wayki::client::WaykiClient;
use crate::types::{BlockData, AppError};
use crate::fetcher::fetcher::BlockFetcher;
use async_trait::async_trait;
use std::sync::Arc;

pub struct WaykiFetcher {
  pub client: Arc<WaykiClient>,
}

#[async_trait]
impl BlockFetcher for WaykiFetcher {
  async fn fetch_block(&self, block_number: u64) -> Result<BlockData, AppError> {
    let block = self.client.fetch_block_by_number(block_number).await
      .map_err(|e| AppError::Client(format!("Failed to fetch WAYKI block: {}", e)))?;
    Ok(BlockData::Wayki(block))
  }

  fn chain_name(&self) -> &'static str {
    "WAYKI"
  }
}
