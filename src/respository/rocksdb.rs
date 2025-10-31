use crate::types::AppError;
use std::path::Path;

#[cfg(feature = "rocksdb-backend")]
use rocksdb::{DB, Options, WriteBatch};

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

/// ?? ??? ?? ID? RocksDB? ??
/// Key: chain_name:address (???? ???)
/// Value: customer_id
#[cfg(feature = "rocksdb-backend")]
pub fn add_customer_address_to_rocksdb(
    db: &DB,
    address: &str,
    customer_id: &str,
    chain_name: &str,
) -> Result<(), AppError> {
    let normalized_address = address.to_lowercase();
    let key = format!("{}:{}", chain_name.to_lowercase(), normalized_address);
    db.put(key.as_bytes(), customer_id.as_bytes())
        .map_err(|e| AppError::Database(format!("RocksDB put failed: {}", e)))
}

/// ??? ?? ID ?? (RocksDB?? ??? ??)
/// chain_name? ???? ??? ??? ??? ??
#[cfg(feature = "rocksdb-backend")]
pub fn get_customer_id_from_rocksdb(
    db: &DB,
    address: &str,
    chain_name: &str,
) -> Result<Option<String>, AppError> {
    let normalized_address = address.to_lowercase();
    let key = format!("{}:{}", chain_name.to_lowercase(), normalized_address);
    match db.get(key.as_bytes()) {
        Ok(Some(value)) => {
            let customer_id = String::from_utf8(value.to_vec())
                .map_err(|e| AppError::Database(format!("Invalid UTF-8 in customer_id: {}", e)))?;
            Ok(Some(customer_id))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(AppError::Database(format!("RocksDB get failed: {}", e))),
    }
}

/// ?? ?? ?? (?? ??? ?? ??)
#[cfg(feature = "rocksdb-backend")]
pub fn batch_add_customer_addresses(
    db: &DB,
    addresses: Vec<(String, String, String)>, // (address, customer_id, chain_name)
) -> Result<usize, AppError> {
    let mut batch = WriteBatch::default();
    let mut count = 0;

    for (address, customer_id, chain_name) in addresses {
        let normalized_address = address.to_lowercase();
        let key = format!("{}:{}", chain_name.to_lowercase(), normalized_address);
        batch.put(key.as_bytes(), customer_id.as_bytes());
        count += 1;
    }

    db.write(batch)
        .map_err(|e| AppError::Database(format!("RocksDB batch write failed: {}", e)))?;

    Ok(count)
}

// No-op implementations when rocksdb feature is not enabled
#[cfg(not(feature = "rocksdb-backend"))]
pub fn open_rocksdb(_path_str: &str) -> Result<(), AppError> {
    Err(AppError::Initialization("RocksDB feature not enabled".to_string()))
}

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
