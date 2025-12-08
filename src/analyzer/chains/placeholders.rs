use super::types::DepositInfo;
use crate::respository::RepositoryWrapper;
use std::sync::Arc;
use log::info;

#[cfg(feature = "rocksdb-backend")]
use rocksdb::DB as RocksDB;
#[cfg(feature = "rocksdb-backend")]
pub type KeyValueDB = RocksDB;

#[cfg(not(feature = "rocksdb-backend"))]
pub type KeyValueDB = ();

/// GXCHAIN 블록 분석 (Placeholder)
pub async fn analyze_gxchain_block<F>(
    _block: crate::coin::gxchain::model::GxchainBlock,
    _repository: &Arc<RepositoryWrapper>,
    _kv_db: Option<&KeyValueDB>,
    _is_monitored: F,
) -> Result<(String, u64, Vec<DepositInfo>), String>
where
    F: Fn(&Arc<RepositoryWrapper>, Option<&KeyValueDB>, &str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>,
{
    info!("[Analyzer] GXCHAIN block received (placeholder - not yet implemented)");
    Ok(("GXCHAIN".to_string(), 0, Vec::new()))
}

/// TERRA 블록 분석 (Placeholder)
pub async fn analyze_terra_block<F>(
    block: crate::coin::terra::model::TerraBlock,
    _repository: &Arc<RepositoryWrapper>,
    _kv_db: Option<&KeyValueDB>,
    _is_monitored: F,
) -> Result<(String, u64, Vec<DepositInfo>), String>
where
    F: Fn(&Arc<RepositoryWrapper>, Option<&KeyValueDB>, &str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>,
{
    let chain_name = "TERRA";
    let block_number = block.block.header.height.parse::<u64>()
        .map_err(|e| format!("Failed to parse TERRA block number: {}", e))?;

    info!("[Analyzer] TERRA Block #{} received (placeholder)", block_number);
    Ok((chain_name.to_string(), block_number, Vec::new()))
}

/// TEZOS 블록 분석 (Placeholder)
pub async fn analyze_tezos_block<F>(
    _block: crate::coin::tezos::model::TezosBlock,
    _repository: &Arc<RepositoryWrapper>,
    _kv_db: Option<&KeyValueDB>,
    _is_monitored: F,
) -> Result<(String, u64, Vec<DepositInfo>), String>
where
    F: Fn(&Arc<RepositoryWrapper>, Option<&KeyValueDB>, &str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>,
{
    info!("[Analyzer] TEZOS block received (placeholder - not yet implemented)");
    Ok(("TEZOS".to_string(), 0, Vec::new()))
}

/// WAYKI 블록 분석 (Placeholder)
pub async fn analyze_wayki_block<F>(
    block: crate::coin::wayki::model::WaykiBlock,
    _repository: &Arc<RepositoryWrapper>,
    _kv_db: Option<&KeyValueDB>,
    _is_monitored: F,
) -> Result<(String, u64, Vec<DepositInfo>), String>
where
    F: Fn(&Arc<RepositoryWrapper>, Option<&KeyValueDB>, &str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>,
{
    let chain_name = "WAYKI";
    let block_number = block.height;

    info!("[Analyzer] WAYKI Block #{} received (placeholder)", block_number);
    Ok((chain_name.to_string(), block_number, Vec::new()))
}
