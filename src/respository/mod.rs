mod postgresql;
#[cfg(feature = "leveldb-backend")]
mod leveldb;
#[cfg(feature = "rocksdb-backend")]
mod rocksdb;
mod r#trait;
mod memory;
mod postgresql_repo;
#[cfg(feature = "leveldb-backend")]
mod leveldb_repo;
#[cfg(feature = "rocksdb-backend")]
mod rocksdb_repo;
mod wrapper;

// Repository trait
pub use r#trait::Repository;

// Repository implementations
pub use memory::MemoryRepository;
pub use postgresql_repo::PostgreSQLRepository;
#[cfg(feature = "leveldb-backend")]
pub use leveldb_repo::LevelDBRepository;
#[cfg(feature = "rocksdb-backend")]
pub use rocksdb_repo::RocksDBRepository;
pub use wrapper::RepositoryWrapper;

// Legacy exports (for backward compatibility)
pub use postgresql::connect_db;
pub use postgresql::get_last_processed_block;
pub use postgresql::setup_db_schema;
pub use postgresql::update_last_processed_block;
pub use postgresql::init_last_processed_block;
pub use postgresql::save_deposit_event;
// Note: increment_customer_balance removed - balance management handled by backend
// Note: get_customer_id_by_address removed - uses RocksDB cache only
// Note: load_customer_addresses_to_rocksdb removed - uses SQS + file cache instead

#[cfg(feature = "leveldb-backend")]
pub use leveldb::open_leveldb;
#[cfg(feature = "leveldb-backend")]
pub use leveldb::add_customer_address_to_leveldb;
#[cfg(feature = "leveldb-backend")]
pub use leveldb::get_customer_id_from_leveldb;
#[cfg(feature = "leveldb-backend")]
pub use leveldb::batch_add_customer_addresses;

#[cfg(feature = "rocksdb-backend")]
pub use rocksdb::open_rocksdb;
#[cfg(feature = "rocksdb-backend")]
pub use rocksdb::add_monitored_address_to_rocksdb;
#[cfg(feature = "rocksdb-backend")]
pub use rocksdb::is_monitored_address_in_rocksdb;
#[cfg(feature = "rocksdb-backend")]
pub use rocksdb::get_address_metadata_from_rocksdb;
#[cfg(feature = "rocksdb-backend")]
pub use rocksdb::batch_add_monitored_addresses;
#[cfg(feature = "rocksdb-backend")]
pub use rocksdb::AddressMetadata;
// Deprecated exports for backward compatibility
#[cfg(feature = "rocksdb-backend")]
#[allow(deprecated)]
pub use rocksdb::add_customer_address_to_rocksdb;
#[cfg(feature = "rocksdb-backend")]
#[allow(deprecated)]
pub use rocksdb::get_customer_id_from_rocksdb;
#[cfg(feature = "rocksdb-backend")]
#[allow(deprecated)]
pub use rocksdb::batch_add_customer_addresses;
