use rust_decimal::Decimal;

/// 입금 정보 구조체
#[derive(Debug, Clone)]
pub struct DepositInfo {
    pub address: String,
    pub tx_hash: String,
    pub block_number: u64,
    pub amount: String,
    pub amount_decimal: Option<Decimal>,
}

impl DepositInfo {
    pub fn new(
        address: String,
        tx_hash: String,
        block_number: u64,
        amount: String,
        amount_decimal: Option<Decimal>,
    ) -> Self {
        Self {
            address,
            tx_hash,
            block_number,
            amount,
            amount_decimal,
        }
    }
}
