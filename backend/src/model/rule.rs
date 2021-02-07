use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{real_transaction::RealTransaction, recorded_transaction::RecordedTransaction};
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
    pub account: String,
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
            account: Default::default(),
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
    ) -> Option<RecordedTransaction> {
        if !self.matches(real_transaction) {
            return None;
        }

        let description = templater
            .render_description_from_rule(self, real_transaction)
            .unwrap_or_default();

        Some(RecordedTransaction::new_with_postings(
            real_transaction,
            &description,
            &self.account,
        ))
    }
}
