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
    
    // chain_name:address -> customer_id (???? ??)
    customer_addresses: Arc<RwLock<HashMap<String, String>>>, // key: "chain_name:address", value: customer_id
    
    // (chain_name, tx_hash) -> deposit_event
    deposit_events: Arc<RwLock<HashMap<(String, String), DepositEvent>>>,
    
    // (customer_id, chain_name) -> balance
    customer_balances: Arc<RwLock<HashMap<(String, String), Decimal>>>,
}

#[derive(Clone)]
struct DepositEvent {
    customer_id: String,
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
            customer_addresses: Arc::new(RwLock::new(HashMap::new())),
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
    
    async fn get_customer_id_by_address(&self, address: &str, chain_name: &str) -> Result<Option<String>, AppError> {
        let addresses = self.customer_addresses.read().await;
        let normalized_address = address.to_lowercase();
        let key = format!("{}:{}", chain_name.to_lowercase(), normalized_address);
        
        // chain_name:address ??? ?? ?? ??
        Ok(addresses.get(&key).cloned())
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
        let mut events = self.deposit_events.write().await;
        let key = (chain_name.to_string(), tx_hash.to_string());
        
        // ?? ???? ?? (UNIQUE ??)
        if events.contains_key(&key) {
            return Ok(());
        }
        
        events.insert(key, DepositEvent {
            customer_id: customer_id.to_string(),
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
    
    async fn increment_customer_balance(
        &self,
        customer_id: &str,
        chain_name: &str,
        amount: Decimal,
    ) -> Result<(), AppError> {
        let mut balances = self.customer_balances.write().await;
        let key = (customer_id.to_string(), chain_name.to_string());
        
        let current_balance = balances.get(&key).copied().unwrap_or(Decimal::ZERO);
        balances.insert(key, current_balance + amount);
        
        Ok(())
    }
    
    async fn load_customer_addresses(&self, chain_name: &str) -> Result<usize, AppError> {
        // ??? ?????? ?? ??? ???? ???? ?? ???? ??
        let addresses = self.customer_addresses.read().await;
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
