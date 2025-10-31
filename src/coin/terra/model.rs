use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TerraBlockId {
  pub hash: String,
}

#[derive(Deserialize, Debug)]
pub struct TerraBlockMeta {
  pub block_id: TerraBlockId,
}

#[derive(Deserialize, Debug)]
pub struct TerraHeader {
  pub chain_id: String,
  pub height: String,
}

#[derive(Deserialize, Debug)]
pub struct TerraData {
  pub txs: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct TerraBlockInner {
  pub header: TerraHeader,
  pub data: TerraData,
}

#[derive(Deserialize, Debug)]
pub struct TerraBlock {
  pub block_meta: TerraBlockMeta,
  pub block: TerraBlockInner,
}
