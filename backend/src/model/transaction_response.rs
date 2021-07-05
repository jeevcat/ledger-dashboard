use serde::Serialize;

use super::{hledger_transaction::HledgerTransaction, rule::Rule};

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub real_transaction: serde_json::Value,
    // For now a hledger transaction. None if unmatched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hledger_transaction: Option<HledgerTransaction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<Rule>,
}

#[derive(Debug, Serialize)]
pub struct ExistingTransactionResponse {
    // For now a N26 transaction
    pub real_transaction: serde_json::Value,
    // For now a hledger transaction. None if unmatched.
    pub hledger_transaction: HledgerTransaction,
    pub errors: Vec<String>,
}
