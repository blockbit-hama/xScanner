use super::types::DepositInfo;
use super::utils::loop_to_icx;
use crate::respository::RepositoryWrapper;
use std::sync::Arc;
use log::info;

#[cfg(feature = "rocksdb-backend")]
use rocksdb::DB as RocksDB;
#[cfg(feature = "rocksdb-backend")]
pub type KeyValueDB = RocksDB;

#[cfg(not(feature = "rocksdb-backend"))]
pub type KeyValueDB = ();

/// ICON 블록 분석
pub async fn analyze_icon_block<F>(
    block: crate::coin::icon::model::IconBlock,
    repository: &Arc<RepositoryWrapper>,
    kv_db: Option<&KeyValueDB>,
    is_monitored: F,
) -> Result<(String, u64, Vec<DepositInfo>), String>
where
    F: Fn(&Arc<RepositoryWrapper>, Option<&KeyValueDB>, &str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>,
{
    let chain_name = "ICON";
    let block_number = block.height;

    info!("[Analyzer] ICON Block #{} received", block_number);

    let mut deposits = Vec::new();

    for tx in &block.confirmed_transaction_list {
        let to_address = &tx.to;

        if is_monitored(repository, kv_db, to_address, chain_name).await? {
            let amount_decimal = tx.value.as_ref()
                .and_then(|v| v.parse::<u64>().ok())
                .map(loop_to_icx);

            info!("[Analyzer] ✅ ICON 입금 감지! 블록: {} | 주소: {} | 금액: {:?} ICX",
                block_number, to_address, amount_decimal);

            deposits.push(DepositInfo::new(
                to_address.clone(),
                tx.tx_hash.clone(),
                block_number,
                tx.value.clone().unwrap_or_default(),
                amount_decimal,
            ));
        }
    }

    Ok((chain_name.to_string(), block_number, deposits))
}
