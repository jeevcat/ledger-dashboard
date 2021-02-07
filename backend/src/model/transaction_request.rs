use serde::{Deserialize, Serialize};

use super::n26transaction::N26Transaction;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    pub account: String,
    pub description_template: String,
    pub source_transaction: N26Transaction,
    pub should_write: Option<bool>,
}
