use std::{borrow::Cow, collections::HashMap};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::real_transaction::RealTransaction;

const DATE_FMT: &str = "%Y-%m-%d";

#[derive(Debug, Serialize, Deserialize)]
pub struct SaltEdgeTransaction {
    id: String,
    made_on: String,
    amount: Decimal,
    currency_code: String,
    duplicated: bool,
    mode: String,
    status: String,
    description: String,
    category: String,
    account_id: String,
    created_at: String,
    updated_at: String,
    extra: HashMap<String, Value>,
}

impl RealTransaction for SaltEdgeTransaction {
    fn get_id(&self) -> Cow<str> {
        Cow::Borrowed(&self.id)
    }

    fn get_date(&self) -> NaiveDate {
        // TODO: Write custom deserializer
        NaiveDate::parse_from_str(&self.made_on, DATE_FMT).unwrap()
    }

    fn get_amount(&self) -> Decimal {
        self.amount
    }

    fn get_currency(&self) -> &str {
        self.currency_code.as_str()
    }

    fn get_account(&self) -> &str {
        "Assets:Cash:ING"
    }
}

#[cfg(test)]
mod tests {

    use rust_decimal::{prelude::FromPrimitive, Decimal};

    use super::SaltEdgeTransaction;

    #[test]
    fn deserialize() {
        let sample = r#"{
            "id": "444444444444444444",
            "duplicated": false,
            "mode": "normal",
            "status": "posted",
            "made_on": "2020-05-03",
            "amount": -200.0,
            "currency_code": "USD",
            "description": "Money spent on company advertising",
            "category": "advertising",
            "extra": {
              "merchant_id": "b3e8ec2349df872072c051e0c...",
              "original_amount": -3974.6,
              "original_currency_code": "CZK",
              "posting_date": "2020-05-07",
              "time": "23:56:12"
            },
            "account_id": "333333333333333333",
            "created_at": "2021-03-03T10:56:11Z",
            "updated_at": "2021-03-04T10:56:11Z"
          }"#;
        let deserialized: SaltEdgeTransaction = serde_json::from_str(sample).unwrap();
        println!("{:#?}", deserialized);
        assert_eq!(deserialized.id, "444444444444444444");
        assert_eq!(deserialized.made_on, "2020-05-03");
        assert_eq!(deserialized.amount, Decimal::from_f32(-200.).unwrap());
        assert_eq!(deserialized.currency_code, "USD");
    }
}
