use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaltEdgeAccount {
    pub id: String,
    pub balance: f64,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}
