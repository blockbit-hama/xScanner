use crate::coin::tezos::client::TezosClient;
use crate::types::{BlockData, AppError};
use crate::fetcher::fetcher::BlockFetcher;
use async_trait::async_trait;
use std::sync::Arc;

pub struct TezosFetcher {
  pub client: Arc<TezosClient>,
}

#[async_trait]
impl BlockFetcher for TezosFetcher {
  async fn fetch_block(&self, block_number: u64) -> Result<BlockData, AppError> {
    let block = self.client.fetch_block_by_number(block_number).await
      .map_err(|e| AppError::Client(format!("Failed to fetch TEZOS block: {}", e)))?;
    Ok(BlockData::Tezos(block))
  }

  fn chain_name(&self) -> &'static str {
    "TEZOS"
  }
}
