use serde::Deserialize;

/**
 * filename: model
 * author: HAMA
 * date: 2025. 10. 31.
 * description: AION blockchain data structures
 **/

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AionTransaction {
  pub nrg_price: Option<String>,
  pub nrg: Option<i64>,
  pub transaction_index: Option<i32>,
  pub nonce: Option<i64>,
  pub input: Option<String>,
  pub block_number: Option<i64>,
  pub from: Option<String>,
  pub to: Option<String>,
  pub value: Option<String>,
  pub hash: Option<String>,
  pub gas_price: Option<String>,
  pub timestamp: Option<u64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AionResult {
  pub difficulty: String,
  pub extra_data: String,
  pub hash: String,
  pub gas_limit: String,
  pub gas_used: String,
  pub logs_bloom: String,
  pub miner: String,
  pub mix_hash: String,
  pub nonce: String,
  pub number: String,
  pub parent_hash: String,
  pub receipts_root: String,
  pub sha3_uncles: String,
  pub size: String,
  pub state_root: String,
  pub timestamp: String,
  pub total_difficulty: String,
  pub transactions: Vec<AionTransaction>,
  pub transactions_root: String,
  pub uncles: Vec<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AionError {
  pub code: i32,
  pub message: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AionBlock {
  pub jsonrpc: String,
  pub id: usize,
  pub result: Option<AionResult>,
  pub error: Option<AionError>,
}
