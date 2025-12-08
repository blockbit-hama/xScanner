use super::chains::DepositInfo;
use crate::respository::{Repository, RepositoryWrapper};
use crate::config::ChainConfig;
use crate::notification::sqs_client::SqsNotifier;
use crate::types::BlockData;
use std::sync::Arc;
use std::collections::HashMap;
use log::{error, info};
use tokio::sync::mpsc::Receiver;
use rust_decimal::Decimal;

#[cfg(feature = "rocksdb-backend")]
use crate::respository::is_monitored_address_in_rocksdb;
#[cfg(feature = "rocksdb-backend")]
use rocksdb::DB as RocksDB;

#[cfg(feature = "leveldb-backend")]
use leveldb::database::Database as LevelDB;

// Unified database type for conditional compilation
#[cfg(feature = "leveldb-backend")]
pub type KeyValueDB = LevelDB;

#[cfg(feature = "rocksdb-backend")]
pub type KeyValueDB = RocksDB;

#[cfg(not(any(feature = "leveldb-backend", feature = "rocksdb-backend")))]
pub type KeyValueDB = ();

/// Main analyzer loop - receives blocks and processes them
pub async fn run_analyzer(
    mut receiver: Receiver<BlockData>,
    repository: Arc<RepositoryWrapper>,
    kv_db: Option<Arc<KeyValueDB>>,
    sqs_notifier: Option<Arc<SqsNotifier>>,
    chain_configs: HashMap<String, ChainConfig>,
) {
    info!("[Analyzer] Starting loop...");

    while let Some(block_data) = receiver.recv().await {
        info!("[Analyzer] 블록 데이터 수신! 분석 시작...");
        let repository_clone = repository.clone();
        let kv_db_clone = kv_db.clone();
        let sqs_clone = sqs_notifier.clone();

        // 블록 분석 및 주소 매칭
        let result = analyze_block(block_data, &repository_clone, kv_db_clone.as_deref()).await;

        match result {
            Ok((chain_name, block_number, deposits)) => {
                info!(
                    "[Analyzer] Finished processing {} block {}, found {} deposits",
                    chain_name, block_number, deposits.len()
                );

                // Get chain config for required confirmations
                let required_confirmations = chain_configs.get(&chain_name.to_uppercase())
                    .or_else(|| chain_configs.get(&chain_name.to_lowercase()))
                    .map(|c| c.required_confirmations)
                    .unwrap_or(12); // Default to 12 if not found

                // 입금 처리
                for deposit in deposits {
                    if let Err(e) = process_deposit(
                        &repository_clone,
                        &chain_name,
                        deposit,
                        block_number,
                        required_confirmations,
                        sqs_clone.as_deref(),
                    ).await {
                        error!("[Analyzer] Failed to process deposit: {}", e);
                    }
                }

                // 마지막 처리 블록 업데이트
                if let Err(e) = repository_clone.update_last_processed_block(&chain_name, block_number).await {
                    error!(
                        "[Analyzer] Failed to update last processed block for {} block {}: {}",
                        chain_name, block_number, e
                    );
                }
            }
            Err(e) => {
                error!("[Analyzer] ❌ 블록 분석 실패: {}", e);
            }
        }
    }

    info!("[Analyzer] Loop finished because the channel was closed.");
}

/// Analyze a block and extract deposits
async fn analyze_block(
    block_data: BlockData,
    repository: &Arc<RepositoryWrapper>,
    kv_db: Option<&KeyValueDB>,
) -> Result<(String, u64, Vec<DepositInfo>), String> {
    // Create async closure for is_monitored_address
    // We need to clone Arc and convert strings to owned for the 'static lifetime requirement
    let is_monitored = |_repo: &Arc<RepositoryWrapper>, _db: Option<&KeyValueDB>, addr: &str, chain: &str| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>> {
        // Clone what we need to move into the async block
        let repo = repository.clone();
        let addr = addr.to_string();
        let chain = chain.to_string();

        // We can't capture kv_db by reference because of lifetime issues
        // Instead, we'll always use None and rely on the repository fallback
        // This is acceptable since RocksDB is checked inside is_monitored_address via the repository
        Box::pin(async move {
            is_monitored_address(&repo, None, &addr, &chain).await
        })
    };

    match block_data {
        BlockData::Ethereum(block) => {
            super::chains::analyze_ethereum_block(block, repository, kv_db, is_monitored).await
        }
        BlockData::Bitcoin(block) => {
            super::chains::analyze_bitcoin_block(block, repository, kv_db, is_monitored).await
        }
        BlockData::Tron(block) => {
            super::chains::analyze_tron_block(block, repository, kv_db, is_monitored).await
        }
        BlockData::Theta(block) => {
            super::chains::analyze_theta_block(block, repository, kv_db, is_monitored).await
        }
        BlockData::Icon(block) => {
            super::chains::analyze_icon_block(block, repository, kv_db, is_monitored).await
        }
        BlockData::Aion(block) => {
            super::chains::analyze_aion_block(block, repository, kv_db, is_monitored).await
        }
        BlockData::Algorand(block) => {
            super::chains::analyze_algorand_block(block, repository, kv_db, is_monitored).await
        }
        BlockData::Gxchain(block) => {
            super::chains::analyze_gxchain_block(block, repository, kv_db, is_monitored).await
        }
        BlockData::Quark(block) => {
            super::chains::analyze_quark_block(block, repository, kv_db, is_monitored).await
        }
        BlockData::Terra(block) => {
            super::chains::analyze_terra_block(block, repository, kv_db, is_monitored).await
        }
        BlockData::Tezos(block) => {
            super::chains::analyze_tezos_block(block, repository, kv_db, is_monitored).await
        }
        BlockData::Wayki(block) => {
            super::chains::analyze_wayki_block(block, repository, kv_db, is_monitored).await
        }
    }
}

/// Check if an address is monitored (RocksDB first, fallback to Repository)
async fn is_monitored_address(
    repository: &Arc<RepositoryWrapper>,
    kv_db: Option<&KeyValueDB>,
    address: &str,
    chain_name: &str,
) -> Result<bool, String> {
    // RocksDB 먼저 조회
    #[cfg(feature = "rocksdb-backend")]
    if let Some(kv_db) = kv_db {
        if let Ok(is_monitored) = is_monitored_address_in_rocksdb(kv_db, address, chain_name) {
            return Ok(is_monitored);
        }
    }

    #[cfg(not(feature = "rocksdb-backend"))]
    let _ = kv_db;

    // Repository fallback
    repository.is_monitored_address(address, chain_name).await
        .map_err(|e| format!("Failed to check if address is monitored: {}", e))
}

/// Process a deposit (save to DB and send SQS notification)
async fn process_deposit(
    repository: &Arc<RepositoryWrapper>,
    chain_name: &str,
    deposit: DepositInfo,
    current_block: u64,
    required_confirmations: u64,
    sqs_notifier: Option<&SqsNotifier>,
) -> Result<(), String> {
    let confirmations = current_block.saturating_sub(deposit.block_number) + 1;

    // Get wallet_id and account_id from repository (RocksDB cache)
    let (wallet_id, account_id) = repository
        .get_address_metadata(&deposit.address, chain_name)
        .await
        .map_err(|e| format!("Failed to get address metadata: {}", e))?
        .ok_or_else(|| format!("Address metadata not found for {}", deposit.address))?;

    info!(
        "[DEPOSIT] Received {} {} at address {} (wallet: {}, account: {:?}, tx: {}, block: {}, confirmations: {})",
        deposit.amount, chain_name, deposit.address, wallet_id, account_id, deposit.tx_hash, deposit.block_number, confirmations
    );

    // Check if deposit already exists in database
    let already_exists = repository
        .deposit_exists(&deposit.tx_hash, chain_name)
        .await
        .map_err(|e| format!("Failed to check deposit existence: {}", e))?;

    if already_exists {
        // Deposit already processed in Stage 1, only check Stage 2
        if confirmations >= required_confirmations {
            // Check if already confirmed to prevent duplicate notifications
            let is_confirmed = repository
                .is_deposit_confirmed(&deposit.tx_hash)
                .await
                .map_err(|e| format!("Failed to check confirmation status: {}", e))?;

            if !is_confirmed {
                info!("[DEPOSIT_CONFIRMED] {} confirmations reached (required: {}), sending confirmation", confirmations, required_confirmations);

                // Update deposit confirmed status
                repository.update_deposit_confirmed(&deposit.tx_hash)
                    .await
                    .map_err(|e| format!("Failed to update deposit confirmation: {}", e))?;

                // Send SQS notification
                if let Some(notifier) = sqs_notifier {
                    if let Err(e) = notifier.send_deposit_confirmed(
                        deposit.address.clone(),
                        wallet_id.clone(),
                        account_id.clone(),
                        chain_name.to_uppercase(),
                        deposit.tx_hash.clone(),
                        deposit.amount.clone(),
                        deposit.block_number,
                        confirmations,
                    ).await {
                        error!("[DEPOSIT_CONFIRMED] Failed to send SQS: {}", e);
                    } else {
                        info!("[DEPOSIT_CONFIRMED] ✅ SQS notification sent");
                    }
                }
            } else {
                // Already confirmed, skip
                return Ok(());
            }
        }
        return Ok(());
    }

    // New deposit - Stage 1: DEPOSIT_DETECTED (1 confirmation)
    if confirmations == 1 {
        info!("[DEPOSIT_DETECTED] {} confirmations reached for tx {}", confirmations, deposit.tx_hash);

        // Save to DB with status PENDING
        repository.save_deposit_event(
            &deposit.address,
            &wallet_id,
            account_id.as_deref(),
            chain_name,
            &deposit.tx_hash,
            deposit.block_number,
            &deposit.amount,
            deposit.amount_decimal,
        )
            .await
            .map_err(|e| format!("Failed to save deposit event: {}", e))?;

        // Send SQS notification
        if let Some(notifier) = sqs_notifier {
            if let Err(e) = notifier.send_deposit_detected(
                deposit.address.clone(),
                wallet_id.clone(),
                account_id.clone(),
                chain_name.to_uppercase(),
                deposit.tx_hash.clone(),
                deposit.amount.clone(),
                deposit.block_number,
            ).await {
                error!("[DEPOSIT_DETECTED] Failed to send SQS: {}", e);
            } else {
                info!("[DEPOSIT_DETECTED] ✅ SQS notification sent");
            }
        }
    }

    Ok(())
}
