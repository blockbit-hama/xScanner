use crate::respository::{
    connect_db, setup_db_schema,
    Repository, MemoryRepository, PostgreSQLRepository,
};
use crate::config::Settings;
use crate::types::AppError;
use async_trait::async_trait;
use std::sync::Arc;

/// Repository wrapper - ??? ?? ??? Repository? ??
pub enum RepositoryWrapper {
    Memory(Arc<MemoryRepository>),
    PostgreSQL(Arc<PostgreSQLRepository>),
}

impl RepositoryWrapper {
    pub async fn from_settings(settings: &Settings) -> Result<Self, AppError> {
        if settings.repository.memory_db {
            Ok(RepositoryWrapper::Memory(Arc::new(MemoryRepository::new())))
        } else {
            let db_connection_pool = connect_db(&settings.repository.postgresql_url).await
                .map_err(|e| AppError::Database(format!("Failed to connect to PostgreSQL: {}", e)))?;
            setup_db_schema(&db_connection_pool).await
                .map_err(|e| AppError::Database(format!("Failed to setup database schema: {}", e)))?;
            Ok(RepositoryWrapper::PostgreSQL(Arc::new(PostgreSQLRepository::new(Arc::new(db_connection_pool)))))
        }
    }
    
    pub fn get_postgresql_repo(&self) -> Option<Arc<PostgreSQLRepository>> {
        match self {
            RepositoryWrapper::PostgreSQL(r) => Some(r.clone()),
            _ => None,
        }
    }
}

#[async_trait]
impl Repository for RepositoryWrapper {
    async fn get_last_processed_block(&self, chain: &str) -> Result<u64, AppError> {
        match self {
            RepositoryWrapper::Memory(r) => r.get_last_processed_block(chain).await,
            RepositoryWrapper::PostgreSQL(r) => r.get_last_processed_block(chain).await,
        }
    }
    
    async fn update_last_processed_block(&self, chain: &str, block_number: u64) -> Result<(), AppError> {
        match self {
            RepositoryWrapper::Memory(r) => r.update_last_processed_block(chain, block_number).await,
            RepositoryWrapper::PostgreSQL(r) => r.update_last_processed_block(chain, block_number).await,
        }
    }
    
    async fn init_last_processed_block(&self, chain: &str, initial_block: u64) -> Result<(), AppError> {
        match self {
            RepositoryWrapper::Memory(r) => r.init_last_processed_block(chain, initial_block).await,
            RepositoryWrapper::PostgreSQL(r) => r.init_last_processed_block(chain, initial_block).await,
        }
    }
    
    async fn get_customer_id_by_address(&self, address: &str, chain_name: &str) -> Result<Option<String>, AppError> {
        match self {
            RepositoryWrapper::Memory(r) => r.get_customer_id_by_address(address, chain_name).await,
            RepositoryWrapper::PostgreSQL(r) => r.get_customer_id_by_address(address, chain_name).await,
        }
    }
    
    async fn save_deposit_event(
        &self,
        customer_id: &str,
        address: &str,
        chain_name: &str,
        tx_hash: &str,
        block_number: u64,
        amount: &str,
        amount_decimal: Option<rust_decimal::Decimal>,
    ) -> Result<(), AppError> {
        match self {
            RepositoryWrapper::Memory(r) => r.save_deposit_event(customer_id, address, chain_name, tx_hash, block_number, amount, amount_decimal).await,
            RepositoryWrapper::PostgreSQL(r) => r.save_deposit_event(customer_id, address, chain_name, tx_hash, block_number, amount, amount_decimal).await,
        }
    }
    
    async fn increment_customer_balance(
        &self,
        customer_id: &str,
        chain_name: &str,
        amount: rust_decimal::Decimal,
    ) -> Result<(), AppError> {
        match self {
            RepositoryWrapper::Memory(r) => r.increment_customer_balance(customer_id, chain_name, amount).await,
            RepositoryWrapper::PostgreSQL(r) => r.increment_customer_balance(customer_id, chain_name, amount).await,
        }
    }
    
    async fn load_customer_addresses(&self, chain_name: &str) -> Result<usize, AppError> {
        match self {
            RepositoryWrapper::Memory(r) => r.load_customer_addresses(chain_name).await,
            RepositoryWrapper::PostgreSQL(r) => r.load_customer_addresses(chain_name).await,
        }
    }
}
