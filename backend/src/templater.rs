use handlebars::{Handlebars, RenderError, TemplateError, TemplateRenderError};
use log::error;
use serde::Serialize;

use crate::model::rule::Rule;

pub struct Templater<'a> {
    handlebars: Handlebars<'a>,
}

impl<'a> Templater<'a> {
    pub fn new() -> Self {
        Self {
            handlebars: Handlebars::new(),
        }
    }

    pub fn from_rules<'r, I: IntoIterator<Item = &'r Rule>>(rules: I) -> Self {
        let mut templater = Self::new();
        for rule in rules {
            templater
                .register_rule(rule)
                .unwrap_or_else(|e| error!("Error registering rule: {:#?}", e));
        }
        templater
    }

    pub fn register_rule(&mut self, rule: &Rule) -> Result<(), TemplateError> {
        if self.handlebars.has_template(&rule.rule_name) {
            return Ok(());
        }

        self.handlebars
            .register_template_string(&rule.rule_name, &rule.description_template)
    }

    pub fn render_description_from_rule<T>(
        &self,
        rule: &Rule,
        data: &T,
    ) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        self.handlebars.render(&rule.rule_name, data)
    }

    pub fn render_description<T>(
        &self,
        template_string: &str,
        data: &T,
    ) -> Result<String, TemplateRenderError>
    where
        T: Serialize,
    {
        self.handlebars.render_template(template_string, data)
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use regex::Regex;

    use super::*;
    use crate::model::{n26transaction::N26Transaction, rule::Rule};

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
    fn render_description_from_rule() {
        let mut templater = Templater::new();
        templater.register_rule(&RULE).unwrap();
        let desc = templater
            .render_description_from_rule(&RULE, &*TRANSACTION)
            .unwrap();
        assert_eq!(&desc, "Test description for Amazon deals");
    }

    #[test]
    fn render_description() {
        let templater = Templater::new();
        let desc = templater
            .render_description(
                "Transaction with {{{amount}}} {{{currencyCode}}}",
                &*TRANSACTION,
            )
            .unwrap();
        assert_eq!(&desc, "Transaction with 219.56 EUR");
    }
}
