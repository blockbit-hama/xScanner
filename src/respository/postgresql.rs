// src/db.rs
use crate::types::AppError;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use log::{info, warn};
use std::str::FromStr;


pub async fn connect_db(db_url: &str) -> Result<PgPool, sqlx::Error> {
  PgPoolOptions::new()
    .max_connections(5) // Adjust pool size as needed
    .connect(db_url)
    .await
}

pub const STATE_TABLE_NAME: &str = "blockchain_state";
pub const CUSTOMER_ADDRESSES_TABLE: &str = "customer_addresses";
pub const DEPOSIT_EVENTS_TABLE: &str = "deposit_events";
pub const CUSTOMER_BALANCES_TABLE: &str = "customer_balances";

// Ensure all tables exist
pub async fn setup_db_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
  // ???? ?? ???
  sqlx::query(&format!(
    r#"
        CREATE TABLE IF NOT EXISTS {} (
            chain_name VARCHAR(50) PRIMARY KEY,
            last_processed_block BIGINT NOT NULL
        )
        "#,
    STATE_TABLE_NAME
  ))
    .execute(pool)
    .await?;
  
  // Note: customer_addresses table removed
  // Customer addresses are managed by blockbit-back-custody (Backend)
  // xScanner receives address updates via SQS and caches them in RocksDB only
  
  // 입금 이벤트 테이블 (wallet_id, account_id 추가)
  sqlx::query(&format!(
    r#"
        CREATE TABLE IF NOT EXISTS {} (
            id SERIAL PRIMARY KEY,
            address VARCHAR(255) NOT NULL,
            wallet_id VARCHAR(255) NOT NULL,
            account_id VARCHAR(255),
            chain_name VARCHAR(50) NOT NULL,
            tx_hash VARCHAR(255) NOT NULL,
            block_number BIGINT NOT NULL,
            amount VARCHAR(255) NOT NULL,
            amount_decimal NUMERIC(36, 18),
            confirmed BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(chain_name, tx_hash)
        )
        "#,
    DEPOSIT_EVENTS_TABLE
  ))
    .execute(pool)
    .await?;

  // 인덱스 생성
  let _ = sqlx::query(&format!("CREATE INDEX IF NOT EXISTS idx_de_address ON {} (address)", DEPOSIT_EVENTS_TABLE))
    .execute(pool)
    .await;
  let _ = sqlx::query(&format!("CREATE INDEX IF NOT EXISTS idx_de_block_number ON {} (block_number)", DEPOSIT_EVENTS_TABLE))
    .execute(pool)
    .await;

  // Note: customer_balances table is NOT created by xScanner
  // Balance management is handled by blockbit-back-custody
  // xScanner only logs deposit events for audit purposes

  Ok(())
}


pub async fn get_last_processed_block(
  pool: &PgPool,
  chain: &str,
) -> Result<u64, AppError> {
  let row: Option<(i64,)> = sqlx::query_as(&format!( // Use query_as for type mapping
                                                     "SELECT last_processed_block FROM {} WHERE chain_name = $1",
                                                     STATE_TABLE_NAME
  ))
    .bind(chain)
    .fetch_optional(pool) // Use fetch_optional to handle no row case
    .await
    .map_err(|e| AppError::Initialization(format!("DB query failed: {}", e)))?;
  
  // If row exists, return the value, otherwise return 0 (or configured start block)
  Ok(row.map_or(0, |(block_num,)| block_num as u64))
}


pub async fn update_last_processed_block(
  pool: &PgPool,
  chain: &str,
  block_number: u64,
) -> Result<(), AppError> {
  let query = format!(
    r#"
        INSERT INTO {0} (chain_name, last_processed_block)
        VALUES ($1, $2)
        ON CONFLICT (chain_name) DO UPDATE SET last_processed_block = $2
        "#,
    STATE_TABLE_NAME
  );
  
  sqlx::query(&query)
    .bind(chain)
    .bind(block_number as i64) // Bind as i64 if column is BIGINT
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to update last block: {}", e)))?; // Use specific error variant
  Ok(())
}

// ?? ?? ??? DB? ?? (?? ?? ??)
pub async fn init_last_processed_block(
  pool: &PgPool,
  chain: &str,
  initial_block: u64,
) -> Result<(), AppError> {
  // ?? ?? ??? ??
  let existing = get_last_processed_block(pool, chain).await?;
  
  // ?? ??? 0?? ??? ?? (start_block - 1? ???? ??? start_block?? ??)
  if existing == 0 {
    let init_block = if initial_block > 0 { initial_block - 1 } else { 0 };
    update_last_processed_block(pool, chain, init_block).await?;
    info!("Initialized {} last processed block to {} (will start from block {})", chain, init_block, initial_block);
  } else {
    info!("{} already has last processed block: {}, skipping initialization", chain, existing);
  }
  
  Ok(())
}

// Note: get_customer_id_by_address removed
// xScanner no longer queries customer addresses from PostgreSQL
// All address lookups are done via RocksDB cache (populated from Backend via SQS)

// 입금 이벤트 저장 (wallet_id, account_id 추가)
pub async fn save_deposit_event(
  pool: &PgPool,
  address: &str,
  wallet_id: &str,
  account_id: Option<&str>,
  chain_name: &str,
  tx_hash: &str,
  block_number: u64,
  amount: &str,
  amount_decimal: Option<rust_decimal::Decimal>,
) -> Result<(), AppError> {
  let query = format!(
    r#"
        INSERT INTO {} (address, wallet_id, account_id, chain_name, tx_hash, block_number, amount, amount_decimal)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (chain_name, tx_hash) DO NOTHING
        "#,
    DEPOSIT_EVENTS_TABLE
  );

  // Convert rust_decimal::Decimal to bigdecimal::BigDecimal for sqlx
  let amount_decimal_bigdecimal = amount_decimal.map(|d| {
    bigdecimal::BigDecimal::from_str(&d.to_string())
      .ok()
  }).flatten();

  sqlx::query(&query)
    .bind(address)
    .bind(wallet_id)
    .bind(account_id)
    .bind(chain_name)
    .bind(tx_hash)
    .bind(block_number as i64)
    .bind(amount)
    .bind(amount_decimal_bigdecimal)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to save deposit event: {}", e)))?;

  Ok(())
}

// Note: increment_customer_balance function removed
// Balance management is handled by blockbit-back-custody, not xScanner
// xScanner only logs deposit events for audit/reconciliation purposes

// Note: load_customer_addresses_to_rocksdb removed
// RocksDB cache is populated via SQS messages from Backend
// No need to load from PostgreSQL (customer_addresses table doesn't exist in xScanner DB)

// Check if a deposit already exists in the database
pub async fn deposit_exists(
  pool: &PgPool,
  tx_hash: &str,
  chain_name: &str,
) -> Result<bool, AppError> {
  let query = format!(
    "SELECT EXISTS(SELECT 1 FROM {} WHERE tx_hash = $1 AND chain_name = $2)",
    DEPOSIT_EVENTS_TABLE
  );

  let row: (bool,) = sqlx::query_as(&query)
    .bind(tx_hash)
    .bind(chain_name)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to check deposit existence: {}", e)))?;

  Ok(row.0)
}

// Check if a deposit is already confirmed
pub async fn is_deposit_confirmed(
  pool: &PgPool,
  tx_hash: &str,
) -> Result<bool, AppError> {
  let query = format!(
    "SELECT confirmed FROM {} WHERE tx_hash = $1",
    DEPOSIT_EVENTS_TABLE
  );

  let row: Option<(bool,)> = sqlx::query_as(&query)
    .bind(tx_hash)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to check deposit confirmation status: {}", e)))?;

  Ok(row.map(|(confirmed,)| confirmed).unwrap_or(false))
}

// Update deposit confirmation status
pub async fn update_deposit_confirmed(
  pool: &PgPool,
  tx_hash: &str,
) -> Result<(), AppError> {
  let query = format!(
    "UPDATE {} SET confirmed = TRUE WHERE tx_hash = $1",
    DEPOSIT_EVENTS_TABLE
  );

  sqlx::query(&query)
    .bind(tx_hash)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to update deposit confirmation: {}", e)))?;

  Ok(())
}

// Get all pending (unconfirmed) deposits for confirmation checking
pub async fn get_pending_deposits(
  pool: &PgPool,
) -> Result<Vec<crate::tasks::PendingDeposit>, AppError> {
  let query = format!(
    "SELECT address, wallet_id, account_id, chain_name, tx_hash, block_number, amount, amount_decimal FROM {} WHERE confirmed = FALSE ORDER BY block_number ASC",
    DEPOSIT_EVENTS_TABLE
  );

  let rows = sqlx::query(&query)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to get pending deposits: {}", e)))?;

  let mut deposits = Vec::new();
  for row in rows {
    let address: String = row.get("address");
    let wallet_id: String = row.get("wallet_id");
    let account_id: Option<String> = row.get("account_id");
    let chain_name: String = row.get("chain_name");
    let tx_hash: String = row.get("tx_hash");
    let block_number: i64 = row.get("block_number");
    let amount: String = row.get("amount");
    let amount_decimal_bigdecimal: Option<bigdecimal::BigDecimal> = row.get("amount_decimal");

    // Convert bigdecimal::BigDecimal to rust_decimal::Decimal
    let amount_decimal = amount_decimal_bigdecimal.and_then(|bd| {
      rust_decimal::Decimal::from_str(&bd.to_string()).ok()
    });

    deposits.push(crate::tasks::PendingDeposit {
      address,
      wallet_id,
      account_id,
      chain_name,
      tx_hash,
      block_number: block_number as u64,
      amount,
      amount_decimal,
    });
  }

  Ok(deposits)
}

