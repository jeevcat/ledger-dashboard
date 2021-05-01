use std::env;

pub fn n26_username() -> Option<String> {
    env::var("N26_USERNAME").ok()
}

pub fn n26_password() -> Option<String> {
    env::var("N26_PASSWORD").ok()
}

pub fn saltedge_app_id() -> Option<String> {
    env::var("SALTEDGE_APP_ID").ok()
}

pub fn saltedge_secret() -> Option<String> {
    env::var("SALTEDGE_SECRET").ok()
}

pub fn saltedge_connection_id() -> Option<String> {
    env::var("SALTEDGE_CONNECTION_ID").ok()
}

pub fn saltedge_account_id() -> Option<String> {
    env::var("SALTEDGE_ACCOUNT_ID").ok()
}

pub fn alpha_vantage_key() -> Option<String> {
    env::var("ALPHA_VANTAGE_KEY").ok()
}

pub fn ib_flex_token() -> Option<String> {
    env::var("IB_FLEX_TOKEN").ok()
}

pub fn ib_flex_balance_query_id() -> Option<String> {
    env::var("IB_FLEX_BALANCE_QUERY_ID").ok()
}

pub fn ib_flex_transactions_query_id() -> Option<String> {
    env::var("IB_FLEX_TRANSACTIONS_QUERY_ID").ok()
}

pub fn journal_path() -> Option<String> {
    env::var("JOURNAL_PATH").ok()
}

pub fn database_path() -> Option<String> {
    env::var("DATABASE_PATH").ok()
}
