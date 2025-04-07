// src/db.rs
use crate::types::AppError;
use sqlx::{postgres::PgPoolOptions, PgPool};


pub async fn connect_db(db_url: &str) -> Result<PgPool, sqlx::Error> {
  PgPoolOptions::new()
    .max_connections(5) // Adjust pool size as needed
    .connect(db_url)
    .await
}

const STATE_TABLE_NAME: &str = "blockchain_state";

// Ensure the state table exists
pub async fn setup_db_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
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

