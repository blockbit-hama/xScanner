use crate::respository::Repository;
use crate::respository::RepositoryWrapper;
use std::sync::Arc;
use log::{error, info, warn};
use tokio::sync::mpsc::Receiver;
use rust_decimal::Decimal;
use crate::types::BlockData;

#[cfg(feature = "leveldb-backend")]
use crate::respository::get_customer_id_from_leveldb;
#[cfg(feature = "leveldb-backend")]
use leveldb::database::Database as LevelDB;

#[cfg(feature = "rocksdb-backend")]
use crate::respository::get_customer_id_from_rocksdb;
#[cfg(feature = "rocksdb-backend")]
use rocksdb::DB as RocksDB;

// Unified database type for conditional compilation
#[cfg(feature = "leveldb-backend")]
pub type KeyValueDB = LevelDB;

#[cfg(feature = "rocksdb-backend")]
pub type KeyValueDB = RocksDB;

#[cfg(not(any(feature = "leveldb-backend", feature = "rocksdb-backend")))]
pub type KeyValueDB = ();

#[derive(Debug, Clone)]
struct DepositInfo {
  address: String,
  customer_id: String,
  tx_hash: String,
  block_number: u64,
  amount: String,
  amount_decimal: Option<Decimal>,
}

pub async fn run_analyzer(
  mut receiver: Receiver<BlockData>,
  repository: Arc<RepositoryWrapper>,
  kv_db: Option<Arc<KeyValueDB>>,
) {
  info!("[Analyzer] Starting loop...");

  while let Some(block_data) = receiver.recv().await {
    info!("[Analyzer] 블록 데이터 수신! 분석 시작...");
    let repository_clone = repository.clone();
    let kv_db_clone = kv_db.clone();

    // 블록 분석 및 고객 주소 매칭 (KeyValueDB 또는 Repository 사용)
    let result = analyze_block(block_data, &repository_clone, kv_db_clone.as_deref()).await;

    match result {
      Ok((chain_name, block_number, deposits)) => {
        info!(
          "[Analyzer] Finished processing {} block {}, found {} deposits",
          chain_name, block_number, deposits.len()
        );

        // ?? ??
        for deposit in deposits {
          if let Err(e) = process_deposit(&repository_clone, &chain_name, deposit).await {
            error!("[Analyzer] Failed to process deposit: {}", e);
          }
        }

        // ??? ?? ?? ????
        if let Err(e) = repository_clone.update_last_processed_block(&chain_name, block_number).await {
          error!(
            "[Analyzer] Failed to update last processed block for {} block {}: {}",
            chain_name, block_number, e
          );
        }
      }
      Err(e) => {
        error!("[Analyzer] ❌ 블록 분석 실패: {}", e);
      }
    }
  }
  
  info!("[Analyzer] Loop finished because the channel was closed.");
}

async fn analyze_block(
  block_data: BlockData,
  repository: &Arc<RepositoryWrapper>,
  kv_db: Option<&KeyValueDB>,
) -> Result<(String, u64, Vec<DepositInfo>), String> {
  match block_data {
    BlockData::Ethereum(block) => analyze_ethereum_block(block, repository, kv_db).await,
    BlockData::Bitcoin(block) => analyze_bitcoin_block(block, repository, kv_db).await,
    BlockData::Tron(block) => analyze_tron_block(block, repository, kv_db).await,
    BlockData::Theta(block) => analyze_theta_block(block, repository, kv_db).await,
    BlockData::Icon(block) => analyze_icon_block(block, repository, kv_db).await,
  }
}

// 고객 ID 조회 함수 (KeyValueDB 우선, 없으면 Repository 사용)
async fn get_customer_id(
  repository: &Arc<RepositoryWrapper>,
  kv_db: Option<&KeyValueDB>,
  address: &str,
  chain_name: &str,
) -> Result<Option<String>, String> {
  // KeyValueDB가 있으면 고속 조회 (먼저 시도)
  #[cfg(feature = "leveldb-backend")]
  if let Some(kv_db) = kv_db {
    if let Ok(Some(customer_id)) = get_customer_id_from_leveldb(kv_db, address, chain_name) {
      return Ok(Some(customer_id));
    }
  }
  
  #[cfg(feature = "rocksdb-backend")]
  if let Some(kv_db) = kv_db {
    if let Ok(Some(customer_id)) = get_customer_id_from_rocksdb(kv_db, address, chain_name) {
      return Ok(Some(customer_id));
    }
  }
  
  #[cfg(not(any(feature = "leveldb-backend", feature = "rocksdb-backend")))]
  let _ = kv_db; // unused variable warning 방지

  // KeyValueDB가 없거나 KeyValueDB에 없으면 Repository 조회
  repository.get_customer_id_by_address(address, chain_name).await
    .map_err(|e| format!("Failed to get customer ID: {}", e))
}

async fn analyze_ethereum_block(
  block: crate::coin::ethereum::model::EthereumBlock,
  repository: &Arc<RepositoryWrapper>,
  kv_db: Option<&KeyValueDB>,
) -> Result<(String, u64, Vec<DepositInfo>), String> {
  let chain_name = "ETH";
  let result = block.result.ok_or("Missing 'result' in EthereumBlock")?;
  
  let number_hex = result.number.trim();
  let block_number = if number_hex.starts_with("0x") {
    u64::from_str_radix(&number_hex[2..], 16)
  } else {
    number_hex.parse::<u64>()
  }.map_err(|e| format!("Failed to parse ETH block number: {}", e))?;
  
  info!("[Analyzer] 🔍 블록 #{} 스캔 중... (트랜잭션: {}개)", block_number, result.transactions.len());

  let mut deposits = Vec::new();
  let mut checked_count = 0;

  for tx in &result.transactions {
    if let Some(to_address) = &tx.to {
      checked_count += 1;
      if let Ok(Some(customer_id)) = get_customer_id(repository, kv_db, to_address, chain_name).await {
        let amount_hex = tx.value.as_deref().unwrap_or("0x0");
        let amount_decimal = parse_wei_to_decimal(amount_hex).ok();

        info!("[Analyzer] ✅ 입금 감지! 블록: {} | 고객: {} | 주소: {} | 금액: {:?} ETH",
          block_number, customer_id, to_address, amount_decimal);

        deposits.push(DepositInfo {
          address: to_address.clone(),
          customer_id,
          tx_hash: tx.hash.as_deref().unwrap_or("").to_string(),
          block_number,
          amount: amount_hex.to_string(),
          amount_decimal,
        });
      }
    }
  }

  if checked_count > 0 && deposits.is_empty() {
    info!("[Analyzer] 블록 #{}: {}개 트랜잭션 확인, 고객 계정 없음", block_number, checked_count);
  }

  Ok((chain_name.to_string(), block_number, deposits))
}

async fn analyze_bitcoin_block(
  block: crate::coin::bitcoin::model::BitcoinBlock,
  repository: &Arc<RepositoryWrapper>,
  kv_db: Option<&KeyValueDB>,
) -> Result<(String, u64, Vec<DepositInfo>), String> {
  let chain_name = "BTC";
  let block_number = block.height;
  
  info!("[Analyzer] Bitcoin Block Received: {}", block_number);
  
  let mut deposits = Vec::new();
  
  for tx in &block.tx {
    for output in &tx.out {
      if let Some(address) = &output.addr {
        if let Ok(Some(customer_id)) = get_customer_id(repository, kv_db, address, chain_name).await {
          let amount_decimal = Decimal::from(output.value) / Decimal::from(100_000_000u64); // satoshi to BTC
          
          deposits.push(DepositInfo {
            address: address.clone(),
            customer_id,
            tx_hash: tx.hash.clone(),
            block_number,
            amount: output.value.to_string(),
            amount_decimal: Some(amount_decimal),
          });
        }
      }
    }
  }
  
  Ok((chain_name.to_string(), block_number, deposits))
}

async fn analyze_tron_block(
  block: crate::coin::tron::model::TronBlock,
  repository: &Arc<RepositoryWrapper>,
  kv_db: Option<&KeyValueDB>,
) -> Result<(String, u64, Vec<DepositInfo>), String> {
  let chain_name = "TRON";
  let block_number = block.block_header.raw_data.number;
  
  info!("[Analyzer] TRON Block Received: {}", block_number);
  
  let mut deposits = Vec::new();
  
  for tx in &block.transactions {
    // TRX ?? ??
    if let Some(contract) = tx.raw_data.contract.first() {
      if contract.contract_type == "TransferContract" {
        if let Some(to_address) = &contract.parameter.value.to_address {
          if let Ok(Some(customer_id)) = get_customer_id(repository, kv_db, to_address, chain_name).await {
            let amount = contract.parameter.value.amount.unwrap_or(0);
            let amount_decimal = Decimal::from(amount) / Decimal::from(1_000_000u64); // SUN to TRX
            
            deposits.push(DepositInfo {
              address: to_address.clone(),
              customer_id,
              tx_hash: tx.tx_id.clone(),
              block_number,
              amount: amount.to_string(),
              amount_decimal: Some(amount_decimal),
            });
          }
        }
      }
    }
  }
  
  Ok((chain_name.to_string(), block_number, deposits))
}

async fn analyze_theta_block(
  block: crate::coin::theta::model::ThetaBlock,
  repository: &Arc<RepositoryWrapper>,
  kv_db: Option<&KeyValueDB>,
) -> Result<(String, u64, Vec<DepositInfo>), String> {
  let chain_name = "THETA";
  let block_number = block.height.parse::<u64>()
    .map_err(|e| format!("Failed to parse THETA block number: {}", e))?;

  info!("[Analyzer] THETA Block Received: {}", block_number);

  let mut deposits = Vec::new();

  // THETA 트랜잭션 분석 (Ethereum 호환 RPC 형식)
  for tx in &block.transactions {
    if let Some(to_address) = &tx.to {
      if let Ok(Some(customer_id)) = get_customer_id(repository, kv_db, to_address, chain_name).await {
        // value는 Wei 단위 (1 THETA = 10^18 Wei)
        let value_hex = if tx.value.starts_with("0x") {
          &tx.value
        } else {
          &format!("0x{}", tx.value)
        };

        let amount_decimal = parse_wei_to_decimal(value_hex).ok();

        deposits.push(DepositInfo {
          address: to_address.clone(),
          customer_id,
          tx_hash: tx.hash.clone(),
          block_number,
          amount: tx.value.clone(),
          amount_decimal,
        });
      }
    }
  }

  if !deposits.is_empty() {
    info!("[Analyzer] Found {} THETA deposits in block {}", deposits.len(), block_number);
  }

  Ok((chain_name.to_string(), block_number, deposits))
}

async fn analyze_icon_block(
  block: crate::coin::icon::model::IconBlock,
  repository: &Arc<RepositoryWrapper>,
  kv_db: Option<&KeyValueDB>,
) -> Result<(String, u64, Vec<DepositInfo>), String> {
  let chain_name = "ICON";
  let block_number = block.height;
  
  info!("[Analyzer] ICON Block Received: {}", block_number);
  
  let mut deposits = Vec::new();
  
  for tx in &block.confirmed_transaction_list {
    let to_address = &tx.to;
    
      if let Ok(Some(customer_id)) = get_customer_id(repository, kv_db, to_address, chain_name).await {
      let amount_decimal = tx.value.as_ref()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| Decimal::from(v) / Decimal::from(1_000_000_000_000_000_000u64)); // Loop to ICX
      
      deposits.push(DepositInfo {
        address: to_address.clone(),
        customer_id,
        tx_hash: tx.tx_hash.clone(),
        block_number,
        amount: tx.value.clone().unwrap_or_default(),
        amount_decimal,
      });
    }
  }
  
  Ok((chain_name.to_string(), block_number, deposits))
}

async fn process_deposit(
  repository: &Arc<RepositoryWrapper>,
  chain_name: &str,
  deposit: DepositInfo,
) -> Result<(), String> {
  info!(
    "[DEPOSIT] Customer {} received {} {} at address {} (tx: {}, block: {})",
    deposit.customer_id, deposit.amount, chain_name, deposit.address, deposit.tx_hash, deposit.block_number
  );
  
  // ?? ??? ??
  repository.save_deposit_event(
    &deposit.customer_id,
    &deposit.address,
    chain_name,
    &deposit.tx_hash,
    deposit.block_number,
    &deposit.amount,
    deposit.amount_decimal,
  )
  .await
  .map_err(|e| format!("Failed to save deposit event: {}", e))?;
  
  // ?? ?? ??
  if let Some(amount_decimal) = deposit.amount_decimal {
    repository.increment_customer_balance(&deposit.customer_id, chain_name, amount_decimal)
      .await
      .map_err(|e| format!("Failed to update balance: {}", e))?;
    
    info!(
      "[DEPOSIT] Updated balance for customer {}: +{} {}",
      deposit.customer_id, amount_decimal, chain_name
    );
  } else {
    warn!(
      "[DEPOSIT] Could not parse amount for deposit: {}",
      deposit.amount
    );
  }
  
  Ok(())
}

// Wei? Decimal? ??
fn parse_wei_to_decimal(wei_hex: &str) -> Result<Decimal, String> {
  let wei_str = if wei_hex.starts_with("0x") || wei_hex.starts_with("0X") {
    &wei_hex[2..]
  } else {
    wei_hex
  };
  
  let wei = u128::from_str_radix(wei_str, 16)
    .map_err(|e| format!("Failed to parse wei: {}", e))?;
  
  // Wei? ETH? ?? (1 ETH = 10^18 Wei)
  let eth = Decimal::from(wei) / Decimal::from(1_000_000_000_000_000_000u128);
  
  Ok(eth)
}
