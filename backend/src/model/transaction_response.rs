use serde::Serialize;

use super::{recorded_transaction::RecordedTransaction, rule::Rule};

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    // For now a N26 transaction
    pub real_transaction: serde_json::Value,
    // For now a hledger transaction. None if unmatched.
    pub recorded_transaction: Option<RecordedTransaction>,
    pub rule: Option<Rule>,
}
