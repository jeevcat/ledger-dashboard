use std::borrow::Cow;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{de::DeserializeOwned, Serialize};

use super::{hledger_transaction::Posting, rule::RulePosting};

pub trait RealTransaction: Serialize {
    fn get_id(&self) -> Cow<str>;
    fn get_date(&self) -> NaiveDate;
    fn get_default_amount_field_name(&self) -> &str;
    fn get_default_currency_field_name(&self) -> &str;

    fn get_postings(&self, postings: &[RulePosting]) -> Vec<Posting> {
        postings
            .iter()
            .filter_map(|posting| self.create_posting(posting))
            .collect()
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
        println!("{:#?} {:#?}", amount, commodity);
        Some(Posting::new(&rule_posting.account, &commodity, amount))
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use rust_decimal::{prelude::FromPrimitive, Decimal};

    use super::RealTransaction;
    use crate::model::{n26_transaction::N26Transaction, rule::RulePosting};

    lazy_static! {
        static ref TRANSACTION: N26Transaction = serde_json::from_str(
            r#"{
                "id": "1fc7d65c-de7c-415f-bf17-94de40c2e5d2",
                "amount": -219.56,
                "currencyCode": "EUR",
                "visibleTS": 1597308032422,
                "partnerName": "Amazon deals"
            }"#,
        )
        .unwrap();
        static ref RULE_POSTINGS: Vec<RulePosting> = vec![
            RulePosting {
                amount_field_name: Some("amount".to_string()),
                currency_field_name: Some("currencyCode".to_string()),
                account: "Assets:Cash:N26".to_string(),
                negate: false,
            },
            RulePosting {
                amount_field_name: Some("amount".to_string()),
                currency_field_name: Some("currencyCode".to_string()),
                account: "Expenses:Personal:Entertainment".to_string(),
                negate: true,
            }
        ];
    }

    #[test]
    fn check_get_amount() {
        assert_eq!(
            TRANSACTION.get_amount(&RULE_POSTINGS[0]),
            Decimal::from_f64(-219.56)
        );
    }

    #[test]
    fn check_get_currency() {
        assert_eq!(
            TRANSACTION
                .get_currency(&RULE_POSTINGS[0])
                .unwrap()
                .as_str(),
            "EUR"
        );
    }

    #[test]
    fn check_get_postings() {
        let postings = TRANSACTION.get_postings(&RULE_POSTINGS);
        println!("{:#?}", postings);
    }
}
