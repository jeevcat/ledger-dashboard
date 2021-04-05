use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N26Accounts {
    pub available_balance: Decimal,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}
