use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SaltEdgeAccount {
    pub id: String,
    pub balance: Decimal,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}
