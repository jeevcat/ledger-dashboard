use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SaltEdgeAccount {
    pub id: String,
    pub balance: Decimal,
    pub currency_code: String,
}
