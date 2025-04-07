use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
  pub blockchain: BlockchainSettings,
  pub repository: RepositorySettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BlockchainSettings {
  pub ethereum: ChainConfig,
  pub bitcoin: ChainConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChainConfig {
  pub api: String,
  pub symbol: String,
  pub start_block: u64,
  pub interval_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RepositorySettings {
  pub postgresql_url: String,
  pub leveldb_path: String,
  pub customer_address_file: String,
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
    
    builder.build()?.try_deserialize()
  }
}
