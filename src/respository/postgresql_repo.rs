use crate::types::AppError;
use crate::respository::r#trait::Repository;
use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use log::info;

/// PostgreSQL ?? Repository ??
#[derive(Clone)]
pub struct PostgreSQLRepository {
    pool: Arc<PgPool>,
}

impl PostgreSQLRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
    
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl Repository for PostgreSQLRepository {
    async fn get_last_processed_block(&self, chain: &str) -> Result<u64, AppError> {
        crate::respository::postgresql::get_last_processed_block(&self.pool, chain).await
    }
    
    async fn update_last_processed_block(&self, chain: &str, block_number: u64) -> Result<(), AppError> {
        crate::respository::postgresql::update_last_processed_block(&self.pool, chain, block_number).await
    }
    
    async fn init_last_processed_block(&self, chain: &str, initial_block: u64) -> Result<(), AppError> {
        crate::respository::postgresql::init_last_processed_block(&self.pool, chain, initial_block).await
    }
    
    async fn is_monitored_address(&self, _address: &str, _chain_name: &str) -> Result<bool, AppError> {
        // Note: This function is no longer used in PostgreSQL repository
        // All address lookups are done via RocksDB cache
        // (populated from Backend via SQS + optional file cache)
        Ok(false)
    }

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
        crate::respository::postgresql::save_deposit_event(
            &self.pool,
            address,
            wallet_id,
            account_id,
            chain_name,
            tx_hash,
            block_number,
            amount,
            amount_decimal,
        ).await
    }

    async fn get_address_metadata(&self, _address: &str, _chain_name: &str) -> Result<Option<(String, Option<String>)>, AppError> {
        // PostgreSQL repository doesn't store address metadata
        // All address metadata lookups are done via RocksDB cache
        Ok(None)
    }

    // Note: increment_customer_balance removed
    // Balance management is handled by blockbit-back-custody, not xScanner

    async fn load_customer_addresses(&self, chain_name: &str) -> Result<usize, AppError> {
        // PostgreSQL??? ???? LevelDB? ???? ??? ???,
        // ???? ?? ??? ??
        use sqlx::Row;
        let query = format!(
            "SELECT COUNT(*) FROM {} WHERE chain_name = $1",
            crate::respository::postgresql::CUSTOMER_ADDRESSES_TABLE
        );
        
        let row = sqlx::query(&query)
            .bind(chain_name)
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| AppError::Database(format!("Failed to count customer addresses: {}", e)))?;
        
        let count: i64 = row.try_get(0)
            .map_err(|e| AppError::Database(format!("Failed to get count: {}", e)))?;
        
        Ok(count as usize)
    }

    async fn deposit_exists(&self, tx_hash: &str, chain_name: &str) -> Result<bool, AppError> {
        crate::respository::postgresql::deposit_exists(&self.pool, tx_hash, chain_name).await
    }

    async fn is_deposit_confirmed(&self, tx_hash: &str) -> Result<bool, AppError> {
        crate::respository::postgresql::is_deposit_confirmed(&self.pool, tx_hash).await
    }

    async fn get_pending_deposits(&self) -> Result<Vec<crate::tasks::PendingDeposit>, AppError> {
        crate::respository::postgresql::get_pending_deposits(&self.pool).await
    }
}
