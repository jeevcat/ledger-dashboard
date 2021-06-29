use std::{borrow::Cow, fmt::Debug};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{de::DeserializeOwned, Serialize};

use super::{
    hledger_transaction::{Posting, Price},
    rule::RulePosting,
};

pub trait RealTransaction: Serialize + DeserializeOwned + Send + Sync + Debug {
    fn get_id(&self) -> Cow<str>;
    fn get_date(&self) -> NaiveDate;
    fn get_default_amount_field_name(&self) -> &str;
    fn get_default_currency_field_name(&self) -> &str;

    fn get_postings(&self, hledger_account: &str, postings: &[RulePosting]) -> Vec<Posting> {
        let mut result: Vec<Posting> = vec![];
        if postings.len() < 2 {
            // Add default posting rule
            if let Some(p) = self.create_posting(&RulePosting {
                amount_field_name: None,
                currency_field_name: None,
                price: None,
                comment: None,
                account: hledger_account.to_string(),
                negate: false,
            }) {
                result.push(p);
            }
        }
        result.extend(
            postings
                .iter()
                .filter_map(|posting| self.create_posting(posting)),
        );
        result
    }

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

    fn to_doc(&self) -> Result<bson::Document, bson::ser::Error> {
        let mut doc = bson::to_document(self).unwrap();
        if !doc.contains_key("_id") {
            doc.insert("_id", self.get_id().as_ref());
        }
        Ok(doc)
    }

    fn get_amount(&self, rule_posting: &RulePosting) -> Option<Decimal> {
        self.get_field(
            rule_posting
                .amount_field_name
                .as_deref()
                .unwrap_or_else(|| self.get_default_amount_field_name()),
        )
    }

    fn get_currency(&self, rule_posting: &RulePosting) -> Option<String> {
        self.get_field(
            rule_posting
                .currency_field_name
                .as_deref()
                .unwrap_or_else(|| self.get_default_currency_field_name()),
        )
    }

    // TODO: Can I make the next two methods "private"?
    fn get_field<T: DeserializeOwned>(&self, field_name: &str) -> Option<T> {
        let value = self.to_json_value();
        serde_json::from_value(value[field_name].clone()).ok()
    }

    fn create_posting(&self, rule_posting: &RulePosting) -> Option<Posting> {
        let amount = self.get_amount(rule_posting)?;
        let amount = if rule_posting.negate { -amount } else { amount };
        let commodity = self.get_currency(rule_posting)?;
        let price = self.get_price(rule_posting);
        Some(Posting::new(
            &rule_posting.account,
            &commodity,
            amount,
            price,
            rule_posting.comment.as_deref(),
        ))
    }

    fn get_price(&self, rule_posting: &RulePosting) -> Option<Price> {
        let price = rule_posting.price.as_ref()?;
        let commodity: String = self.get_field(&price.currency_field_name)?;
        let quantity = self.get_field(&price.amount_field_name)?;
        Some(Price::new(&commodity, quantity))
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::{prelude::FromPrimitive, Decimal};

    use super::RealTransaction;
    use crate::test_statics::{ASSET_ACCOUNT, REAL, RULES};

    #[test]
    fn check_get_amount() {
        assert_eq!(
            REAL[0].get_amount(&RULES[0].postings[0]),
            Decimal::from_f64(-219.56)
        );
    }

    #[test]
    fn check_get_currency() {
        assert_eq!(
            REAL[0]
                .get_currency(&RULES[0].postings[0])
                .unwrap()
                .as_str(),
            "EUR"
        );
    }

    #[test]
    fn check_get_postings() {
        let postings = REAL[0].get_postings(ASSET_ACCOUNT, &RULES[0].postings);
        // TODO: do asserts
        println!("{:#?}", postings);
    }
}
