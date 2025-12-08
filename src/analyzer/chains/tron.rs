use super::types::DepositInfo;
use super::utils::sun_to_trx;
use crate::respository::RepositoryWrapper;
use std::sync::Arc;
use log::info;

#[cfg(feature = "rocksdb-backend")]
use rocksdb::DB as RocksDB;
#[cfg(feature = "rocksdb-backend")]
pub type KeyValueDB = RocksDB;

#[cfg(not(feature = "rocksdb-backend"))]
pub type KeyValueDB = ();

/// TRON 블록 분석
pub async fn analyze_tron_block<F>(
    block: crate::coin::tron::model::TronBlock,
    repository: &Arc<RepositoryWrapper>,
    kv_db: Option<&KeyValueDB>,
    is_monitored: F,
) -> Result<(String, u64, Vec<DepositInfo>), String>
where
    F: Fn(&Arc<RepositoryWrapper>, Option<&KeyValueDB>, &str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>,
{
    let chain_name = "TRON";
    let block_number = block.block_header.raw_data.number;

    info!("[Analyzer] TRON Block #{} received", block_number);

    let mut deposits = Vec::new();

    for tx in &block.transactions {
        // TRX 전송 트랜잭션 처리
        if let Some(contract) = tx.raw_data.contract.first() {
            if contract.contract_type == "TransferContract" {
                if let Some(to_address) = &contract.parameter.value.to_address {
                    if is_monitored(repository, kv_db, to_address, chain_name).await? {
                        let amount = contract.parameter.value.amount.unwrap_or(0);
                        let amount_decimal = sun_to_trx(amount as u64);

                        info!("[Analyzer] ✅ TRON 입금 감지! 블록: {} | 주소: {} | 금액: {} TRX",
                            block_number, to_address, amount_decimal);

                        deposits.push(DepositInfo::new(
                            to_address.clone(),
                            tx.tx_id.clone(),
                            block_number,
                            amount.to_string(),
                            Some(amount_decimal),
                        ));
                    }
                }
            }
        }
    }

    Ok((chain_name.to_string(), block_number, deposits))
}
