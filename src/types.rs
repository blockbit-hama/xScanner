use crate::coin::bitcoin::model::BitcoinBlock;
use crate::coin::ethereum::model::EthereumBlock;
use tokio::sync::mpsc::{Sender, Receiver};

// ====== BlockData (BTC or ETH) ======
#[derive(Debug)]
pub enum BlockData {
  Ethereum(EthereumBlock),
  Bitcoin(BitcoinBlock),
}

// ====== Channel aliases ======
pub type BlockSender = Sender<BlockData>;
pub type BlockReceiver = Receiver<BlockData>;

// ====== Unified Error Type ======
#[derive(Debug, thiserror::Error)]
pub enum AppError {
  #[error("API Client error: {0}")]
  Client(String),
  
  #[error("Channel send error: {0}")]
  SendError(String),
  
  #[error("Task join error: {0}")]
  JoinError(#[from] tokio::task::JoinError),
  
  #[error("Configuration error: {0}")]
  Config(String),
  
  #[error("Processing error in Analyzer: {0}")]
  Analyzer(String),
  
  #[error("Initialization error: {0}")]
  Initialization(String),
  
  #[error("Database error: {0}")]
  Database(String),
  
  #[error("Block error: {0}")]
  Block(String),
}

// ====== Error Conversions (From impls) ======

impl From<reqwest::Error> for AppError {
  fn from(err: reqwest::Error) -> Self {
    AppError::Client(format!("Reqwest error: {}", err))
  }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for AppError {
  fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Self {
    AppError::SendError(format!("Channel send failed: {}", err))
  }
}

impl From<sqlx::Error> for AppError {
  fn from(err: sqlx::Error) -> Self {
    AppError::Database(format!("SQLx error: {}", err))
  }
}

impl From<std::io::Error> for AppError {
  fn from(err: std::io::Error) -> Self {
    AppError::Initialization(format!("IO error: {}", err))
  }
}

impl From<serde_json::Error> for AppError {
  fn from(err: serde_json::Error) -> Self {
    AppError::Client(format!("JSON parse error: {}", err))
  }
}
