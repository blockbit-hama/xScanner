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
    
    /// 주소가 관리 대상인지 확인 (RocksDB 캐시 조회용)
    async fn is_monitored_address(&self, address: &str, chain_name: &str) -> Result<bool, AppError>;

    /// 입금 이벤트 저장 (wallet_id, account_id 추가)
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
    ) -> Result<(), AppError>;

    /// 주소의 메타데이터 조회 (wallet_id, account_id)
    async fn get_address_metadata(&self, address: &str, chain_name: &str) -> Result<Option<(String, Option<String>)>, AppError>;

    // Note: increment_customer_balance removed
    // Balance management is handled by blockbit-back-custody, not xScanner

    /// ?? ???? ?? (????)
    async fn load_customer_addresses(&self, chain_name: &str) -> Result<usize, AppError>;

    /// Check if a deposit already exists in the database
    async fn deposit_exists(&self, tx_hash: &str, chain_name: &str) -> Result<bool, AppError>;

    /// Check if a deposit is already confirmed
    async fn is_deposit_confirmed(&self, tx_hash: &str) -> Result<bool, AppError>;

    /// Get all pending (unconfirmed) deposits for confirmation checking
    async fn get_pending_deposits(&self) -> Result<Vec<crate::tasks::PendingDeposit>, AppError>;
}
