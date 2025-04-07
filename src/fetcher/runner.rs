/**
* filename : runner
* author : HAMA
* date: 2025. 4. 6.
* description: 
**/

use crate::fetcher::fetcher::BlockFetcher;
use crate::types::BlockSender;

use log::{info, warn, error};
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub async fn run_fetcher<F: BlockFetcher + 'static>(
  fetcher: Arc<F>,
  sender: BlockSender,
  mut current_block_number: u64,
  interval_duration: Duration,
) {
  let mut tick = interval(interval_duration);
  info!(
        "[{} Fetcher] Starting from block {} with interval {:?}",
        fetcher.chain_name(),
        current_block_number,
        interval_duration
    );
  
  loop {
    tick.tick().await;
    let block_to_fetch = current_block_number;
    
    match fetcher.fetch_block(block_to_fetch).await {
      Ok(block_data) => {
        if let Err(e) = sender.send(block_data).await {
          error!(
                  "[{} Fetcher] Failed to send block {}: {}",
                  fetcher.chain_name(),
                  block_to_fetch,
                  e
              );
        }
        current_block_number += 1;
      }
      Err(e) => {
        warn!(
              "[{} Fetcher] Error fetching block {}: {}",
              fetcher.chain_name(),
              block_to_fetch,
              e
          );
      }
    }
  }
  
  warn!("[{} Fetcher] Loop exited.", fetcher.chain_name());
}
