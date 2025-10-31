use crate::types::AppError;
use async_trait::async_trait;
use rust_decimal::Decimal;

/// Repository trait - ?? ??? ??? ???? ?? ?????
#[async_trait]
pub trait Repository: Send + Sync {
    /// ????? ??? ?? ?? ??
    async fn get_last_processed_block(&self, chain: &str) -> Result<u64, AppError>;
    
    /// ????? ??? ?? ?? ????
    async fn update_last_processed_block(&self, chain: &str, block_number: u64) -> Result<(), AppError>;
    
    /// ????? ??? ?? ?? ???
    async fn init_last_processed_block(&self, chain: &str, initial_block: u64) -> Result<(), AppError>;
    
    /// ??? ?? ID ??
    async fn get_customer_id_by_address(&self, address: &str, chain_name: &str) -> Result<Option<String>, AppError>;
    
    /// ?? ??? ??
    async fn save_deposit_event(
        &self,
        customer_id: &str,
        address: &str,
        chain_name: &str,
        tx_hash: &str,
        block_number: u64,
        amount: &str,
        amount_decimal: Option<Decimal>,
    ) -> Result<(), AppError>;
    
    /// ?? ?? ??
    async fn increment_customer_balance(
        &self,
        customer_id: &str,
        chain_name: &str,
        amount: Decimal,
    ) -> Result<(), AppError>;
    
    /// ?? ???? ?? (????)
    async fn load_customer_addresses(&self, chain_name: &str) -> Result<usize, AppError>;
}
