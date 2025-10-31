use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WaykiTransaction {
  pub txid: String,
  #[serde(rename = "type")]
  pub tx_type: String,
  pub vin: Option<Vec<serde_json::Value>>,
  pub vout: Option<Vec<serde_json::Value>>,
}

#[derive(Deserialize, Debug)]
pub struct WaykiBlock {
  pub hash: String,
  pub height: u64,
  pub time: u64,
  pub tx: Vec<WaykiTransaction>,
}
