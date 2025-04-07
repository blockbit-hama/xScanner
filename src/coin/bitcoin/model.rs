/**
* filename : model
* author : HAMA
* date: 2025. 4. 7.
* description:
**/

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BitcoinBlock {
  pub hash: String,
  pub ver: u32,
  pub prev_block: String,
  pub mrkl_root: String,
  pub time: u64,
  pub bits: u32,
  pub next_block: Option<Vec<String>>,
  pub fee: u64,
  pub nonce: u64,
  pub n_tx: u32,
  pub size: u32,
  pub block_index: u64,
  pub main_chain: bool,
  pub height: u64,
  pub weight: u32,
  pub tx: Vec<Transaction>,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
  pub hash: String,
  pub ver: u32,
  pub vin_sz: u32,
  pub vout_sz: u32,
  pub size: u32,
  pub weight: u32,
  pub fee: Option<u64>,
  pub relayed_by: String,
  pub lock_time: u32,
  pub tx_index: u64,
  pub double_spend: bool,
  pub time: u64,
  pub block_index: u64,
  pub block_height: u64,
  pub inputs: Vec<Input>,
  pub out: Vec<Output>,
}

#[derive(Debug, Deserialize)]
pub struct Input {
  pub sequence: u64,
  pub witness: String,
  pub script: String,
  pub index: u32,
  pub prev_out: Option<Output>,
}

#[derive(Debug, Deserialize)]
pub struct Output {
  pub r#type: u32,
  pub spent: bool,
  pub value: u64,
  pub spending_outpoints: Vec<SpendingOutpoint>,
  pub n: u32,
  pub tx_index: u64,
  pub script: String,
  pub addr: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpendingOutpoint {
  pub tx_index: u64,
  pub n: u32,
}
