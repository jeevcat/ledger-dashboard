use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{hledger_transaction::HledgerTransaction, real_transaction::RealTransaction};
use crate::templater::Templater;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: u32,
    pub priority: i32,
    pub rule_name: String,
    pub match_field_name: String,
    #[serde(with = "serde_regex")]
    pub match_field_regex: Regex,
    pub import_account: String,
    pub target_account: String,
    pub description_template: String,
}

impl Default for Rule {
    fn default() -> Self {
        Rule {
            id: Default::default(),
            priority: Default::default(),
            rule_name: Default::default(),
            match_field_name: Default::default(),
            match_field_regex: Regex::new("$^").unwrap(),
            import_account: Default::default(),
            target_account: Default::default(),
            description_template: Default::default(),
        }
    }
}

impl Rule {
    pub fn matches(&self, real_transaction: &impl RealTransaction) -> bool {
        let value = real_transaction.to_json_value();
        if let Some(field) = value.get(&self.match_field_name) {
            return self.match_field_regex.is_match(&field.to_string());
        }
        false
    }

    // Map fields in n26transaction to new transaction
    pub fn apply(
        &self,
        templater: &Templater,
        real_transaction: &impl RealTransaction,
    ) -> Option<HledgerTransaction> {
        if !self.matches(real_transaction) {
            return None;
        }
        let description = templater
            .render_description_from_rule(self, real_transaction)
            .unwrap_or_default();

        Some(HledgerTransaction::new_with_postings(
            real_transaction,
            &description,
            &self.target_account,
        ))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use lazy_static::lazy_static;
    use regex::Regex;

    use super::*;
    use crate::model::{n26_transaction::N26Transaction, rule::Rule};

    lazy_static! {
        static ref TRANSACTION: N26Transaction = serde_json::from_str(
            r#"{
                "id": "1fc7d65c-de7c-415f-bf17-94de40c2e5d2",
                "amount": 219.56,
                "currencyCode": "EUR",
                "visibleTS": 1597308032422,
                "partnerName": "Amazon deals"
            }"#,
        )
        .unwrap();
        static ref RULE: Rule = Rule {
            match_field_name: "partnerName".to_string(),
            match_field_regex: Regex::new("(?i)amazon").unwrap(),
            target_account: "Expenses:Personal:Entertainment".to_string(),
            description_template: "Test description for {{{partnerName}}}".to_string(),
            ..Rule::default()
        };
    }

    #[test]
    fn apply_rule() {
        let mut templater = Templater::new();
        templater.register_rule(&RULE).unwrap();
        let t = RULE.apply(&templater, &*TRANSACTION).unwrap();
        assert_eq!(t.tdescription, "Test description for Amazon deals");
        let date = t.get_date(None);
        assert_eq!(date.year(), 2020);
        assert_eq!(date.month(), 8);
        assert_eq!(date.day(), 13);
        assert_eq!(t.ttags[0][0], "uuid");
        assert_eq!(t.ttags[0][1], "1fc7d65c-de7c-415f-bf17-94de40c2e5d2");
        assert_eq!(t.tpostings[0].paccount, "Assets:Cash:N26");
        assert_eq!(t.tpostings[1].paccount, "Expenses:Personal:Entertainment");
    }

    #[test]
    fn apply_rule_no_match() {
        let rule = Rule {
            match_field_regex: Regex::new(".*supermarket").unwrap(),
            ..Rule::default()
        };
        let mut templater = Templater::new();
        templater.register_rule(&rule).unwrap();
        let t = rule.apply(&templater, &*TRANSACTION);
        assert!(t.is_none());
    }
}
