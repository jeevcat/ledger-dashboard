use serde::Serialize;

use super::{recorded_transaction::RecordedTransaction, rule::Rule};

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    // For now a N26 transaction
    pub real_transaction: serde_json::Value,
    // For now a hledger transaction. None if unmatched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorded_transaction: Option<RecordedTransaction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<Rule>,
    pub errors: Vec<String>,
}
