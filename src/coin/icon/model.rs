use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconBlock {
    pub version: String,
    pub prev_block_hash: String,
    pub merkle_tree_root_hash: String,
    pub time_stamp: f64,
    pub confirmed_transaction_list: Vec<IconTransaction>,
    pub block_hash: String,
    pub height: u64,
    pub peer_id: String,
    pub signature: String,
    pub next_leader: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconTransaction {
    pub version: String,
    pub from: String,
    pub to: String,
    pub value: Option<String>,
    pub step_limit: String,
    pub timestamp: String,
    pub nid: String,
    pub nonce: Option<String>,
    pub signature: String,
    pub tx_hash: String,
    pub data_type: Option<String>,
    pub data: Option<String>,
}
