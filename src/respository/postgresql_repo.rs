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
    
    async fn get_customer_id_by_address(&self, address: &str, chain_name: &str) -> Result<Option<String>, AppError> {
        crate::respository::postgresql::get_customer_id_by_address(&self.pool, address, chain_name).await
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
        crate::respository::postgresql::save_deposit_event(
            &self.pool,
            customer_id,
            address,
            chain_name,
            tx_hash,
            block_number,
            amount,
            amount_decimal,
        ).await
    }
    
    async fn increment_customer_balance(
        &self,
        customer_id: &str,
        chain_name: &str,
        amount: Decimal,
    ) -> Result<(), AppError> {
        crate::respository::postgresql::increment_customer_balance(
            &self.pool,
            customer_id,
            chain_name,
            amount,
        ).await
    }
    
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
}
