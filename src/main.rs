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
mod notification;

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
use crate::notification::sqs_client::SqsNotifier;

#[cfg(feature = "leveldb-backend")]
use crate::respository::open_leveldb;

#[cfg(feature = "rocksdb-backend")]
use crate::respository::open_rocksdb;
use crate::shutdown::shutdown_signal;

use log::{error, info, warn};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::Duration;

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
            info!("Note: Customer addresses will be loaded via SQS sync + optional cache file");
            Some(db)
        }

        #[cfg(feature = "rocksdb-backend")]
        {
            let db = Arc::new(open_rocksdb(&settings.repository.leveldb_path)?);
            info!("Opened RocksDB for customer address caching.");
            info!("Note: Customer addresses will be loaded via SQS sync + optional cache file");
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
        
        let interval_duration = Duration::from_secs(chain_config.interval_secs);

        // Spawn fetcher based on chain name
        let handle = match chain_name.to_lowercase().as_str() {
            "ethereum" | "eth" | "sepolia" => {
                let client = Arc::new(EthereumClient::new(chain_config.api.clone()));
                let fetcher = Arc::new(EthereumFetcher { client });
                tokio::spawn(crate::fetcher::runner::run_fetcher(fetcher, sender_clone, start_block, interval_duration))
            }
            "bitcoin" | "btc" => {
                let client = Arc::new(BitcoinClient::new(chain_config.api.clone()));
                let fetcher = Arc::new(BitcoinFetcher { client });
                tokio::spawn(crate::fetcher::runner::run_fetcher(fetcher, sender_clone, start_block, interval_duration))
            }
            "tron" => {
                let client = Arc::new(TronClient::new(chain_config.api.clone()));
                let fetcher = Arc::new(TronFetcher { client });
                tokio::spawn(crate::fetcher::runner::run_fetcher(fetcher, sender_clone, start_block, interval_duration))
            }
            "theta" => {
                let client = Arc::new(ThetaClient::new(chain_config.api.clone()));
                let fetcher = Arc::new(ThetaFetcher { client });
                tokio::spawn(crate::fetcher::runner::run_fetcher(fetcher, sender_clone, start_block, interval_duration))
            }
            "icon" => {
                let client = Arc::new(IconClient::new(chain_config.api.clone()));
                let fetcher = Arc::new(IconFetcher { client });
                tokio::spawn(crate::fetcher::runner::run_fetcher(fetcher, sender_clone, start_block, interval_duration))
            }
            _ => {
                warn!("Unknown blockchain: {}, skipping...", chain_name);
                continue;
            }
        };
        
        fetcher_handles.push(handle);
    }
    
    // 7. Initialize SQS Notifier (if configured)
    let sqs_notifier = if let Some(notification_config) = &settings.notification {
        match SqsNotifier::new(
            notification_config.sqs_queue_url.clone(),
            notification_config.aws_region.clone(),
        ).await {
            Ok(notifier) => {
                info!("SQS Notifier initialized: {}", notification_config.sqs_queue_url);
                Some(Arc::new(notifier))
            }
            Err(e) => {
                warn!("Failed to initialize SQS Notifier: {}", e);
                None
            }
        }
    } else {
        info!("SQS Notifier not configured, skipping");
        None
    };

    // 8. Spawn customer address sync task (if configured)
    if let Some(customer_sync_config) = &settings.customer_sync {
        if let Some(kv_db_ref) = &kv_db {
            info!("Starting customer address sync service...");
            let sync_config = crate::tasks::CustomerSyncConfig {
                sqs_queue_url: customer_sync_config.sqs_queue_url.clone(),
                aws_region: customer_sync_config.aws_region.clone(),
                batch_size: customer_sync_config.batch_size,
                flush_interval_secs: customer_sync_config.flush_interval_secs,
                cache_file_path: customer_sync_config.cache_file_path.clone(),
            };
            crate::tasks::run_customer_address_sync(kv_db_ref.clone(), sync_config).await;
        } else {
            warn!("Customer sync configured but no RocksDB available, skipping");
        }
    }

    // 8.5. Spawn confirmation checker task (if configured)
    let confirmation_checker_handle = if let Some(confirmation_checker_config) = &settings.confirmation_checker {
        let checker_config = crate::tasks::ConfirmationCheckerConfig {
            enabled: confirmation_checker_config.enabled,
            check_interval_secs: confirmation_checker_config.check_interval_secs,
        };

        let chain_configs_map: std::collections::HashMap<String, config::ChainConfig> =
            settings.get_chain_configs().into_iter().collect();

        Some(tokio::spawn(crate::tasks::run_confirmation_checker(
            repository.clone(),
            chain_configs_map,
            sqs_notifier.clone(),
            checker_config,
        )))
    } else {
        info!("Confirmation checker not configured, using defaults");
        let chain_configs_map: std::collections::HashMap<String, config::ChainConfig> =
            settings.get_chain_configs().into_iter().collect();

        Some(tokio::spawn(crate::tasks::run_confirmation_checker(
            repository.clone(),
            chain_configs_map,
            sqs_notifier.clone(),
            crate::tasks::ConfirmationCheckerConfig::default(),
        )))
    };

    // 9. Spawn analyzer
    let analyzer_handle = tokio::spawn(analyzer::run_analyzer(
        receiver,
        repository.clone(),
        kv_db,
        sqs_notifier,
        settings.get_chain_configs().into_iter().collect(),
    ));

    // 11. Wait for shutdown signal
    shutdown_signal().await;
    info!("Shutdown signal received. Waiting for tasks to finish...");

    // 12. Gracefully wait for all fetchers
    for handle in fetcher_handles {
        let _ = handle.await;
    }
    let _ = analyzer_handle.await;
    if let Some(handle) = confirmation_checker_handle {
        let _ = handle.await;
    }

    info!("Application exited cleanly.");
    Ok(())
}
