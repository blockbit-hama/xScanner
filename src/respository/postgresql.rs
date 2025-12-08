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
  
  // ?? ?? ??? (?? -> ?? ID)
  sqlx::query(&format!(
    r#"
        CREATE TABLE IF NOT EXISTS {} (
            id SERIAL PRIMARY KEY,
            address VARCHAR(255) NOT NULL,
            customer_id VARCHAR(255) NOT NULL,
            chain_name VARCHAR(50) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(address, chain_name)
        )
        "#,
    CUSTOMER_ADDRESSES_TABLE
  ))
    .execute(pool)
    .await?;
  
  // ??? ??
  let _ = sqlx::query(&format!("CREATE INDEX IF NOT EXISTS idx_ca_address ON {} (LOWER(address))", CUSTOMER_ADDRESSES_TABLE))
    .execute(pool)
    .await;
  let _ = sqlx::query(&format!("CREATE INDEX IF NOT EXISTS idx_ca_customer_id ON {} (customer_id)", CUSTOMER_ADDRESSES_TABLE))
    .execute(pool)
    .await;
  
  // ?? ??? ???
  sqlx::query(&format!(
    r#"
        CREATE TABLE IF NOT EXISTS {} (
            id SERIAL PRIMARY KEY,
            customer_id VARCHAR(255) NOT NULL,
            address VARCHAR(255) NOT NULL,
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
  
  // ??? ??
  let _ = sqlx::query(&format!("CREATE INDEX IF NOT EXISTS idx_de_customer_id ON {} (customer_id)", DEPOSIT_EVENTS_TABLE))
    .execute(pool)
    .await;
  let _ = sqlx::query(&format!("CREATE INDEX IF NOT EXISTS idx_de_address ON {} (address)", DEPOSIT_EVENTS_TABLE))
    .execute(pool)
    .await;
  let _ = sqlx::query(&format!("CREATE INDEX IF NOT EXISTS idx_de_block_number ON {} (block_number)", DEPOSIT_EVENTS_TABLE))
    .execute(pool)
    .await;
  
  // ?? ??? ???
  sqlx::query(&format!(
    r#"
        CREATE TABLE IF NOT EXISTS {} (
            id SERIAL PRIMARY KEY,
            customer_id VARCHAR(255) NOT NULL,
            chain_name VARCHAR(50) NOT NULL,
            balance NUMERIC(36, 18) DEFAULT 0,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(customer_id, chain_name)
        )
        "#,
    CUSTOMER_BALANCES_TABLE
  ))
    .execute(pool)
    .await?;
  
  // ??? ??
  let _ = sqlx::query(&format!("CREATE INDEX IF NOT EXISTS idx_cb_customer_id ON {} (customer_id)", CUSTOMER_BALANCES_TABLE))
    .execute(pool)
    .await;
  
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

// ?? ??? ?? ID ??
pub async fn get_customer_id_by_address(
  pool: &PgPool,
  address: &str,
  chain_name: &str,
) -> Result<Option<String>, AppError> {
  let row: Option<(String,)> = sqlx::query_as(&format!(
    "SELECT customer_id FROM {} WHERE LOWER(address) = LOWER($1) AND chain_name = $2",
    CUSTOMER_ADDRESSES_TABLE
  ))
    .bind(address)
    .bind(chain_name)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to query customer_id: {}", e)))?;
  
  Ok(row.map(|(customer_id,)| customer_id))
}

// ?? ??? ??
pub async fn save_deposit_event(
  pool: &PgPool,
  customer_id: &str,
  address: &str,
  chain_name: &str,
  tx_hash: &str,
  block_number: u64,
  amount: &str,
  amount_decimal: Option<rust_decimal::Decimal>,
) -> Result<(), AppError> {
  let query = format!(
    r#"
        INSERT INTO {} (customer_id, address, chain_name, tx_hash, block_number, amount, amount_decimal)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
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
    .bind(customer_id)
    .bind(address)
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

// ?? ??? ?? (??)
pub async fn increment_customer_balance(
  pool: &PgPool,
  customer_id: &str,
  chain_name: &str,
  amount: rust_decimal::Decimal,
) -> Result<(), AppError> {
  use std::str::FromStr;
  let amount_bigdecimal = bigdecimal::BigDecimal::from_str(&amount.to_string())
    .map_err(|e| AppError::Database(format!("Failed to convert Decimal to BigDecimal: {}", e)))?;
  
  let query = format!(
    r#"
        INSERT INTO {} (customer_id, chain_name, balance, updated_at)
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
        ON CONFLICT (customer_id, chain_name)
        DO UPDATE SET 
          balance = {}.balance + $3,
          updated_at = CURRENT_TIMESTAMP
        "#,
    CUSTOMER_BALANCES_TABLE,
    CUSTOMER_BALANCES_TABLE
  );
  
  sqlx::query(&query)
    .bind(customer_id)
    .bind(chain_name)
    .bind(amount_bigdecimal)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to update customer balance: {}", e)))?;
  
  Ok(())
}

// PostgreSQL?? ?? ???? ???? RocksDB? ??
#[cfg(feature = "rocksdb-backend")]
pub async fn load_customer_addresses_to_rocksdb(
  pool: &PgPool,
  rocksdb: &rocksdb::DB,
  chain_name: &str,
) -> Result<usize, AppError> {
  use rocksdb::WriteBatch;
  
  // PostgreSQL?? ??? ?? ID ????
  let query = format!(
    "SELECT address, customer_id FROM {} WHERE chain_name = $1",
    CUSTOMER_ADDRESSES_TABLE
  );
  
  let rows = sqlx::query(&query)
    .bind(chain_name)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to query customer addresses: {}", e)))?;
  
  if rows.is_empty() {
    warn!("No customer addresses found for chain: {} (this is normal if no addresses are registered yet)", chain_name);
    return Ok(0);
  }
  
  // ??? RocksDB? ?? (???? ??: chain_name:address ??)
  let mut batch = WriteBatch::default();
  let mut count = 0;
  
  for row in rows {
    let address: String = row.try_get("address")
      .map_err(|e| AppError::Database(format!("Failed to get address: {}", e)))?;
    let customer_id: String = row.try_get("customer_id")
      .map_err(|e| AppError::Database(format!("Failed to get customer_id: {}", e)))?;
    
    let normalized_address = address.to_lowercase();
    let key = format!("{}:{}", chain_name.to_lowercase(), normalized_address);
    batch.put(key.as_bytes(), customer_id.as_bytes());
    count += 1;
  }
  
  rocksdb.write(batch)
    .map_err(|e| AppError::Database(format!("Failed to write to RocksDB: {}", e)))?;

  Ok(count)
}

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

