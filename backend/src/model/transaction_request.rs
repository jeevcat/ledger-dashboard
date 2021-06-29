use serde::{Deserialize, Serialize};

use super::{
    n26_transaction::N26Transaction, real_transaction::RealTransaction, rule::RulePosting,
    saltedge_transaction::SaltEdgeTransaction,
};
use crate::ib::IbTransaction;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum SourceTransaction {
    N26(N26Transaction),
    SaltEdge(SaltEdgeTransaction),
    Ib(IbTransaction),
}

impl RealTransaction for SourceTransaction {
    fn get_id(&self) -> std::borrow::Cow<str> {
        match self {
            SourceTransaction::N26(t) => t.get_id(),
            SourceTransaction::SaltEdge(t) => t.get_id(),
            SourceTransaction::Ib(t) => t.get_id(),
        }
    }

    fn get_date(&self) -> chrono::NaiveDate {
        match self {
            SourceTransaction::N26(t) => t.get_date(),
            SourceTransaction::SaltEdge(t) => t.get_date(),
            SourceTransaction::Ib(t) => t.get_date(),
        }
    }

    fn get_default_amount_field_name(&self) -> &str {
        match self {
            SourceTransaction::N26(t) => t.get_default_amount_field_name(),
            SourceTransaction::SaltEdge(t) => t.get_default_amount_field_name(),
            SourceTransaction::Ib(t) => t.get_default_amount_field_name(),
        }
    }

    fn get_default_currency_field_name(&self) -> &str {
        match self {
            SourceTransaction::N26(t) => t.get_default_currency_field_name(),
            SourceTransaction::SaltEdge(t) => t.get_default_currency_field_name(),
            SourceTransaction::Ib(t) => t.get_default_currency_field_name(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    pub description_template: String,
    pub source_transaction: SourceTransaction,
    pub postings: Vec<RulePosting>,
    pub should_write: Option<bool>,
}
