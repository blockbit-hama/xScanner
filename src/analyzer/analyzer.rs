/**
* author : HAMA
* date: 2025. 4. 5.
* description:
**/

use leveldb::database::Database;
use sqlx::PgPool;
use std::sync::Arc;
use log::{error, info};
use sqlx::__rt::spawn_blocking;
use tokio::sync::mpsc::Receiver;

use crate::respository::check_address_exists_in_leveldb;
use crate::respository::update_last_processed_block;
use crate::types::BlockData;

pub async fn run_analyzer(
  mut receiver: Receiver<BlockData>,
  postgresql_connection_pool: Arc<PgPool>,
  level_db: Arc<Database<i32>>,
) {
  info!("[Analyzer] Starting loop...");
  
  while let Some(block_data) = receiver.recv().await {
    let pool_clone = postgresql_connection_pool.clone();
    let level_db_clone = level_db.clone();
    
    let result = spawn_blocking(move || -> Result<(String, u64), String> {
      let mut chain_name: &str = "";
      let mut block_number: u64 = 0;
      let addresses_to_check: Vec<String>;
      
      // --- Extract block info and addresses ---
      match block_data {
        BlockData::Ethereum(block) => {
          chain_name = "ETH";
          let result = match &block.result {
            Some(res) => res,
            None => return Err("Missing 'result' in EthereumBlock".into()),
          };
          
          let number_hex = result.number.trim();
          block_number = if number_hex.starts_with("0x") {
            u64::from_str_radix(&number_hex[2..], 16)
          } else {
            number_hex.parse::<u64>()
          }.map_err(|e| format!("Failed to parse ETH block number {}: {}", number_hex, e))?;
          
          info!("[Analyzer] Ethereum Block Received: {}", block_number);
          
          addresses_to_check = result.transactions.iter()
            .filter_map(|tx| tx.to.clone())
            .collect();
          
          info!("[Analyzer] Processing ETH block: {}", block_number);
        }
        
        BlockData::Bitcoin(block) => {
          chain_name = "BTC";
          block_number = block.height;
          info!("[Analyzer] Bitcoin Block Received: {}", block_number);
          
          addresses_to_check = block.tx.iter()
            .flat_map(|tx| tx.out.iter().filter_map(|out| out.addr.clone()))
            .collect();
          
          info!("[Analyzer] Processing BTC block: {}", block_number);
        }
      }
      
      // --- Check addresses against LevelDB ---
      for address in &addresses_to_check {
        match check_address_exists_in_leveldb(&level_db_clone, address) {
          Ok(true) => {
            info!("[ALERT] Monitored address {} found in {} block {}", address, chain_name, block_number);
          }
          Ok(false) => {}
          Err(e) => {
            error!("[Analyzer] LevelDB check failed for address {}: {}", address, e);
          }
        }
      }
      
      // --- Simulate work ---
      std::thread::sleep(std::time::Duration::from_millis(50));
      
      // --- Return result ---
      Ok((chain_name.to_string(), block_number))
    }).await;
    
    // --- Handle result ---
    match result {
      Ok((chain_name_processed, block_num_processed)) => {
        info!("[Analyzer] Finished processing {} block {}", chain_name_processed, block_num_processed);
        let update_res = update_last_processed_block(
          &pool_clone,
          &chain_name_processed,
          block_num_processed,
        ).await;
        
        if let Err(e) = update_res {
          error!(
            "[Analyzer] Failed to update last processed block in DB for {} block {}: {}",
            chain_name_processed, block_num_processed, e
          );
        } else {
          info!(
            "[Analyzer] Updated last processed block in DB for {} to {}",
            chain_name_processed, block_num_processed
          );
        }
      }
      Err(join_err) => {
        error!("[Analyzer] Blocking task failed (panic?): {}", join_err);
      }
      _ => {}
    }
  }
  
  info!("[Analyzer] Loop finished because the channel was closed.");
}
