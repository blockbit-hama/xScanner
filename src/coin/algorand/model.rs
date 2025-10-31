use serde::Deserialize;

/**
 * filename: model
 * author: HAMA
 * date: 2025. 10. 31.
 * description: ALGORAND blockchain data structures
 **/

#[derive(Deserialize, Debug)]
pub struct AlgorandPayment {
  pub to: String,
  pub amount: u64,
  #[serde(rename = "torewards")]
  pub to_rewards: Option<u64>,
  #[serde(rename = "closerewards")]
  pub close_rewards: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct AlgorandTransaction {
  #[serde(rename = "type")]
  pub tx_type: String,
  pub tx: String,
  pub from: String,
  pub fee: u64,
  pub round: u64,
  pub payment: Option<AlgorandPayment>,
  #[serde(rename = "fromrewards")]
  pub from_rewards: Option<u64>,
  #[serde(rename = "genesisID")]
  pub genesis_id: Option<String>,
  #[serde(rename = "genesishashb64")]
  pub genesis_hash: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AlgorandTransactionList {
  pub transactions: Vec<AlgorandTransaction>,
}

#[derive(Deserialize, Debug)]
pub struct AlgorandBlock {
  pub hash: String,
  #[serde(rename = "previousBlockHash")]
  pub prev_block_hash: String,
  pub seed: String,
  pub proposer: String,
  pub round: u64,
  pub txns: AlgorandTransactionList,
}
