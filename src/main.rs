// src/main.rs
/**
* author : HAMA
* date: 2025. 4. 6.
* description: Entry point for the blockchain event processing service.
*/

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
use crate::coin::tron::client::TronClient;
use crate::coin::theta::client::ThetaClient;
use crate::coin::icon::client::IconClient;
use crate::config::Settings;
use crate::types::{AppError, BlockData};
use crate::fetcher::bitcoin_fetcher::BitcoinFetcher;
use crate::fetcher::ethereum_fetcher::EthereumFetcher;
use crate::fetcher::tron_fetcher::TronFetcher;
use crate::fetcher::theta_fetcher::ThetaFetcher;
use crate::fetcher::icon_fetcher::IconFetcher;

use crate::respository::{
    Repository,
    RepositoryWrapper,
};
use crate::analyzer::analyzer::KeyValueDB;

#[cfg(feature = "leveldb-backend")]
use crate::respository::{load_customer_addresses_to_leveldb, open_leveldb, batch_add_customer_addresses as leveldb_batch_add};

#[cfg(feature = "rocksdb-backend")]
use crate::respository::{open_rocksdb, batch_add_customer_addresses as rocksdb_batch_add};
use crate::shutdown::shutdown_signal;
use crate::tasks::spawn_fetcher;

use log::{error, info, warn};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 1. Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Application starting...");
    
    // 2. Load configuration
    let settings = Settings::new().map_err(|e| AppError::Config(e.to_string()))?;
    info!("Configuration loaded.");
    
    // 3. Initialize Repository based on configuration
    let repository = Arc::new(RepositoryWrapper::from_settings(&settings).await?);
    if settings.repository.memory_db {
        info!("Using MemoryRepository (memory_db = true)");
    } else {
        info!("Using PostgreSQLRepository (memory_db = false)");
    }
    
    // 4. Open KeyValueDB for fast address lookups (RocksDB or LevelDB)
    let kv_db: Option<Arc<KeyValueDB>> = if !settings.repository.memory_db {
        #[cfg(feature = "leveldb-backend")]
        {
            let db = Arc::new(open_leveldb(&settings.repository.leveldb_path)?);
            info!("Opened LevelDB for customer address caching.");

            // Load customer addresses from PostgreSQL to LevelDB
            if let Some(pg_repo) = repository.get_postgresql_repo() {
                let chain_configs = settings.get_chain_configs();
                for (chain_name, _chain_config) in &chain_configs {
                    let chain_symbol = chain_name.to_uppercase();
                    match load_customer_addresses_to_leveldb(
                        pg_repo.pool(),
                        &db,
                        &chain_symbol,
                    ).await {
                        Ok(count) => {
                            info!("Loaded {} customer addresses for {} from PostgreSQL to LevelDB cache", count, chain_symbol);
                        }
                        Err(e) => {
                            warn!("Failed to load customer addresses for {}: {}", chain_symbol, e);
                        }
                    }
                }
            }

            Some(db)
        }

        #[cfg(feature = "rocksdb-backend")]
        {
            let db = Arc::new(open_rocksdb(&settings.repository.leveldb_path)?);
            info!("Opened RocksDB for customer address caching.");

            // Load customer addresses from PostgreSQL to RocksDB
            if let Some(pg_repo) = repository.get_postgresql_repo() {
                use crate::respository::load_customer_addresses_to_rocksdb;
                let chain_configs = settings.get_chain_configs();
                for (chain_name, _chain_config) in &chain_configs {
                    let chain_symbol = chain_name.to_uppercase();
                    match load_customer_addresses_to_rocksdb(
                        pg_repo.pool(),
                        &db,
                        &chain_symbol,
                    ).await {
                        Ok(count) => {
                            info!("Loaded {} customer addresses for {} from PostgreSQL to RocksDB cache", count, chain_symbol);
                        }
                        Err(e) => {
                            warn!("Failed to load customer addresses for {}: {}", chain_symbol, e);
                        }
                    }
                }
            }

            Some(db)
        }

        #[cfg(not(any(feature = "leveldb-backend", feature = "rocksdb-backend")))]
        {
            warn!("No KeyValueDB feature enabled. Using PostgreSQL only.");
            None
        }
    } else {
        None
    };
    
    // 5. Create channel for blocks
    let (sender, receiver) = mpsc::channel::<BlockData>(128);
    
    // 6. Dynamically spawn fetchers for all configured chains
    let mut fetcher_handles: Vec<JoinHandle<()>> = Vec::new();
    let chain_configs = settings.get_chain_configs();
    
    info!("Found {} blockchain(s) to monitor", chain_configs.len());
    
    for (chain_name, chain_config) in chain_configs {
        let chain_symbol = chain_config.symbol.to_uppercase();
        let sender_clone = sender.clone();
        
        // ???: ??? ?? ?? ?? (?? ??? ?)
        if let Err(e) = repository.init_last_processed_block(&chain_symbol, chain_config.start_block).await {
            warn!("Failed to initialize last processed block for {}: {}", chain_symbol, e);
        }
        
        // Get last processed block for this chain
        let start_block = match repository.get_last_processed_block(&chain_symbol).await {
            Ok(block) => {
                let next_block = block + 1;
                info!("{} last processed block: {}, starting from block {}", chain_symbol, block, next_block);
                next_block
            },
            Err(e) => {
                warn!("Failed to get last processed block for {}, using start_block: {}", chain_symbol, e);
                chain_config.start_block
            }
        };
        
        // ?? start_block?? ??? start_block?? ??
        let start_block = if start_block < chain_config.start_block {
            chain_config.start_block
        } else {
            start_block
        };
        
        info!("Initializing {} scanner from block {}", chain_name, start_block);
        
        // Spawn fetcher based on chain name
        let handle = match chain_name.to_lowercase().as_str() {
            "ethereum" | "eth" => {
                let client = Arc::new(EthereumClient::new(chain_config.api.clone()));
                let fetcher = Arc::new(EthereumFetcher { client });
                spawn_fetcher(fetcher, sender_clone, start_block, chain_config.interval_secs)
            }
            "bitcoin" | "btc" => {
                let client = Arc::new(BitcoinClient::new(chain_config.api.clone()));
                let fetcher = Arc::new(BitcoinFetcher { client });
                spawn_fetcher(fetcher, sender_clone, start_block, chain_config.interval_secs)
            }
            "tron" => {
                let client = Arc::new(TronClient::new(chain_config.api.clone()));
                let fetcher = Arc::new(TronFetcher { client });
                spawn_fetcher(fetcher, sender_clone, start_block, chain_config.interval_secs)
            }
            "theta" => {
                let client = Arc::new(ThetaClient::new(chain_config.api.clone()));
                let fetcher = Arc::new(ThetaFetcher { client });
                spawn_fetcher(fetcher, sender_clone, start_block, chain_config.interval_secs)
            }
            "icon" => {
                let client = Arc::new(IconClient::new(chain_config.api.clone()));
                let fetcher = Arc::new(IconFetcher { client });
                spawn_fetcher(fetcher, sender_clone, start_block, chain_config.interval_secs)
            }
            _ => {
                warn!("Unknown blockchain: {}, skipping...", chain_name);
                continue;
            }
        };
        
        fetcher_handles.push(handle);
    }
    
    // 7. Spawn analyzer
    let analyzer_handle = tokio::spawn(analyzer::run_analyzer(
        receiver,
        repository.clone(),
        kv_db,
    ));
    
    // 8. Wait for shutdown signal
    shutdown_signal().await;
    info!("Shutdown signal received. Waiting for tasks to finish...");
    
    // 9. Gracefully wait for all fetchers
    for handle in fetcher_handles {
        let _ = handle.await;
    }
    let _ = analyzer_handle.await;
    
    info!("Application exited cleanly.");
    Ok(())
}
