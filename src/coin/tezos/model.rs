use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TezosOperation {
  pub kind: String,
  pub source: Option<String>,
  pub destination: Option<String>,
  pub amount: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TezosOperationGroup {
  pub contents: Vec<TezosOperation>,
  pub hash: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TezosBlock {
  pub hash: String,
  pub header: serde_json::Value,
  pub operations: Vec<Vec<TezosOperationGroup>>,
}
