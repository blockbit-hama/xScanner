use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GxchainTransaction {
  pub operations: Vec<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct GxchainBlock {
  pub previous: String,
  pub timestamp: String,
  #[serde(rename = "withness")]
  pub witness: Option<String>,
  #[serde(rename = "withness_signature")]
  pub witness_signature: Option<String>,
  pub transactions: Vec<GxchainTransaction>,
  pub transaction_ids: Vec<String>,
  #[serde(rename = "block_id")]
  pub hash: String,
}
