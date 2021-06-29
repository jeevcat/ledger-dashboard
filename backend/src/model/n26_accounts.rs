use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N26Accounts {
    pub available_balance: f64,
}
