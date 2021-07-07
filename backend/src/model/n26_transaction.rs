use std::borrow::Cow;

use bson::Document;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

use super::real_transaction::RealTransaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N26Transaction {
    id: String,
    pub amount: f64,
    pub currency_code: String,
    #[serde(rename = "visibleTS")]
    visible_ts: i64,
    #[serde(flatten)]
    extra: Document,
}

impl RealTransaction for N26Transaction {
    fn get_id(&self) -> Cow<str> {
        Cow::Borrowed(&self.id)
    }

    fn get_date(&self) -> NaiveDate {
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
