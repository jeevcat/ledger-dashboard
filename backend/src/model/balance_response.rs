use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub hledger: Decimal,
    pub real: Decimal,
}
