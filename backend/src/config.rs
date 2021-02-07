use std::env;

pub fn n26_username() -> Option<String> {
    env::var("N26_USERNAME").ok()
}

pub fn n26_password() -> Option<String> {
    env::var("N26_PASSWORD").ok()
}

pub fn alpha_vantage_key() -> Option<String> {
    env::var("ALPHA_VANTAGE_KEY").ok()
}

pub fn journal_path() -> Option<String> {
    env::var("JOURNAL_PATH").ok()
}

pub fn database_path() -> Option<String> {
    env::var("DATABASE_PATH").ok()
}
