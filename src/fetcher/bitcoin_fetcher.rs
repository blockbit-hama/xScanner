/**
* filename : bitcoin
* author : HAMA
* date: 2025. 4. 6.
* description:
**/

use crate::fetcher::fetcher::BlockFetcher;
use crate::coin::bitcoin::client::BitcoinClient;
use crate::types::{BlockData, AppError};


use async_trait::async_trait;
use std::sync::Arc;

pub struct BitcoinFetcher {
  pub client: Arc<BitcoinClient>,
}

#[async_trait]
impl BlockFetcher for BitcoinFetcher {
  async fn fetch_block(&self, block_number: u64) -> Result<BlockData, AppError> {
    let block = self.client.fetch_block_by_number(block_number).await?;
    Ok(BlockData::Bitcoin(block))
  }
  
  // fn extract_block_number(&self, block: &BlockData) -> Result<u64, AppError> {
  //   if let BlockData::Bitcoin(btc_block) = block {
  //     Ok(btc_block.height)
  //   } else {
  //     Err(AppError::Block("Invalid block type for Bitcoin".into()))
  //   }
  // }
  
  fn chain_name(&self) -> &'static str {
    "BTC"
  }
}
