use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TronBlock {
    pub block_id: String,
    pub block_header: BlockHeader,
    pub transactions: Vec<TronTransaction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    pub raw_data: BlockHeaderRawData,
    pub witness_signature: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeaderRawData {
    pub number: u64,
    pub tx_trie_root: String,
    pub witness_address: String,
    pub parent_hash: String,
    pub version: i32,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TronTransaction {
    pub ret: Vec<TransactionRet>,
    pub signature: Vec<String>,
    pub tx_id: String,
    pub raw_data: TronTransactionRawData,
}

#[derive(Debug, Deserialize)]
pub struct TransactionRet {
    pub contract_ret: String,
    pub fee: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TronTransactionRawData {
    pub contract: Vec<TronContract>,
    pub ref_block_bytes: String,
    pub ref_block_hash: String,
    pub expiration: i64,
    pub fee_limit: Option<i64>,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TronContract {
    #[serde(rename = "type")]
    pub contract_type: String,
    pub parameter: TronContractParameter,
}

#[derive(Debug, Deserialize)]
pub struct TronContractParameter {
    pub value: TronContractValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TronContractValue {
    pub amount: Option<i64>,
    pub owner_address: Option<String>,
    pub to_address: Option<String>,
    pub asset_name: Option<String>,
    pub data: Option<String>,
}
