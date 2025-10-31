use crate::types::AppError;
use crate::respository::r#trait::Repository;
use async_trait::async_trait;
use rust_decimal::Decimal;
use leveldb::database::Database;
use leveldb::kv::KV;
use leveldb::options::{ReadOptions, WriteOptions};
use leveldb::batch::WriteBatch;
use std::sync::Arc;
use log::info;

/// LevelDB ?? Repository ??
#[derive(Clone)]
pub struct LevelDBRepository {
    db: Arc<Database<String>>,
}

impl LevelDBRepository {
    pub fn new(db: Arc<Database<String>>) -> Self {
        Self { db }
    }
    
    pub fn db(&self) -> &Database<String> {
        &self.db
    }
    
    /// ??? ?? ID ?? (LevelDB?? ??? ??)
    pub fn get_customer_id_from_leveldb(
        &self,
        address: &str,
    ) -> Result<Option<String>, AppError> {
        let read_options = ReadOptions::new();
        let normalized_address = address.to_lowercase();
        match self.db.get(read_options, normalized_address) {
            Ok(Some(customer_id)) => Ok(Some(customer_id)),
            Ok(None) => Ok(None),
            Err(e) => Err(AppError::Database(format!("LevelDB get failed: {}", e))),
        }
    }
}

#[async_trait]
impl Repository for LevelDBRepository {
    async fn get_last_processed_block(&self, chain: &str) -> Result<u64, AppError> {
        let read_options = ReadOptions::new();
        let key = format!("last_block:{}", chain);
        match self.db.get(read_options, key.clone()) {
            Ok(Some(value)) => {
                let block_str = String::from_utf8_lossy(&value);
                block_str.parse::<u64>()
                    .map_err(|e| AppError::Database(format!("Failed to parse block number: {}", e)))
            }
            Ok(None) => Ok(0),
            Err(e) => Err(AppError::Database(format!("LevelDB get failed: {}", e))),
        }
    }
    
    async fn update_last_processed_block(&self, chain: &str, block_number: u64) -> Result<(), AppError> {
        let write_options = WriteOptions::new();
        let key = format!("last_block:{}", chain);
        let value = block_number.to_string();
        self.db.put(write_options, key, value)
            .map_err(|e| AppError::Database(format!("LevelDB put failed: {}", e)))?;
        Ok(())
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
    
    async fn get_customer_id_by_address(&self, address: &str, _chain_name: &str) -> Result<Option<String>, AppError> {
        // LevelDB? ??? ?? ????? chain_name? ??
        self.get_customer_id_from_leveldb(address)
    }
    
    async fn save_deposit_event(
        &self,
        customer_id: &str,
        address: &str,
        chain_name: &str,
        tx_hash: &str,
        block_number: u64,
        amount: &str,
        amount_decimal: Option<Decimal>,
    ) -> Result<(), AppError> {
        let write_options = WriteOptions::new();
        let key = format!("deposit:{}:{}", chain_name, tx_hash);
        
        // ?? ??
        let read_options = ReadOptions::new();
        if self.db.get(read_options.clone(), key.clone()).is_ok_and(|v| v.is_some()) {
            return Ok(()); // ?? ???? ??
        }
        
        // DepositEvent? JSON?? ????? ??
        let event = serde_json::json!({
            "customer_id": customer_id,
            "address": address,
            "chain_name": chain_name,
            "tx_hash": tx_hash,
            "block_number": block_number,
            "amount": amount,
            "amount_decimal": amount_decimal.map(|d| d.to_string()),
        });
        
        let value = serde_json::to_string(&event)
            .map_err(|e| AppError::Database(format!("Failed to serialize deposit event: {}", e)))?;
        
        self.db.put(write_options, key, value)
            .map_err(|e| AppError::Database(format!("LevelDB put failed: {}", e)))?;
        
        Ok(())
    }
    
    async fn increment_customer_balance(
        &self,
        customer_id: &str,
        chain_name: &str,
        amount: Decimal,
    ) -> Result<(), AppError> {
        let key = format!("balance:{}:{}", customer_id, chain_name);
        let read_options = ReadOptions::new();
        
        let current_balance = match self.db.get(read_options, key.clone()) {
            Ok(Some(value)) => {
                let balance_str = String::from_utf8_lossy(&value);
                balance_str.parse::<Decimal>()
                    .unwrap_or(Decimal::ZERO)
            }
            Ok(None) => Decimal::ZERO,
            Err(_) => Decimal::ZERO,
        };
        
        let new_balance = current_balance + amount;
        let write_options = WriteOptions::new();
        self.db.put(write_options, key, new_balance.to_string())
            .map_err(|e| AppError::Database(format!("LevelDB put failed: {}", e)))?;
        
        Ok(())
    }
    
    async fn load_customer_addresses(&self, chain_name: &str) -> Result<usize, AppError> {
        // LevelDB? ?-? ?????? ?? ?? ???? ?
        // ???? ?? ??? ?? ???? ?? ??? ??
        // ???? ?? ??? ???? ????? 0 ??
        // ?? ????? ??? ???? ???? ???? ? ? ??
        info!("LevelDBRepository: load_customer_addresses called for {}, but counting addresses requires iteration", chain_name);
        Ok(0) // ?? ???? ???? ????? 0 ??
    }
}
