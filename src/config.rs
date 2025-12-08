use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
  #[serde(default)]
  pub blockchain: BlockchainSettings,
  pub repository: RepositorySettings,
  #[serde(default)]
  pub notification: Option<NotificationSettings>,
  #[serde(default)]
  pub customer_sync: Option<CustomerSyncSettings>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct BlockchainSettings {
  // ?? ???? (?? ??? ??)
  #[serde(default)]
  pub ethereum: Option<ChainConfig>,
  #[serde(default)]
  pub bitcoin: Option<ChainConfig>,
  
  // ?? ???? ??
  #[serde(default)]
  #[serde(flatten)]
  pub chains: HashMap<String, ChainConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChainConfig {
  pub api: String,
  pub symbol: String,
  pub start_block: u64,
  pub interval_secs: u64,
  #[serde(default = "default_required_confirmations")]
  pub required_confirmations: u64,
  #[serde(default)]
  pub rpc_method: Option<String>, // "eth_getBlockByNumber", "getblock" ?
  #[serde(default)]
  pub rpc_params_format: Option<String>, // "hex", "decimal", "string"
}

#[derive(Debug, Deserialize, Clone)]
pub struct NotificationSettings {
  pub sqs_queue_url: String,
  pub aws_region: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CustomerSyncSettings {
  pub sqs_queue_url: String,
  #[serde(default = "default_aws_region")]
  pub aws_region: String,
  #[serde(default = "default_batch_size")]
  pub batch_size: usize,
  #[serde(default = "default_flush_interval_secs")]
  pub flush_interval_secs: u64,
  #[serde(default)]
  pub cache_file_path: Option<String>,
}

fn default_aws_region() -> String {
  "ap-northeast-2".to_string()
}

fn default_batch_size() -> usize {
  100
}

fn default_flush_interval_secs() -> u64 {
  5
}

fn default_required_confirmations() -> u64 {
  12 // Default to Ethereum's 12 confirmations
}

#[derive(Debug, Deserialize, Clone)]
pub struct RepositorySettings {
  #[serde(default = "default_memory_db")]
  pub memory_db: bool,
  pub postgresql_url: String,
  pub leveldb_path: String,
  pub customer_address_file: String,
}

fn default_memory_db() -> bool {
  false
}

impl Settings {
  pub fn new() -> Result<Self, config::ConfigError> {
    let default_config_path = "./config.toml";
    let env_prefix = "APP"; // Environment variable prefix (e.g., APP_BLOCKCHAIN__ETHEREUM__API=...)
    
    let builder = config::Config::builder()
      .add_source(config::File::with_name(default_config_path).required(true))
      // E.g., `APP_BLOCKCHAIN__ETHEREUM__API=http://...` would override config file value
      .add_source(config::Environment::with_prefix(env_prefix).separator("__"))
      ;
    
    let mut settings: Settings = builder.build()?.try_deserialize()?;
    
    // ?? ???: ?? ethereum, bitcoin? chains? ??
    if let Some(eth) = settings.blockchain.ethereum.clone() {
      settings.blockchain.chains.insert("ethereum".to_string(), eth);
    }
    if let Some(btc) = settings.blockchain.bitcoin.clone() {
      settings.blockchain.chains.insert("bitcoin".to_string(), btc);
    }
    
    Ok(settings)
  }
  
  pub fn get_chain_configs(&self) -> Vec<(String, ChainConfig)> {
    self.blockchain.chains.iter()
      .map(|(name, config)| (name.clone(), config.clone()))
      .collect()
  }
}
