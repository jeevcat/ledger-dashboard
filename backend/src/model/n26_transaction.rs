use std::borrow::Cow;

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::real_transaction::RealTransaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N26Transaction {
    id: String,
    pub amount: Decimal,
    pub currency_code: String,
    #[serde(rename = "visibleTS")]
    visible_ts: i64,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl RealTransaction for N26Transaction {
    fn get_id(&self) -> Cow<str> {
        Cow::Borrowed(&self.id)
    }

    fn get_date(&self) -> NaiveDate {
        // TODO: Write custom deserializer
        let s: i64 = self.visible_ts / 1000i64;
        NaiveDateTime::from_timestamp(s, 0).date()
    }

    fn get_default_amount_field_name(&self) -> &str {
        "amount"
    }

    fn get_default_currency_field_name(&self) -> &str {
        "currencyCode"
    }
}
