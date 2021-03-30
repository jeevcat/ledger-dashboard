use serde::{Deserialize, Serialize};

use super::{
    n26transaction::N26Transaction, real_transaction::RealTransaction,
    saltedge_transaction::SaltEdgeTransaction,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum SourceTransaction {
    N26(N26Transaction),
    SaltEdge(SaltEdgeTransaction),
}

impl RealTransaction for SourceTransaction {
    fn get_id(&self) -> std::borrow::Cow<str> {
        match self {
            SourceTransaction::N26(t) => t.get_id(),
            SourceTransaction::SaltEdge(t) => t.get_id(),
        }
    }

    fn get_date(&self) -> chrono::NaiveDate {
        match self {
            SourceTransaction::N26(t) => t.get_date(),
            SourceTransaction::SaltEdge(t) => t.get_date(),
        }
    }

    fn get_amount(&self) -> rust_decimal::Decimal {
        match self {
            SourceTransaction::N26(t) => t.get_amount(),
            SourceTransaction::SaltEdge(t) => t.get_amount(),
        }
    }

    fn get_currency(&self) -> &str {
        match self {
            SourceTransaction::N26(t) => t.get_currency(),
            SourceTransaction::SaltEdge(t) => t.get_currency(),
        }
    }

    fn get_account(&self) -> &str {
        match self {
            SourceTransaction::N26(t) => t.get_account(),
            SourceTransaction::SaltEdge(t) => t.get_account(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    pub account: String,
    pub description_template: String,
    pub source_transaction: SourceTransaction,
    pub should_write: Option<bool>,
}
