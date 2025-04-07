use serde::Deserialize;

/**
* filename : model
* author : HAMA
* date: 2025. 4. 7.
* description: 
**/


#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Transaction{
  pub id: usize,
  pub jsonrpc: String,
  pub result: Option<TransactionResult>,
  pub error: Option<EthereumError>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResult {
  pub block_hash: Option<String>,
  pub block_number: Option<String>,
  pub chain_id: Option<String>,
  pub from: Option<String>,
  pub gas: Option<String>,
  pub gas_price: Option<String>,
  pub hash: Option<String>,
  pub input: Option<String>,
  pub nonce: Option<String>,
  pub r: Option<String>,
  pub s: Option<String>,
  pub to: Option<String>, // `to` 필드가 null일 수 있으므로 Option<String>
  pub transaction_index: Option<String>,
  pub v: Option<String>,
  pub value: Option<String>, // `value` 필드도 null 가능성
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EthereumResult {
  pub difficulty: String,
  pub extra_data: String,
  pub gas_limit: String,
  pub gas_used: String,
  pub hash: String,
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
  pub transactions: Vec<TransactionResult>,
  pub transactions_root: String,
  pub uncles: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EthereumError {
  pub code: i32,
  pub message: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EthereumBlock {
  pub jsonrpc: String,
  pub id: usize,
  pub result: Option<EthereumResult>,
  pub error: Option<EthereumError>,
}

