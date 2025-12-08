use crate::types::AppError;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[cfg(feature = "rocksdb-backend")]
use rocksdb::{DB, Options, WriteBatch};

/// Address metadata stored in RocksDB
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddressMetadata {
    pub wallet_id: String,
    pub account_id: Option<String>, // None for Omnibus (Master) Address
}

/// Open RocksDB database
#[cfg(feature = "rocksdb-backend")]
pub fn open_rocksdb(path_str: &str) -> Result<DB, AppError> {
    let path = Path::new(path_str);
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);

    DB::open(&opts, path).map_err(|e| {
        AppError::Initialization(format!("Failed to open RocksDB at '{}': {}", path_str, e))
    })
}

/// 관리 대상 주소를 RocksDB에 추가 (wallet_id, account_id 포함)
/// Key: chain_name:address (소문자 정규화)
/// Value: JSON {"wallet_id": "...", "account_id": "..." or null}
#[cfg(feature = "rocksdb-backend")]
pub fn add_monitored_address_to_rocksdb(
    db: &DB,
    address: &str,
    chain_name: &str,
    wallet_id: &str,
    account_id: Option<&str>,
) -> Result<(), AppError> {
    let normalized_address = address.to_lowercase();
    let key = format!("{}:{}", chain_name.to_lowercase(), normalized_address);

    let metadata = AddressMetadata {
        wallet_id: wallet_id.to_string(),
        account_id: account_id.map(|s| s.to_string()),
    };

    let value = serde_json::to_string(&metadata)
        .map_err(|e| AppError::Database(format!("Failed to serialize metadata: {}", e)))?;

    db.put(key.as_bytes(), value.as_bytes())
        .map_err(|e| AppError::Database(format!("RocksDB put failed: {}", e)))
}

/// 주소가 관리 대상인지 확인 (RocksDB에서 빠른 조회)
#[cfg(feature = "rocksdb-backend")]
pub fn is_monitored_address_in_rocksdb(
    db: &DB,
    address: &str,
    chain_name: &str,
) -> Result<bool, AppError> {
    let normalized_address = address.to_lowercase();
    let key = format!("{}:{}", chain_name.to_lowercase(), normalized_address);
    match db.get(key.as_bytes()) {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => Err(AppError::Database(format!("RocksDB get failed: {}", e))),
    }
}

/// 주소의 메타데이터 조회 (wallet_id, account_id)
#[cfg(feature = "rocksdb-backend")]
pub fn get_address_metadata_from_rocksdb(
    db: &DB,
    address: &str,
    chain_name: &str,
) -> Result<Option<AddressMetadata>, AppError> {
    let normalized_address = address.to_lowercase();
    let key = format!("{}:{}", chain_name.to_lowercase(), normalized_address);
    match db.get(key.as_bytes()) {
        Ok(Some(value)) => {
            let metadata: AddressMetadata = serde_json::from_slice(&value)
                .map_err(|e| AppError::Database(format!("Failed to deserialize metadata: {}", e)))?;
            Ok(Some(metadata))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(AppError::Database(format!("RocksDB get failed: {}", e))),
    }
}

/// 배치로 주소 추가 (SQS 메시지 처리용)
#[cfg(feature = "rocksdb-backend")]
pub fn batch_add_monitored_addresses(
    db: &DB,
    addresses: Vec<(String, String, String, Option<String>)>, // (address, chain_name, wallet_id, account_id)
) -> Result<usize, AppError> {
    let mut batch = WriteBatch::default();
    let mut count = 0;

    for (address, chain_name, wallet_id, account_id) in addresses {
        let normalized_address = address.to_lowercase();
        let key = format!("{}:{}", chain_name.to_lowercase(), normalized_address);

        let metadata = AddressMetadata {
            wallet_id,
            account_id,
        };

        let value = serde_json::to_string(&metadata)
            .map_err(|e| AppError::Database(format!("Failed to serialize metadata: {}", e)))?;

        batch.put(key.as_bytes(), value.as_bytes());
        count += 1;
    }

    db.write(batch)
        .map_err(|e| AppError::Database(format!("RocksDB batch write failed: {}", e)))?;

    Ok(count)
}

// Backward compatibility (deprecated)
#[cfg(feature = "rocksdb-backend")]
#[deprecated(note = "Use add_monitored_address_to_rocksdb instead")]
pub fn add_customer_address_to_rocksdb(
    db: &DB,
    address: &str,
    _customer_id: &str,
    chain_name: &str,
) -> Result<(), AppError> {
    // Use placeholder values for wallet_id and account_id for backward compatibility
    add_monitored_address_to_rocksdb(db, address, chain_name, "unknown", None)
}

#[cfg(feature = "rocksdb-backend")]
#[deprecated(note = "Use is_monitored_address_in_rocksdb instead")]
pub fn get_customer_id_from_rocksdb(
    db: &DB,
    address: &str,
    chain_name: &str,
) -> Result<Option<String>, AppError> {
    match is_monitored_address_in_rocksdb(db, address, chain_name)? {
        true => Ok(Some("exists".to_string())), // Dummy value for compatibility
        false => Ok(None),
    }
}

#[cfg(feature = "rocksdb-backend")]
#[deprecated(note = "Use batch_add_monitored_addresses instead")]
pub fn batch_add_customer_addresses(
    db: &DB,
    addresses: Vec<(String, String, String)>, // (address, customer_id, chain_name)
) -> Result<usize, AppError> {
    let simplified: Vec<(String, String, String, Option<String>)> = addresses
        .into_iter()
        .map(|(addr, _cust_id, chain)| (addr, chain, "unknown".to_string(), None))
        .collect();
    batch_add_monitored_addresses(db, simplified)
}

// No-op implementations when rocksdb feature is not enabled
#[cfg(not(feature = "rocksdb-backend"))]
pub fn open_rocksdb(_path_str: &str) -> Result<(), AppError> {
    Err(AppError::Initialization("RocksDB feature not enabled".to_string()))
}

#[cfg(not(feature = "rocksdb-backend"))]
pub fn add_monitored_address_to_rocksdb(
    _db: &(),
    _address: &str,
    _chain_name: &str,
) -> Result<(), AppError> {
    Err(AppError::Database("RocksDB feature not enabled".to_string()))
}

#[cfg(not(feature = "rocksdb-backend"))]
pub fn is_monitored_address_in_rocksdb(
    _db: &(),
    _address: &str,
    _chain_name: &str,
) -> Result<bool, AppError> {
    Err(AppError::Database("RocksDB feature not enabled".to_string()))
}

#[cfg(not(feature = "rocksdb-backend"))]
pub fn batch_add_monitored_addresses(
    _db: &(),
    _addresses: Vec<(String, String)>,
) -> Result<usize, AppError> {
    Err(AppError::Database("RocksDB feature not enabled".to_string()))
}

// Backward compatibility (deprecated) - no-op versions
#[cfg(not(feature = "rocksdb-backend"))]
pub fn add_customer_address_to_rocksdb(
    _db: &(),
    _address: &str,
    _customer_id: &str,
    _chain_name: &str,
) -> Result<(), AppError> {
    Err(AppError::Database("RocksDB feature not enabled".to_string()))
}

#[cfg(not(feature = "rocksdb-backend"))]
pub fn get_customer_id_from_rocksdb(
    _db: &(),
    _address: &str,
    _chain_name: &str,
) -> Result<Option<String>, AppError> {
    Err(AppError::Database("RocksDB feature not enabled".to_string()))
}

#[cfg(not(feature = "rocksdb-backend"))]
pub fn batch_add_customer_addresses(
    _db: &(),
    _addresses: Vec<(String, String, String)>,
) -> Result<usize, AppError> {
    Err(AppError::Database("RocksDB feature not enabled".to_string()))
}
