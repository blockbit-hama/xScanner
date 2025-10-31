use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThetaBlock {
    pub chain_id: String,
    pub epoch: String,
    pub height: String,
    pub parent: String,
    pub transactions_hash: String,
    pub state_hash: String,
    pub timestamp: String,
    pub proposer: String,
    pub children: Vec<String>,
    pub status: i64,
    pub hash: String,
    pub transactions: Vec<ThetaTransaction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThetaTransaction {
    pub hash: String,
    pub nonce: Option<String>,
    pub block_hash: Option<String>,
    pub block_number: Option<String>,
    pub transaction_index: Option<String>,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub gas_price: Option<String>,
    pub gas: Option<String>,
    pub input: Option<String>,
    pub v: Option<String>,
    pub r: Option<String>,
    pub s: Option<String>,
}
