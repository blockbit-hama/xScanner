use crate::types::AppError;
use crate::respository::r#trait::Repository;
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;
use log::info;

#[cfg(feature = "rocksdb-backend")]
use rocksdb::DB;

/// RocksDB 기반 Repository 구현
#[derive(Clone)]
pub struct RocksDBRepository {
    #[cfg(feature = "rocksdb-backend")]
    db: Arc<DB>,
    #[cfg(not(feature = "rocksdb-backend"))]
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(feature = "rocksdb-backend")]
impl RocksDBRepository {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DB {
        &self.db
    }

    /// 고객 주소로 고객 ID 조회 (RocksDB에서 빠른 조회)
    pub fn get_customer_id_from_rocksdb(
        &self,
        address: &str,
        chain_name: &str,
    ) -> Result<Option<String>, AppError> {
        crate::respository::rocksdb::get_customer_id_from_rocksdb(&self.db, address, chain_name)
    }
}

#[cfg(not(feature = "rocksdb-backend"))]
impl RocksDBRepository {
    pub fn new(_db: Arc<()>) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl Repository for RocksDBRepository {
    #[cfg(feature = "rocksdb-backend")]
    async fn get_last_processed_block(&self, chain: &str) -> Result<u64, AppError> {
        let key = format!("last_block:{}", chain);
        match self.db.get(key.as_bytes()) {
            Ok(Some(value)) => {
                let block_str = String::from_utf8(value.to_vec())
                    .map_err(|e| AppError::Database(format!("Invalid UTF-8: {}", e)))?;
                block_str.parse::<u64>()
                    .map_err(|e| AppError::Database(format!("Failed to parse block number: {}", e)))
            }
            Ok(None) => Ok(0),
            Err(e) => Err(AppError::Database(format!("RocksDB get failed: {}", e))),
        }
    }

    #[cfg(not(feature = "rocksdb-backend"))]
    async fn get_last_processed_block(&self, _chain: &str) -> Result<u64, AppError> {
        Err(AppError::Database("RocksDB feature not enabled".to_string()))
    }

    #[cfg(feature = "rocksdb-backend")]
    async fn update_last_processed_block(&self, chain: &str, block_number: u64) -> Result<(), AppError> {
        let key = format!("last_block:{}", chain);
        let value = block_number.to_string();
        self.db.put(key.as_bytes(), value.as_bytes())
            .map_err(|e| AppError::Database(format!("RocksDB put failed: {}", e)))?;
        Ok(())
    }

    #[cfg(not(feature = "rocksdb-backend"))]
    async fn update_last_processed_block(&self, _chain: &str, _block_number: u64) -> Result<(), AppError> {
        Err(AppError::Database("RocksDB feature not enabled".to_string()))
    }

    async fn init_last_processed_block(&self, chain: &str, initial_block: u64) -> Result<(), AppError> {
        let existing = self.get_last_processed_block(chain).await?;

        if existing != 0 {
            info!("{} already has last processed block: {}, skipping initialization", chain, existing);
            return Ok(());
        }

        let init_block = if initial_block > 0 { initial_block - 1 } else { 0 };
        self.update_last_processed_block(chain, init_block).await?;
        info!("Initialized {} last processed block to {} (will start from block {})", chain, init_block, initial_block);
        Ok(())
    }

    #[cfg(feature = "rocksdb-backend")]
    async fn is_monitored_address(&self, address: &str, chain_name: &str) -> Result<bool, AppError> {
        use crate::respository::is_monitored_address_in_rocksdb;
        is_monitored_address_in_rocksdb(&self.db, address, chain_name)
    }

    #[cfg(not(feature = "rocksdb-backend"))]
    async fn is_monitored_address(&self, _address: &str, _chain_name: &str) -> Result<bool, AppError> {
        Err(AppError::Database("RocksDB feature not enabled".to_string()))
    }

    #[cfg(feature = "rocksdb-backend")]
    async fn save_deposit_event(
        &self,
        address: &str,
        wallet_id: &str,
        account_id: Option<&str>,
        chain_name: &str,
        tx_hash: &str,
        block_number: u64,
        amount: &str,
        amount_decimal: Option<Decimal>,
    ) -> Result<(), AppError> {
        let key = format!("deposit:{}:{}", chain_name, tx_hash);

        // 중복 체크
        if self.db.get(key.as_bytes()).is_ok_and(|v| v.is_some()) {
            return Ok(()); // 이미 존재하면 무시
        }

        // DepositEvent를 JSON으로 직렬화하여 저장
        let event = serde_json::json!({
            "address": address,
            "wallet_id": wallet_id,
            "account_id": account_id,
            "chain_name": chain_name,
            "tx_hash": tx_hash,
            "block_number": block_number,
            "amount": amount,
            "amount_decimal": amount_decimal.map(|d| d.to_string()),
        });

        let value = serde_json::to_string(&event)
            .map_err(|e| AppError::Database(format!("Failed to serialize deposit event: {}", e)))?;

        self.db.put(key.as_bytes(), value.as_bytes())
            .map_err(|e| AppError::Database(format!("RocksDB put failed: {}", e)))?;

        Ok(())
    }

    #[cfg(not(feature = "rocksdb-backend"))]
    async fn save_deposit_event(
        &self,
        _address: &str,
        _wallet_id: &str,
        _account_id: Option<&str>,
        _chain_name: &str,
        _tx_hash: &str,
        _block_number: u64,
        _amount: &str,
        _amount_decimal: Option<Decimal>,
    ) -> Result<(), AppError> {
        Err(AppError::Database("RocksDB feature not enabled".to_string()))
    }

    #[cfg(feature = "rocksdb-backend")]
    async fn get_address_metadata(&self, address: &str, chain_name: &str) -> Result<Option<(String, Option<String>)>, AppError> {
        match crate::respository::get_address_metadata_from_rocksdb(&self.db, address, chain_name) {
            Ok(Some(metadata)) => Ok(Some((metadata.wallet_id, metadata.account_id))),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[cfg(not(feature = "rocksdb-backend"))]
    async fn get_address_metadata(&self, _address: &str, _chain_name: &str) -> Result<Option<(String, Option<String>)>, AppError> {
        Err(AppError::Database("RocksDB feature not enabled".to_string()))
    }

    // Note: increment_customer_balance removed
    // Balance management is handled by blockbit-back-custody, not xScanner

    async fn load_customer_addresses(&self, chain_name: &str) -> Result<usize, AppError> {
        // RocksDB는 키-값 저장소이므로 특정 체인의 주소 개수를 세려면 반복이 필요
        // 현재는 단순히 로그만 남기고 0 반환
        // 실제로는 접두사 검색을 통해 주소 개수를 셀 수 있음
        info!("RocksDBRepository: load_customer_addresses called for {}, but counting addresses requires iteration", chain_name);
        Ok(0) // 주소 개수를 세려면 반복이 필요하므로 0 반환
    }

    #[cfg(feature = "rocksdb-backend")]
    async fn deposit_exists(&self, tx_hash: &str, chain_name: &str) -> Result<bool, AppError> {
        let key = format!("deposit:{}:{}", chain_name, tx_hash);
        match self.db.get(key.as_bytes()) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(AppError::Database(format!("RocksDB get failed: {}", e))),
        }
    }

    #[cfg(not(feature = "rocksdb-backend"))]
    async fn deposit_exists(&self, _tx_hash: &str, _chain_name: &str) -> Result<bool, AppError> {
        Err(AppError::Database("RocksDB feature not enabled".to_string()))
    }

    #[cfg(feature = "rocksdb-backend")]
    async fn is_deposit_confirmed(&self, tx_hash: &str) -> Result<bool, AppError> {
        let key = format!("deposit_confirmed:{}", tx_hash);
        match self.db.get(key.as_bytes()) {
            Ok(Some(value)) => {
                let confirmed_str = String::from_utf8(value.to_vec())
                    .map_err(|e| AppError::Database(format!("Invalid UTF-8: {}", e)))?;
                Ok(confirmed_str == "true")
            }
            Ok(None) => Ok(false),
            Err(e) => Err(AppError::Database(format!("RocksDB get failed: {}", e))),
        }
    }

    #[cfg(not(feature = "rocksdb-backend"))]
    async fn is_deposit_confirmed(&self, _tx_hash: &str) -> Result<bool, AppError> {
        Err(AppError::Database("RocksDB feature not enabled".to_string()))
    }
}

#[cfg(feature = "rocksdb-backend")]
impl RocksDBRepository {
    pub async fn update_deposit_confirmed(&self, tx_hash: &str) -> Result<(), AppError> {
        let key = format!("deposit_confirmed:{}", tx_hash);
        self.db.put(key.as_bytes(), b"true")
            .map_err(|e| AppError::Database(format!("RocksDB put failed: {}", e)))?;
        Ok(())
    }
}
