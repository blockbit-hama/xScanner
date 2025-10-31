use crate::types::AppError;
use leveldb::database::Database;
use leveldb::kv::KV;
use leveldb::options::{Options, ReadOptions, WriteOptions};
use std::path::Path;

/// 주소를 키로 사용 (주소 문자열을 직접 키로 사용)
pub fn open_leveldb(path_str: &str) -> Result<Database<String>, AppError> {
  let path = Path::new(path_str);
  let mut options = Options::new();
  options.create_if_missing = true;
  
  Database::open(path, options).map_err(|e| {
    AppError::Initialization(format!("Failed to open LevelDB at '{}': {}", path_str, e))
  })
}

/// 고객 주소와 고객 ID를 LevelDB에 저장
/// Key: chain_name:address (소문자로 정규화)
/// Value: customer_id
pub fn add_customer_address_to_leveldb(
  db: &Database<String>,
  address: &str,
  customer_id: &str,
  chain_name: &str,
) -> Result<(), AppError> {
  let write_options = WriteOptions::new();
  let normalized_address = address.to_lowercase();
  let key = format!("{}:{}", chain_name.to_lowercase(), normalized_address);
  db.put(write_options, key, customer_id)
    .map_err(|e| AppError::Database(format!("LevelDB put failed: {}", e)))
}

/// 주소로 고객 ID 조회 (LevelDB에서 빠르게 조회)
/// chain_name을 포함하여 정확한 체인의 주소를 조회
pub fn get_customer_id_from_leveldb(
  db: &Database<String>,
  address: &str,
  chain_name: &str,
) -> Result<Option<String>, AppError> {
  let read_options = ReadOptions::new();
  let normalized_address = address.to_lowercase();
  let key = format!("{}:{}", chain_name.to_lowercase(), normalized_address);
  match db.get(read_options, key) {
    Ok(Some(customer_id)) => Ok(Some(customer_id)),
    Ok(None) => Ok(None),
    Err(e) => Err(AppError::Database(format!("LevelDB get failed: {}", e))),
  }
}

/// 일괄 주소 추가 (배치 처리로 성능 향상)
pub fn batch_add_customer_addresses(
  db: &Database<String>,
  addresses: Vec<(String, String, String)>, // (address, customer_id, chain_name)
) -> Result<usize, AppError> {
  use leveldb::batch::WriteBatch;
  
  let mut batch = WriteBatch::new();
  let mut count = 0;
  
  for (address, customer_id, chain_name) in addresses {
    let normalized_address = address.to_lowercase();
    let key = format!("{}:{}", chain_name.to_lowercase(), normalized_address);
    batch.put(key, customer_id);
    count += 1;
  }
  
  let write_options = WriteOptions::new();
  db.write(write_options, batch)
    .map_err(|e| AppError::Database(format!("LevelDB batch write failed: {}", e)))?;
  
  Ok(count)
}
