use std::env;

pub fn api_key() -> Option<String> {
    env::var("API_KEY").ok()
}

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

pub fn ib_flex_token() -> String {
    env::var("IB_FLEX_TOKEN").expect("Need to set IB_FLEX_TOKEN")
}

pub fn ib_flex_balance_query_id() -> String {
    env::var("IB_FLEX_BALANCE_QUERY_ID").expect("Need to set IB_FLEX_BALANCE_QUERY_ID")
}

pub fn ib_flex_transactions_query_id() -> String {
    env::var("IB_FLEX_TRANSACTIONS_QUERY_ID").expect("Need to set IB_FLEX_TRANSACTIONS_QUERY_ID")
}

pub fn mongodb_url() -> String {
    env::var("MONGODB_URL").expect("MONGODB_URL must be set!")
}

/// Relative to journal repo root if journal_repo_url is supplied
/// Otherwise absolute
pub fn journal_path() -> Option<String> {
    env::var("JOURNAL_PATH").ok()
}

pub fn journal_repo_url() -> String {
    env::var("JOURNAL_REPO_URL").expect("JOURNAL_REPO_URL must be set!")
}

pub fn journal_repo_credentials() -> Option<(String, String)> {
    Some((
        env::var("JOURNAL_REPO_USERNAME").ok()?,
        env::var("JOURNAL_REPO_PASSWORD").ok()?,
    ))
}
