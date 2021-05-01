use std::{borrow::Cow, fmt::Debug};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;

use super::hledger_transaction::Posting;

pub trait PostingTransaction {
    fn get_amount(&self, account: &str) -> Decimal;
    fn get_postings(&self, account: &str) -> Vec<Posting>;
}

pub trait IdentifiableTransaction: Serialize {
    fn get_id(&self) -> Cow<str>;
    fn get_date(&self) -> NaiveDate;
    fn to_json_value(&self) -> serde_json::Value {
        // Ensure we always have an id field
        const ID: &str = "id";
        let mut val = serde_json::to_value(self).unwrap();
        let obj = val.as_object_mut().unwrap();
        if !obj.contains_key(ID) {
            let id = serde_json::to_value(self.get_id()).unwrap();
            obj.insert(ID.to_string(), id);
            return serde_json::to_value(obj).unwrap();
        }
        val
    }
}

pub trait DefaultPostingTransaction {
    fn get_currency(&self) -> &str;
    fn get_amount(&self) -> Decimal;
    fn get_account(&self) -> &str;
}

impl<T: DefaultPostingTransaction + Serialize + Debug> PostingTransaction for T {
    fn get_postings(&self, account: &str) -> Vec<Posting> {
        vec![
            Posting::new(self.get_account(), self.get_currency(), self.get_amount()),
            Posting::new(account, self.get_currency(), -self.get_amount()),
        ]
    }

    fn get_amount(&self, account: &str) -> Decimal {
        assert_eq!(self.get_account(), account);
        self.get_amount()
    }
}

pub trait RealTransaction: PostingTransaction + IdentifiableTransaction {}
impl<T: PostingTransaction + IdentifiableTransaction> RealTransaction for T {}
