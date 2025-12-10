use crate::types::AppError;
use crate::respository::r#trait::Repository;
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn};

/// ??? ?? Repository ??
/// ???? ?? ???? ??
#[derive(Clone)]
pub struct MemoryRepository {
    // chain_name -> last_processed_block
    last_processed_blocks: Arc<RwLock<HashMap<String, u64>>>,
    
    // chain_name:address -> monitored flag (customer_id 제거됨)
    monitored_addresses: Arc<RwLock<HashMap<String, bool>>>, // key: "chain_name:address", value: true
    
    // (chain_name, tx_hash) -> deposit_event
    deposit_events: Arc<RwLock<HashMap<(String, String), DepositEvent>>>,
    
    // (customer_id, chain_name) -> balance
    customer_balances: Arc<RwLock<HashMap<(String, String), Decimal>>>,
}

#[derive(Clone)]
struct DepositEvent {
    address: String,
    chain_name: String,
    tx_hash: String,
    block_number: u64,
    amount: String,
    amount_decimal: Option<Decimal>,
    confirmed: bool,
}

impl MemoryRepository {
    pub fn new() -> Self {
        Self {
            last_processed_blocks: Arc::new(RwLock::new(HashMap::new())),
            monitored_addresses: Arc::new(RwLock::new(HashMap::new())),
            deposit_events: Arc::new(RwLock::new(HashMap::new())),
            customer_balances: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Repository for MemoryRepository {
    async fn get_last_processed_block(&self, chain: &str) -> Result<u64, AppError> {
        let blocks = self.last_processed_blocks.read().await;
        Ok(blocks.get(chain).copied().unwrap_or(0))
    }
    
    async fn update_last_processed_block(&self, chain: &str, block_number: u64) -> Result<(), AppError> {
        let mut blocks = self.last_processed_blocks.write().await;
        blocks.insert(chain.to_string(), block_number);
        Ok(())
    }
    
    async fn init_last_processed_block(&self, chain: &str, initial_block: u64) -> Result<(), AppError> {
        let mut blocks = self.last_processed_blocks.write().await;
        
        // ?? ?? ??? ??
        if blocks.contains_key(chain) {
            let existing = blocks.get(chain).copied().unwrap_or(0);
            if existing != 0 {
                info!("{} already has last processed block: {}, skipping initialization", chain, existing);
                return Ok(());
            }
        }
        
        let init_block = if initial_block > 0 { initial_block - 1 } else { 0 };
        blocks.insert(chain.to_string(), init_block);
        info!("Initialized {} last processed block to {} (will start from block {})", chain, init_block, initial_block);
        Ok(())
    }
    
    async fn is_monitored_address(&self, address: &str, chain_name: &str) -> Result<bool, AppError> {
        let addresses = self.monitored_addresses.read().await;
        let normalized_address = address.to_lowercase();
        let key = format!("{}:{}", chain_name.to_lowercase(), normalized_address);

        Ok(addresses.contains_key(&key))
    }

    async fn save_deposit_event(
        &self,
        address: &str,
        _wallet_id: &str,
        _account_id: Option<&str>,
        chain_name: &str,
        tx_hash: &str,
        block_number: u64,
        amount: &str,
        amount_decimal: Option<Decimal>,
    ) -> Result<(), AppError> {
        let mut events = self.deposit_events.write().await;
        let key = (chain_name.to_string(), tx_hash.to_string());

        // 중복 체크 (UNIQUE 제약)
        if events.contains_key(&key) {
            return Ok(());
        }

        events.insert(key, DepositEvent {
            address: address.to_string(),
            chain_name: chain_name.to_string(),
            tx_hash: tx_hash.to_string(),
            block_number,
            amount: amount.to_string(),
            amount_decimal,
            confirmed: false,
        });

        Ok(())
    }

    async fn get_address_metadata(&self, _address: &str, _chain_name: &str) -> Result<Option<(String, Option<String>)>, AppError> {
        // MemoryRepository doesn't store address metadata
        Ok(None)
    }

    // Note: increment_customer_balance removed
    // Balance management is handled by blockbit-back-custody, not xScanner

    async fn load_customer_addresses(&self, chain_name: &str) -> Result<usize, AppError> {
        // 메모리 저장소에서 특정 체인의 주소 개수 반환
        let addresses = self.monitored_addresses.read().await;
        let prefix = format!("{}:", chain_name.to_lowercase());
        let count = addresses.iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .count();

        warn!("MemoryRepository: load_customer_addresses called for {}, found {} addresses", chain_name, count);
        Ok(count)
    }

    async fn deposit_exists(&self, tx_hash: &str, chain_name: &str) -> Result<bool, AppError> {
        let events = self.deposit_events.read().await;
        let key = (chain_name.to_string(), tx_hash.to_string());
        Ok(events.contains_key(&key))
    }

    async fn is_deposit_confirmed(&self, tx_hash: &str) -> Result<bool, AppError> {
        let events = self.deposit_events.read().await;

        // Find the deposit event by tx_hash (search across all chains)
        for ((_, hash), event) in events.iter() {
            if hash == tx_hash {
                return Ok(event.confirmed);
            }
        }

        Ok(false)
    }

    async fn get_pending_deposits(&self) -> Result<Vec<crate::tasks::PendingDeposit>, AppError> {
        let events = self.deposit_events.read().await;

        let mut deposits = Vec::new();
        for ((_, _), event) in events.iter() {
            if !event.confirmed {
                deposits.push(crate::tasks::PendingDeposit {
                    address: event.address.clone(),
                    wallet_id: String::new(), // MemoryRepository doesn't store wallet_id
                    account_id: None,
                    chain_name: event.chain_name.clone(),
                    tx_hash: event.tx_hash.clone(),
                    block_number: event.block_number,
                    amount: event.amount.clone(),
                    amount_decimal: event.amount_decimal,
                });
            }
        }

        Ok(deposits)
    }
}

// Helper function to update deposit confirmation status for MemoryRepository
impl MemoryRepository {
    pub async fn update_deposit_confirmed(&self, tx_hash: &str) -> Result<(), AppError> {
        let mut events = self.deposit_events.write().await;

        // Find and update the deposit event
        for ((_, hash), event) in events.iter_mut() {
            if hash == tx_hash {
                event.confirmed = true;
                return Ok(());
            }
        }

        Err(AppError::Database(format!("Deposit not found: {}", tx_hash)))
    }
}

impl Default for MemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}
