use handlebars::{Handlebars, RenderError, TemplateError};
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
    ) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        self.handlebars.render_template(template_string, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_statics::{REAL, RULES};

    #[test]
    fn render_description_from_rule() {
        let mut templater = Templater::new();
        templater.register_rule(&RULES[0]).unwrap();
        let desc = templater
            .render_description_from_rule(&RULES[0], &REAL[0])
            .unwrap();
        assert_eq!(&desc, "Test Amazon with Buy item 1");
    }

    #[test]
    fn render_description() {
        let templater = Templater::new();
        let desc = templater
            .render_description("Transaction with {{{amount}}} {{{currencyCode}}}", &REAL[0])
            .unwrap();
        assert_eq!(&desc, "Transaction with -219.56 EUR");
    }
}
