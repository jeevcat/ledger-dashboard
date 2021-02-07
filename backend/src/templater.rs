use handlebars::{Handlebars, RenderError, TemplateError, TemplateRenderError};
use serde::Serialize;

use crate::{db::Database, model::rule::Rule};

pub struct Templater<'a> {
    handlebars: Handlebars<'a>,
}

impl<'a> Templater<'a> {
    pub fn new(db: &Database) -> Result<Self, TemplateError> {
        let mut handlebars = Handlebars::new();
        for rule in db.get_all_rules() {
            handlebars.register_template_string(&rule.rule_name, &rule.description_template)?;
        }
        Ok(Self { handlebars })
    }

    pub fn register_rule(&mut self, rule: &Rule) -> Result<(), TemplateError> {
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
