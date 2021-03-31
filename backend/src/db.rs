use std::collections::BTreeMap;

use log::info;
use rustbreak::FileDatabase;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    file_utils::get_database_file,
    model::{rule::Rule, token_data::TokenData},
};

const RULES_DATA_PATH: &str = "backend_db.yml";
const AUTH_DATA_PATH: &str = "auth_db.yml";

type RulesType = BTreeMap<u32, Rule>;
type AuthenticationType = Option<TokenData>;

pub struct Database {
    rules: FileDatabase<RulesType, rustbreak::deser::Yaml>,
    authentication: FileDatabase<AuthenticationType, rustbreak::deser::Yaml>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            rules: load_or_create_db(RULES_DATA_PATH),
            authentication: load_or_create_db(AUTH_DATA_PATH),
        }
    }

    // RULES

    pub fn create_or_update_rule(&self, rule: Rule) {
        let mut rule = rule;
        if rule.id == u32::default() {
            rule.id = self
                .rules
                .read(|db| {
                    let mut new_id: u32 = 1;
                    while db.contains_key(&new_id) {
                        new_id += 1;
                    }
                    new_id
                })
                .unwrap();
        }
        self.rules.write(|db| db.insert(rule.id, rule)).unwrap();
        self.rules.save().unwrap();
    }

    pub fn get_all_rules(&self, import_account: Option<&str>) -> Vec<Rule> {
        self.rules
            .read(|db| {
                let mut rules: Vec<Rule> = db
                    .iter()
                    .filter_map(|(_, rule)| {
                        if let Some(acc) = import_account {
                            if !rule.import_account.eq(acc) {
                                return None;
                            }
                        }
                        Some(rule)
                    })
                    // TODO: Can this cloned() be removed?
                    .cloned()
                    .collect();
                rules.sort_by(|a, b| a.priority.partial_cmp(&b.priority).unwrap());
                rules
            })
            .unwrap()
    }

    pub fn get_rule(&self, rule_id: u32) -> Option<Rule> {
        self.rules.read(|db| db.get(&rule_id).cloned()).unwrap()
    }

    pub fn delete_rule(&self, rule_id: u32) {
        self.rules.write(|db| db.remove(&rule_id)).unwrap();
        self.rules.save().unwrap();
    }

    // AUTH

    pub fn get_auth(&self) -> Option<TokenData> {
        self.authentication.read(|db| db.clone()).unwrap()
    }

    pub fn set_auth(&self, auth: Option<TokenData>) {
        self.authentication.write(|db| *db = auth).unwrap();
        self.authentication.save().unwrap();
    }
}

fn load_or_create_db<T>(filename: &str) -> FileDatabase<T, rustbreak::deser::Yaml>
where
    T: Serialize + DeserializeOwned + Clone + Send + Default,
{
    let db_path = get_database_file(filename).unwrap();
    info!("Using {} db at: {:#?}", filename, db_path);
    let rules: FileDatabase<T, rustbreak::deser::Yaml> =
        FileDatabase::load_from_path_or_default(&db_path)
            .or_else(|_| FileDatabase::create_at_path(db_path, T::default()))
            .unwrap();
    rules.save().unwrap();
    rules
}
