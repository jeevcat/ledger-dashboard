use std::{borrow::Cow, fmt::Debug};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;

pub trait RealTransaction: Serialize + Debug {
    fn get_id(&self) -> Cow<str>;
    fn get_date(&self) -> NaiveDate;
    fn get_amount(&self) -> Decimal;
    fn get_currency(&self) -> &str;
    fn get_account(&self) -> &str;
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
