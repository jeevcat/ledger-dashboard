use std::borrow::Cow;

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::real_transaction::{DefaultPostingTransaction, IdentifiableTransaction};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N26Transaction {
    id: String,
    amount: Decimal,
    currency_code: String,
    #[serde(rename = "visibleTS")]
    visible_ts: i64,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl IdentifiableTransaction for N26Transaction {
    fn get_id(&self) -> Cow<str> {
        Cow::Borrowed(&self.id)
    }

    fn get_date(&self) -> NaiveDate {
        // TODO: Write custom deserializer
        let s: i64 = self.visible_ts / 1000i64;
        NaiveDateTime::from_timestamp(s, 0).date()
    }
}

impl DefaultPostingTransaction for N26Transaction {
    fn get_amount(&self) -> Decimal {
        self.amount
    }

    fn get_currency(&self) -> &str {
        self.currency_code.as_str()
    }

    fn get_account(&self) -> &str {
        "Assets:Cash:N26"
    }
}
