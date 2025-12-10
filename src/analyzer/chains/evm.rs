use super::types::DepositInfo;
use super::utils::parse_wei_to_decimal;
use crate::respository::RepositoryWrapper;
use std::sync::Arc;
use log::info;

#[cfg(feature = "rocksdb-backend")]
use rocksdb::DB as RocksDB;
#[cfg(feature = "rocksdb-backend")]
pub type KeyValueDB = RocksDB;

#[cfg(not(feature = "rocksdb-backend"))]
pub type KeyValueDB = ();

/// EVM 호환 블록체인 분석 (Ethereum, AION, QUARK, THETA)
pub async fn analyze_evm_block<F>(
    chain_name: &str,
    block_number: u64,
    transactions: Vec<EVMTransaction>,
    repository: &Arc<RepositoryWrapper>,
    kv_db: Option<&KeyValueDB>,
    is_monitored: F,
) -> Result<(String, u64, Vec<DepositInfo>), String>
where
    F: Fn(&Arc<RepositoryWrapper>, Option<&KeyValueDB>, &str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>,
{
    info!("[Analyzer] {} Block #{} received (transactions: {})", chain_name, block_number, transactions.len());

    let mut deposits = Vec::new();
    let mut checked_count = 0;

    for (i, tx) in transactions.iter().enumerate() {
        if let Some(to_address) = &tx.to {
            checked_count += 1;

            // Check RocksDB directly
            #[cfg(feature = "rocksdb-backend")]
            let is_addr_monitored = if let Some(db) = kv_db {
                use crate::respository::is_monitored_address_in_rocksdb;
                is_monitored_address_in_rocksdb(db, to_address, chain_name).unwrap_or(false)
            } else {
                false
            };

            #[cfg(not(feature = "rocksdb-backend"))]
            let is_addr_monitored = is_monitored(repository, kv_db, to_address, chain_name).await?;

            if is_addr_monitored {
                let amount_hex = tx.value.as_deref().unwrap_or("0x0");
                let amount_decimal = parse_wei_to_decimal(amount_hex).ok();

                info!("[Analyzer] ✅ {} 입금 감지! 블록: {} | 주소: {} | 금액: {:?}",
                    chain_name, block_number, to_address, amount_decimal);

                deposits.push(DepositInfo::new(
                    to_address.clone(),
                    tx.hash.as_deref().unwrap_or("").to_string(),
                    block_number,
                    amount_hex.to_string(),
                    amount_decimal,
                ));
            }
        }
    }

    if checked_count > 0 && deposits.is_empty() {
        info!("[Analyzer] {} 블록 #{}: {}개 트랜잭션 확인, 관리 주소 없음", chain_name, block_number, checked_count);
    }

    Ok((chain_name.to_string(), block_number, deposits))
}

/// EVM 트랜잭션 구조체
pub struct EVMTransaction {
    pub to: Option<String>,
    pub value: Option<String>,
    pub hash: Option<String>,
}

/// Ethereum 블록 분석
pub async fn analyze_ethereum_block<F>(
    block: crate::coin::ethereum::model::EthereumBlock,
    repository: &Arc<RepositoryWrapper>,
    kv_db: Option<&KeyValueDB>,
    is_monitored: F,
) -> Result<(String, u64, Vec<DepositInfo>), String>
where
    F: Fn(&Arc<RepositoryWrapper>, Option<&KeyValueDB>, &str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>,
{
    let chain_name = "SEPOLIA";
    let result = block.result.ok_or("Missing 'result' in EthereumBlock")?;

    let number_hex = result.number.trim();
    let block_number = if number_hex.starts_with("0x") {
        u64::from_str_radix(&number_hex[2..], 16)
    } else {
        number_hex.parse::<u64>()
    }.map_err(|e| format!("Failed to parse ETH block number: {}", e))?;

    let transactions: Vec<EVMTransaction> = result.transactions.iter().map(|tx| EVMTransaction {
        to: tx.to.clone(),
        value: tx.value.clone(),
        hash: tx.hash.clone(),
    }).collect();

    analyze_evm_block(chain_name, block_number, transactions, repository, kv_db, is_monitored).await
}

/// AION 블록 분석
pub async fn analyze_aion_block<F>(
    block: crate::coin::aion::model::AionBlock,
    repository: &Arc<RepositoryWrapper>,
    kv_db: Option<&KeyValueDB>,
    is_monitored: F,
) -> Result<(String, u64, Vec<DepositInfo>), String>
where
    F: Fn(&Arc<RepositoryWrapper>, Option<&KeyValueDB>, &str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>,
{
    let chain_name = "AION";
    let result = block.result.ok_or("Missing 'result' in AionBlock")?;

    let number_hex = result.number.trim();
    let block_number = if number_hex.starts_with("0x") {
        u64::from_str_radix(&number_hex[2..], 16)
    } else {
        number_hex.parse::<u64>()
    }.map_err(|e| format!("Failed to parse AION block number: {}", e))?;

    let transactions: Vec<EVMTransaction> = result.transactions.iter().map(|tx| EVMTransaction {
        to: tx.to.clone(),
        value: tx.value.clone(),
        hash: tx.hash.clone(),
    }).collect();

    analyze_evm_block(chain_name, block_number, transactions, repository, kv_db, is_monitored).await
}

/// QUARK 블록 분석
pub async fn analyze_quark_block<F>(
    block: crate::coin::quark::model::QuarkBlock,
    repository: &Arc<RepositoryWrapper>,
    kv_db: Option<&KeyValueDB>,
    is_monitored: F,
) -> Result<(String, u64, Vec<DepositInfo>), String>
where
    F: Fn(&Arc<RepositoryWrapper>, Option<&KeyValueDB>, &str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>,
{
    let chain_name = "QUARK";
    let result = block.result.ok_or("Missing 'result' in QuarkBlock")?;

    let number_hex = result.number.trim();
    let block_number = if number_hex.starts_with("0x") {
        u64::from_str_radix(&number_hex[2..], 16)
    } else {
        number_hex.parse::<u64>()
    }.map_err(|e| format!("Failed to parse QUARK block number: {}", e))?;

    let transactions: Vec<EVMTransaction> = result.transactions.iter().map(|tx| EVMTransaction {
        to: tx.to.clone(),
        value: tx.value.clone(),
        hash: tx.hash.clone(),
    }).collect();

    analyze_evm_block(chain_name, block_number, transactions, repository, kv_db, is_monitored).await
}

/// THETA 블록 분석
pub async fn analyze_theta_block<F>(
    block: crate::coin::theta::model::ThetaBlock,
    repository: &Arc<RepositoryWrapper>,
    kv_db: Option<&KeyValueDB>,
    is_monitored: F,
) -> Result<(String, u64, Vec<DepositInfo>), String>
where
    F: Fn(&Arc<RepositoryWrapper>, Option<&KeyValueDB>, &str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>,
{
    let chain_name = "THETA";
    let block_number = block.height.parse::<u64>()
        .map_err(|e| format!("Failed to parse THETA block number: {}", e))?;

    let transactions: Vec<EVMTransaction> = block.transactions.iter().map(|tx| {
        let value = if tx.value.starts_with("0x") {
            tx.value.clone()
        } else {
            format!("0x{}", tx.value)
        };

        EVMTransaction {
            to: tx.to.clone(),
            value: Some(value),
            hash: Some(tx.hash.clone()),
        }
    }).collect();

    analyze_evm_block(chain_name, block_number, transactions, repository, kv_db, is_monitored).await
}
