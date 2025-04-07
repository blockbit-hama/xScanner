/**
* filename : interface
* author : HAMA
* date: 2025. 4. 6.
* description: 
**/

use async_trait::async_trait;
use crate::types::{BlockData, AppError};

#[async_trait]
pub trait BlockFetcher: Send + Sync {
  async fn fetch_block(&self, block_number: u64) -> Result<BlockData, AppError>;
  // fn extract_block_number(&self, block: &BlockData) -> Result<u64, AppError>;
  fn chain_name(&self) -> &'static str;
}
