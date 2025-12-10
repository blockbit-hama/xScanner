use crate::respository::batch_add_monitored_addresses;
use crate::types::AppError;
use log::{info, warn, error};
use rocksdb::DB;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use aws_sdk_sqs::Client as SqsClient;
use reqwest::Client as HttpClient;

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomerAddressEvent {
    pub event: String, // "AddressAdded"
    pub address: String,
    pub chain: String,
    pub wallet_id: String,
    pub account_id: Option<String>, // None for Omnibus (Master) Address
    pub timestamp: String,
}

/// Configuration for customer address synchronization
pub struct CustomerSyncConfig {
    pub sqs_queue_url: String,
    pub aws_region: String,
    pub batch_size: usize,
    pub flush_interval_secs: u64,
    pub cache_file_path: Option<String>, // e.g., "./customer_addresses.json"
}

impl Default for CustomerSyncConfig {
    fn default() -> Self {
        Self {
            sqs_queue_url: String::new(),
            aws_region: "ap-northeast-2".to_string(),
            batch_size: 100,
            flush_interval_secs: 5,
            cache_file_path: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct CustomerAddressData {
    address: String,
    chain: String,
    wallet_id: String,
    account_id: Option<String>,
}

/// Run customer address synchronization from SQS
///
/// This service listens to SQS messages from the backend when new customer addresses are added,
/// and batch-updates the RocksDB cache for fast lookup during block scanning.
///
/// On startup, it first loads all existing addresses from Backend API (if configured)
/// to handle cases where xScanner was down and missed SQS messages.
pub async fn run_customer_address_sync(
    rocksdb: Arc<DB>,
    config: CustomerSyncConfig,
) {
    info!(
        "[CustomerSync] Starting customer address sync service (batch_size: {}, flush_interval: {}s)",
        config.batch_size, config.flush_interval_secs
    );

    // STEP 1: Startup cache warming from local file (if exists)
    // This handles the case where xScanner was down and missed SQS messages
    if let Some(cache_file) = &config.cache_file_path {
        info!("[CustomerSync] Loading customer addresses from cache file: {}", cache_file);
        match load_addresses_from_file(&rocksdb, cache_file).await {
            Ok(count) => {
                info!("✅ [CustomerSync] Loaded {} customer addresses from file", count);
            }
            Err(e) => {
                warn!("[CustomerSync] Failed to load from cache file: {} (continuing without initial load)", e);
            }
        }
    } else {
        info!("[CustomerSync] No cache file configured, relying on SQS messages only");
    }

    // Channel for batching
    let (sender, mut receiver) = mpsc::channel::<CustomerAddressEvent>(1000);

    // SQS Consumer Task
    let sqs_queue_url = config.sqs_queue_url.clone();
    let aws_region = config.aws_region.clone();
    tokio::spawn(async move {
        let is_local = sqs_queue_url.starts_with("http://localhost") || sqs_queue_url.starts_with("http://127.0.0.1");

        let mut aws_config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_sdk_sqs::config::Region::new(aws_region.clone()));

        // For local development with ElasticMQ
        if is_local {
            info!("[CustomerSync] Using local ElasticMQ endpoint with dummy credentials");
            aws_config_loader = aws_config_loader
                .endpoint_url("http://localhost:9324")
                .credentials_provider(aws_sdk_sqs::config::Credentials::new(
                    "dummy",
                    "dummy",
                    None,
                    None,
                    "static",
                ));
        }

        let aws_config = aws_config_loader.load().await;
        let sqs = SqsClient::new(&aws_config);

        info!("[CustomerSync] SQS Consumer started, queue: {}", sqs_queue_url);

        loop {
            match sqs
                .receive_message()
                .queue_url(&sqs_queue_url)
                .max_number_of_messages(10) // Receive up to 10 messages at once
                .wait_time_seconds(20) // Long polling (reduce empty responses)
                .send()
                .await
            {
                Ok(output) => {
                    if let Some(messages) = output.messages {
                        info!("[CustomerSync] Received {} SQS messages", messages.len());

                        for msg in messages {
                            if let Some(body) = &msg.body {
                                match serde_json::from_str::<CustomerAddressEvent>(body) {
                                    Ok(event) => {
                                        if event.event == "CustomerAddressAdded" {
                                            if sender.send(event).await.is_err() {
                                                error!("[CustomerSync] Failed to send to batch buffer (channel closed)");
                                            }
                                        } else {
                                            warn!("[CustomerSync] Unknown event type: {}", event.event);
                                        }
                                    }
                                    Err(e) => {
                                        error!("[CustomerSync] Failed to parse SQS message: {} | body: {}", e, body);
                                    }
                                }
                            }

                            // Delete message after processing
                            if let Some(receipt_handle) = msg.receipt_handle {
                                if let Err(e) = sqs
                                    .delete_message()
                                    .queue_url(&sqs_queue_url)
                                    .receipt_handle(receipt_handle)
                                    .send()
                                    .await
                                {
                                    warn!("[CustomerSync] Failed to delete SQS message: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("[CustomerSync] SQS receive error: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });

    // Batch Writer Task
    tokio::spawn(async move {
        let mut buffer: Vec<(String, String, String, Option<String>)> = Vec::with_capacity(config.batch_size);
        let mut flush_interval = interval(Duration::from_secs(config.flush_interval_secs));
        flush_interval.tick().await; // Skip first immediate tick

        info!("[CustomerSync] Batch writer started");

        loop {
            tokio::select! {
                // New event received
                Some(event) = receiver.recv() => {
                    buffer.push((
                        event.address.clone(),
                        event.chain.clone(),
                        event.wallet_id.clone(),
                        event.account_id.clone(),
                    ));
                    info!(
                        "[CustomerSync] Buffered: {} (chain: {}, wallet: {}, account: {:?}) | Buffer size: {}/{}",
                        event.address, event.chain, event.wallet_id, event.account_id, buffer.len(), config.batch_size
                    );

                    // Flush when batch size reached
                    if buffer.len() >= config.batch_size {
                        info!("[CustomerSync] Batch size reached, flushing...");
                        flush_batch(&rocksdb, &mut buffer).await;
                    }
                }

                // Timeout flush (ensure small batches are also processed)
                _ = flush_interval.tick() => {
                    if !buffer.is_empty() {
                        info!("[CustomerSync] Flush interval reached, flushing {} items...", buffer.len());
                        flush_batch(&rocksdb, &mut buffer).await;
                    }
                }
            }
        }
    });
}

async fn flush_batch(rocksdb: &Arc<DB>, buffer: &mut Vec<(String, String, String, Option<String>)>) {
    if buffer.is_empty() {
        return;
    }

    let count = buffer.len();
    match batch_add_monitored_addresses(rocksdb, buffer.clone()) {
        Ok(written) => {
            info!("✅ [CustomerSync] Flushed {} monitored addresses to RocksDB cache", written);
            buffer.clear();
        }
        Err(e) => {
            error!("❌ [CustomerSync] Failed to flush batch to RocksDB: {}", e);
            // Clear buffer even on failure to prevent infinite retry
            buffer.clear();
        }
    }
}

/// Load monitored addresses from JSON file
/// File format: [{"address": "0x123", "chain": "ETH"}, ...]
async fn load_addresses_from_file(
    rocksdb: &Arc<DB>,
    file_path: &str,
) -> Result<usize, AppError> {
    use tokio::fs;

    // Check if file exists
    if !std::path::Path::new(file_path).exists() {
        return Ok(0); // File doesn't exist, skip silently
    }

    // Read file
    let content = fs::read_to_string(file_path).await
        .map_err(|e| AppError::Initialization(format!("Failed to read file {}: {}", file_path, e)))?;

    // Parse JSON
    let addresses: Vec<CustomerAddressData> = serde_json::from_str(&content)
        .map_err(|e| AppError::Initialization(format!("Failed to parse JSON from {}: {}", file_path, e)))?;

    if addresses.is_empty() {
        info!("[CustomerSync] Cache file is empty, nothing to load");
        return Ok(0);
    }

    // Convert to batch format
    let batch_data: Vec<(String, String, String, Option<String>)> = addresses
        .into_iter()
        .map(|addr| (addr.address, addr.chain, addr.wallet_id, addr.account_id))
        .collect();

    let count = batch_data.len();

    // Write to RocksDB
    batch_add_monitored_addresses(rocksdb, batch_data)?;

    Ok(count)
}
