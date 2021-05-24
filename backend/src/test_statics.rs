use lazy_static::lazy_static;
use regex::Regex;

use crate::model::{
    hledger_transaction::HledgerTransaction,
    n26_transaction::N26Transaction,
    rule::{Rule, RulePosting},
};

pub const ASSET_ACCOUNT: &str = "Assets:Cash:N26";
pub const EXPENSE_ACCOUNT: &str = "Expenses:Personal:Entertainment";

lazy_static! {
    pub static ref RULES: Vec<Rule> = vec![Rule {
        match_field_name: "partnerName".to_string(),
        match_field_regex: Regex::new("(?i)amazon").unwrap(),
        postings: vec![
            RulePosting {
                amount_field_name: Some("amount".to_string()),
                currency_field_name: Some("currencyCode".to_string()),
                account: ASSET_ACCOUNT.to_string(),
                negate: false,
            },
            RulePosting {
                amount_field_name: Some("amount".to_string()),
                currency_field_name: Some("currencyCode".to_string()),
                account: EXPENSE_ACCOUNT.to_string(),
                negate: true,
            }
        ],
        description_template: "Test {{{partnerName}}} with {{{referenceText}}}".to_string(),
        ..Rule::default()
    }];
    pub static ref REAL: Vec<N26Transaction> = serde_json::from_str(
        r#"[
            {
                "id": "1fc7d65c-de7c-415f-bf17-94de40c2e5d2",
                "amount": -219.56,
                "currencyCode": "EUR",
                "visibleTS": 1597308032422,
                "partnerName": "Amazon",
                "referenceText": "Buy item 1"
            },
            {
                "id": "b33d6f8f-733c-4bf8-bef5-206cb3c27171",
                "amount": -123.45,
                "currencyCode": "EUR",
                "visibleTS": 1597308032422,
                "partnerName": "Supermarket",
                "referenceText": "Buy item 2"
            },
            {
                "id": "02946eaf-8320-4d2d-b44c-54c473771e68",
                "amount": -3,
                "currencyCode": "USD",
                "visibleTS": 1597308032422,
                "partnerName": "Amazon",
                "referenceText": "Buy item 3"
            }
        ]"#,
    )
    .unwrap();
    pub static ref RECORDED: Vec<HledgerTransaction> = vec![HledgerTransaction::new_with_postings(
        &REAL[0],
        "My Description",
        &[
            RulePosting {
                amount_field_name: Some("amount".to_string()),
                currency_field_name: Some("currencyCode".to_string()),
                account: ASSET_ACCOUNT.to_string(),
                negate: false
            },
            RulePosting {
                amount_field_name: Some("amount".to_string()),
                currency_field_name: Some("currencyCode".to_string()),
                account: EXPENSE_ACCOUNT.to_string(),
                negate: true
            },
        ],
    )];
}
