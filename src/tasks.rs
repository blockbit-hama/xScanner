/**
* filename : tasks
* author : HAMA
* date: 2025. 4. 6.
* description: 
**/
use std::{sync::Arc, time::Duration};
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use crate::types::BlockData;
use crate::fetcher::fetcher::BlockFetcher;
use crate::fetcher::runner::run_fetcher;

pub fn spawn_fetcher<F: BlockFetcher + Send + Sync + 'static>(
  fetcher: Arc<F>,
  sender: Sender<BlockData>,
  start_block: u64,
  interval_secs: u64,
) -> JoinHandle<()> {
  tokio::spawn(run_fetcher(
    fetcher,
    sender,
    start_block,
    Duration::from_secs(interval_secs),
  ))
}
