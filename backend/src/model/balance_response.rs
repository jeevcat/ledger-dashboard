use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub recorded: Decimal,
    pub real: Decimal,
}
