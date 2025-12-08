use super::types::DepositInfo;
use super::utils::microalgo_to_algo;
use crate::respository::RepositoryWrapper;
use std::sync::Arc;
use log::info;

#[cfg(feature = "rocksdb-backend")]
use rocksdb::DB as RocksDB;
#[cfg(feature = "rocksdb-backend")]
pub type KeyValueDB = RocksDB;

#[cfg(not(feature = "rocksdb-backend"))]
pub type KeyValueDB = ();

/// Algorand 블록 분석
pub async fn analyze_algorand_block<F>(
    block: crate::coin::algorand::model::AlgorandBlock,
    repository: &Arc<RepositoryWrapper>,
    kv_db: Option<&KeyValueDB>,
    is_monitored: F,
) -> Result<(String, u64, Vec<DepositInfo>), String>
where
    F: Fn(&Arc<RepositoryWrapper>, Option<&KeyValueDB>, &str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>,
{
    let chain_name = "ALGORAND";
    let block_number = block.round;

    info!("[Analyzer] ALGORAND Block #{} received", block_number);

    let mut deposits = Vec::new();

    for tx in &block.txns.transactions {
        if let Some(payment) = &tx.payment {
            let to_address = &payment.to;

            if is_monitored(repository, kv_db, to_address, chain_name).await? {
                let amount_decimal = microalgo_to_algo(payment.amount);

                info!("[Analyzer] ✅ ALGORAND 입금 감지! 블록: {} | 주소: {} | 금액: {} ALGO",
                    block_number, to_address, amount_decimal);

                deposits.push(DepositInfo::new(
                    to_address.clone(),
                    tx.tx.clone(),
                    block_number,
                    payment.amount.to_string(),
                    Some(amount_decimal),
                ));
            }
        }
    }

    if !deposits.is_empty() {
        info!("[Analyzer] Found {} ALGORAND deposits in block {}", deposits.len(), block_number);
    }

    Ok((chain_name.to_string(), block_number, deposits))
}
