// src/main.rs
/**
* author : HAMA
* date: 2025. 4. 6.
* description: Entry point for the blockchain event processing service.
**/

mod coin;
mod config;
mod fetcher;
mod types;
mod respository;
mod analyzer;
mod shutdown;
mod tasks;

use crate::coin::bitcoin::client::BitcoinClient;
use crate::coin::ethereum::client::EthereumClient;
use crate::config::Settings;
use crate::types::{AppError, BlockData};
use crate::fetcher::bitcoin_fetcher::BitcoinFetcher;
use crate::fetcher::ethereum_fetcher::EthereumFetcher;


use crate::respository::{
    add_address_to_leveldb, connect_db, get_last_processed_block, open_leveldb, setup_db_schema,
};
use crate::shutdown::shutdown_signal;
use crate::tasks::spawn_fetcher;

use log::{error, info};
use std::sync::Arc;
use std::{fs::File, io::{BufRead, BufReader}};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 1. Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Application starting...");
    
    // 2. Load configuration
    let settings = Settings::new().map_err(|e| AppError::Config(e.to_string()))?;
    info!("Configuration loaded.");
    
    // 3. Connect to PostgreSQL
    let db_connection_pool = connect_db(&settings.repository.postgresql_url).await.unwrap();
    setup_db_schema(&db_connection_pool).await;
    let db_connection_pool = Arc::new(db_connection_pool);
    
    // 4. Open LevelDB
    let level_db = Arc::new(open_leveldb(&settings.repository.leveldb_path)?);
    info!("Opened LevelDB.");
    
    // 5. Load customer addresses
    let address_file = File::open(&settings.repository.customer_address_file)?;
    let reader = BufReader::new(address_file);
    let mut loaded_count = 0;
    
    for line in reader.lines() {
        if let Ok(address) = line {
            let address = address.trim();
            if !address.is_empty() {
                if add_address_to_leveldb(&level_db, address).is_ok() {
                    loaded_count += 1;
                }
            }
        }
    }
    
    info!("Loaded {} addresses into LevelDB", loaded_count);
    
    // 6. Create API clients
    let ethereum_client = Arc::new(EthereumClient::new(settings.blockchain.ethereum.api.clone()));
    let bitcoin_client = Arc::new(BitcoinClient::new(settings.blockchain.bitcoin.api.clone()));
    
    // 7. Create channel for blocks
    let (sender, receiver) = mpsc::channel::<BlockData>(128);
    
    // 8. Fetch last processed blocks
    let eth_start_block = get_last_processed_block(&db_connection_pool, "ETH").await? + 1;
    let btc_start_block = get_last_processed_block(&db_connection_pool, "BTC").await? + 1;
    
    // 9. Spawn fetchers
    let eth_fetcher = Arc::new(EthereumFetcher { client: ethereum_client });
    let btc_fetcher = Arc::new(BitcoinFetcher { client: bitcoin_client });
    
    let eth_handle = spawn_fetcher(
        eth_fetcher,
        sender.clone(),
        eth_start_block,
        settings.blockchain.ethereum.interval_secs,
    );
    
    let btc_handle = spawn_fetcher(
        btc_fetcher,
        sender,
        btc_start_block,
        settings.blockchain.bitcoin.interval_secs,
    );
    
    // 10. Spawn analyzer
    let analyzer_handle = tokio::spawn(analyzer::run_analyzer(
        receiver,
        db_connection_pool.clone(),
        level_db.clone(),
    ));
    
    // 11. Wait for shutdown signal
    shutdown_signal().await;
    info!("Shutdown signal received. Waiting for tasks to finish...");
    
    // 12. Gracefully wait
    let _ = eth_handle.await;
    let _ = btc_handle.await;
    let _ = analyzer_handle.await;
    
    info!("Application exited cleanly.");
    Ok(())
}
