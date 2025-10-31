use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuarkTransaction {
  pub from: Option<String>,
  pub to: Option<String>,
  pub value: Option<String>,
  pub hash: Option<String>,
  pub gas: Option<String>,
  pub gas_price: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuarkResult {
  pub number: String,
  pub hash: String,
  pub transactions: Vec<QuarkTransaction>,
  pub timestamp: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuarkBlock {
  pub jsonrpc: String,
  pub id: usize,
  pub result: Option<QuarkResult>,
}
