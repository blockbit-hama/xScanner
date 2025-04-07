use crate::types::AppError;
use leveldb::database::Database;
use leveldb::kv::KV;
use leveldb::options::{Options, ReadOptions, WriteOptions};
use std::path::Path;

/// 단순한 해시로 문자열을 i32로 매핑
fn address_to_key(address: &str) -> i32 {
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};
  let mut hasher = DefaultHasher::new();
  address.hash(&mut hasher);
  (hasher.finish() % i32::MAX as u64) as i32
}

/// LevelDB 열기
pub fn open_leveldb(path_str: &str) -> Result<Database<i32>, AppError> {
  let path = Path::new(path_str);
  let mut options = Options::new();
  options.create_if_missing = true;
  
  Database::open(path, options).map_err(|e| {
    AppError::Initialization(format!("Failed to open LevelDB at '{}': {}", path_str, e))
  })
}

/// 주소 추가 (key는 해시값, value는 항상 1)
pub fn add_address_to_leveldb(db: &Database<i32>, address: &str) -> Result<(), AppError> {
  let write_options = WriteOptions::new();
  let key = address_to_key(address);
  db.put(write_options, key, &[1])
    .map_err(|e| AppError::Database(format!("LevelDB put failed: {}", e)))
}

/// 주소 존재 여부 확인
pub fn check_address_exists_in_leveldb(db: &Database<i32>, address: &str) -> Result<bool, AppError> {
  let read_options = ReadOptions::new();
  let key = address_to_key(address);
  match db.get(read_options, key) {
    Ok(Some(_)) => Ok(true),
    Ok(None) => Ok(false),
    Err(e) => Err(AppError::Database(format!("LevelDB get failed: {}", e))),
  }
}
