use super::types::DepositInfo;
use super::utils::satoshi_to_btc;
use crate::respository::RepositoryWrapper;
use std::sync::Arc;
use log::info;

#[cfg(feature = "rocksdb-backend")]
use rocksdb::DB as RocksDB;
#[cfg(feature = "rocksdb-backend")]
pub type KeyValueDB = RocksDB;

#[cfg(not(feature = "rocksdb-backend"))]
pub type KeyValueDB = ();

/// Bitcoin 블록 분석
pub async fn analyze_bitcoin_block<F>(
    block: crate::coin::bitcoin::model::BitcoinBlock,
    repository: &Arc<RepositoryWrapper>,
    kv_db: Option<&KeyValueDB>,
    is_monitored: F,
) -> Result<(String, u64, Vec<DepositInfo>), String>
where
    F: Fn(&Arc<RepositoryWrapper>, Option<&KeyValueDB>, &str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>,
{
    let chain_name = "BTC";
    let block_number = block.height;

    info!("[Analyzer] Bitcoin Block #{} received", block_number);

    let mut deposits = Vec::new();

    for tx in &block.tx {
        for output in &tx.out {
            if let Some(address) = &output.addr {
                if is_monitored(repository, kv_db, address, chain_name).await? {
                    let amount_decimal = satoshi_to_btc(output.value);

                    info!("[Analyzer] ✅ BTC 입금 감지! 블록: {} | 주소: {} | 금액: {} BTC",
                        block_number, address, amount_decimal);

                    deposits.push(DepositInfo::new(
                        address.clone(),
                        tx.hash.clone(),
                        block_number,
                        output.value.to_string(),
                        Some(amount_decimal),
                    ));
                }
            }
        }
    }

    Ok((chain_name.to_string(), block_number, deposits))
}
