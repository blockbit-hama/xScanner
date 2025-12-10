/**
* filename : ethereum
* author : HAMA
* date: 2025. 4. 6.
* description:
**/

use crate::coin::ethereum::client::EthereumClient;
use crate::types::{BlockData, AppError};
use crate::fetcher::fetcher::BlockFetcher;

use async_trait::async_trait;
use std::sync::Arc;

pub struct EthereumFetcher {
  pub client: Arc<EthereumClient>,
}

#[async_trait]
impl BlockFetcher for EthereumFetcher {
  async fn fetch_block(&self, block_number: u64) -> Result<BlockData, AppError> {
    let block = self.client.fetch_block_by_number(block_number).await
      .map_err(|e| AppError::Client(format!("Failed to fetch ETH block: {}", e)))?;
    Ok(BlockData::Ethereum(block))
  }
  
  // fn extract_block_number(&self, block: &BlockData) -> Result<u64, AppError> {
  //   if let BlockData::Ethereum(eth_block) = block {
  //     let number_hex = eth_block.result
  //       .as_ref()
  //       .ok_or(AppError::Block("Missing result".to_string()))?
  //       .number
  //       .trim();
  //
  //     let number = if number_hex.starts_with("0x") {
  //       u64::from_str_radix(&number_hex[2..], 16)
  //     } else {
  //       number_hex.parse::<u64>()
  //     };
  //
  //     number.map_err(|e| AppError::Block(format!("ETH block number parse error: {}", e)))
  //   } else {
  //     Err(AppError::Block("Invalid block type for Ethereum".into()))
  //   }
  // }
  
  fn chain_name(&self) -> &'static str {
    "SEPOLIA"
  }
}
