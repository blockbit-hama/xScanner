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

    info!("===============================================");
    info!("[{} Fetcher] 블록 #{} 가져오는 중...", fetcher.chain_name(), block_to_fetch);

    match fetcher.fetch_block(block_to_fetch).await {
      Ok(block_data) => {
        info!("[{} Fetcher] ✅ 블록 #{} 가져오기 성공!", fetcher.chain_name(), block_to_fetch);
        if let Err(e) = sender.send(block_data).await {
          error!(
                  "[{} Fetcher] Failed to send block {}: {}",
                  fetcher.chain_name(),
                  block_to_fetch,
                  e
              );
        } else {
          info!("[{} Fetcher] 블록 #{} Analyzer로 전송 완료", fetcher.chain_name(), block_to_fetch);
        }
        current_block_number += 1;
      }
      Err(e) => {
        let retry_delay = interval_duration / 2;
        warn!(
          "[{} Fetcher] ⏳ 블록 #{} 가져오기 실패: {} | {:?} 후 재시도...",
          fetcher.chain_name(),
          block_to_fetch,
          e,
          retry_delay
        );
        // 블록 번호를 증가시키지 않고 interval의 절반 시간 후 재시도
        tokio::time::sleep(retry_delay).await;
      }
    }
  }
  
  warn!("[{} Fetcher] Loop exited.", fetcher.chain_name());
}
