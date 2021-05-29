use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{hledger_transaction::HledgerTransaction, real_transaction::RealTransaction};
use crate::templater::Templater;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct RulePosting {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_field_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_field_name: Option<String>,
    pub account: String,
    // Should the amount be negated?
    pub negate: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: u32,
    pub priority: i32,
    pub importer_id: String,
    pub rule_name: String,
    pub match_field_name: String,
    #[serde(with = "serde_regex")]
    pub match_field_regex: Regex,
    pub description_template: String,
    pub postings: Vec<RulePosting>,
}

impl Default for Rule {
    fn default() -> Self {
        Rule {
            id: Default::default(),
            priority: Default::default(),
            importer_id: Default::default(),
            rule_name: Default::default(),
            match_field_name: Default::default(),
            match_field_regex: Regex::new("$^").unwrap(),
            description_template: Default::default(),
            postings: Default::default(),
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

    // Map fields in real transaction to new hledger transaction
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
            .ok()?;

        Some(
            HledgerTransaction::new(
                &description,
                real_transaction.get_date(),
                &real_transaction.get_id(),
            )
            .postings(&mut real_transaction.get_postings(&self.postings)),
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use regex::Regex;

    use super::*;
    use crate::{
        model::rule::Rule,
        test_statics::{REAL, RULES},
    };

    #[test]
    fn apply_rule() {
        let mut templater = Templater::new();
        templater.register_rule(&RULES[0]).unwrap();
        let t = RULES[0].apply(&templater, &REAL[0]).unwrap();
        assert_eq!(t.tdescription, "Test Amazon with Buy item 1");
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
        let t = rule.apply(&templater, &REAL[0]);
        assert!(t.is_none());
    }
}
