use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Instant,
};

use async_trait::async_trait;
use cached::proc_macro::cached;
use chrono::{Duration, NaiveDate, NaiveDateTime};
use log::info;
use rust_decimal::Decimal;
use serde_json::{json, value::Value};

use crate::{
    config,
    db::Database,
    import_account::ImportAccount,
    model::{
        n26_accounts::N26Accounts, n26_transaction::N26Transaction,
        real_transaction::IdentifiableTransaction, token_data::TokenData,
    },
};

const N26_ACCOUNT: &str = "Assets:Cash:N26";
const BASE_URL_GLOBAL: &str = "https://api.tech26.global";
const BASE_URL_DE: &str = "https://api.tech26.de";
const BASIC_AUTH_USERNAME: &str = "nativeweb";
const BASIC_AUTH_PASSWORD: &str = "";
const HEADER_KEY_DEVICE_TOKEN: &str = "device-token";
const HEADER_VALUE_DEVICE_TOKEN: &str = "34d100f8-ff28-487d-a836-3393d5e273d2";
const GRANT_TYPE: &str = "grant_type";
const GRANT_TYPE_PASSWORD: &str = "password";
const GRANT_TYPE_REFRESH_TOKEN: &str = "refresh_token";
const GRANT_TYPE_MFA_OOB: &str = "mfa_oob";
const USERNAME_KEY: &str = "username";
const PASSWORD_KEY: &str = "password";
const REFRESH_TOKEN_KEY: &str = "refresh_token";
const USER_AGENT_KEY: &str = "User-Agent";
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/59.0.3071.86 Safari/537.36";
const CONTENT_TYPE_KEY: &str = "Content-Type";
const CONTENT_TYPE_JSON: &str = "application/json";
const MFA_TOKEN: &str = "mfaToken";
const CHALLENGE_TYPE: &str = "challengeType";
const CHALLENGE_TYPE_OOB: &str = "oob";

/// Retrieves the current balance
#[cached(time = 600)]
async fn get_accounts_request(token: String) -> N26Accounts {
    let request_url: String = format!("{}/api/accounts", BASE_URL_DE);
    let response = reqwest::Client::new()
        .get(&request_url)
        .bearer_auth(token)
        .send()
        .await
        .unwrap();

    response.json().await.unwrap()
}

/// Get a list of transactions.
/// * `from_time` - earliest transaction time as a Timestamp > 0 - milliseconds since 1970 in CET
/// * `to_time`   - latest transaction time as a Timestamp > 0 - milliseconds since 1970 in CET
/// * `limit`     - Limit the number of transactions to return to the given amount
/// * `last_id`   - ??
/// Returns a list of transactions
#[cached(time = 600)]
async fn get_transactions_request(
    token: String,
    from_time: Option<NaiveDateTime>,
    to_time: Option<NaiveDateTime>,
    limit: Option<u32>,
    last_id: Option<String>,
) -> Vec<N26Transaction> {
    let mut params = vec![];
    if let Some(from) = from_time {
        params.push(("from", from.timestamp_millis().to_string()));
    }
    if let Some(to) = to_time {
        params.push(("to", to.timestamp_millis().to_string()));
    }
    if let Some(last_id) = last_id {
        params.push(("lastId", last_id));
    }
    if let Some(limit) = limit {
        params.push(("limit", limit.to_string()));
    }

    let request_url: String = format!("{}/api/smrt/transactions", BASE_URL_DE);
    let response = reqwest::Client::new()
        .get(&request_url)
        .bearer_auth(token)
        .query(&params)
        .send()
        .await
        .unwrap();

    let transactions = response.json::<Vec<N26Transaction>>().await.unwrap();
    // For some reason the api doesn't resect the "from" parameter
    if let Some(from) = from_time {
        return transactions
            .into_iter()
            .filter(|t| t.get_date() >= from.date())
            .collect();
    }
    transactions
}

pub struct N26 {
    http_client: reqwest::Client,
    db: Arc<Database>,
    waiting_for_mfa: AtomicBool,
}

impl N26 {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            waiting_for_mfa: AtomicBool::new(false),
            db,
        }
    }

    /// Returns false if a new authentication flow is needed
    pub async fn attempt_refresh_authentication(&self) -> bool {
        if let Some(auth) = self.get_authentication().as_ref() {
            if auth.is_valid() {
                // Don't need to refresh or reauthenticate
                return true;
            }
            let new_auth = self.refresh_authentication().await;
            if let Some(new_auth) = new_auth {
                self.set_authentication(new_auth);
                return true;
            }
        }
        return false;
    }

    fn get_authentication(&self) -> Option<TokenData> {
        self.db.get_auth()
    }

    fn set_authentication(&self, new_authentication: TokenData) {
        self.db.set_auth(Some(new_authentication))
    }

    fn clear_authentication(&self) {
        self.db.set_auth(None)
    }

    /* Authenication flow:
    1. POST normal URL encoded form to oauth2/token, with username, password, and grant_type=password. Response contains error=mfa_required, and a GUID mfaToken.
    2. POST JSON to /api/mfa/challenge, with challengeType=oob, and the mfaToken.
    3. POST to oauth2/token again, with mfaToken and grant_type=mfa_oob. This responds with an access token and refresh token if the login has been accepted on the paired device.
    4. Access token is used in Authorization: Bearer header to authenticate following requests.
    */

    async fn authenticate(&self) {
        if let Some(mut new_auth) = request_token(
            &self.http_client,
            &self.waiting_for_mfa,
            config::n26_username()
                .as_ref()
                .expect("N26 username not set"),
            config::n26_password()
                .as_ref()
                .expect("N26 password not set"),
        )
        .await
        {
            new_auth.update_expiration_time();
            if new_auth.is_valid() {
                info!("Successfully got access token: {}", new_auth.access_token);
                self.set_authentication(new_auth);
                return;
            }
        }
        panic!("Unable to request authentication token");
    }

    /// Refreshes an existing authentication using a (possibly expired) refresh token
    async fn refresh_authentication(&self) -> Option<TokenData> {
        let refresh_token = self.get_authentication().as_ref().expect("Can't refresh token since no existing token data was found. Please initiate a new authentication instead.").refresh_token.clone();
        info!(
            "Trying to refresh access token using refresh token {}",
            &refresh_token
        );
        if let Some(mut new_auth) = request_token_refresh(&self.http_client, &refresh_token).await {
            new_auth.update_expiration_time();
            if new_auth.is_valid() {
                return Some(new_auth);
            }
        } else {
            self.clear_authentication();
        }
        None
    }

    /// Returns the access token to use for api authentication.
    /// If a token has been requested before it will be reused if it is still valid.
    /// If the previous token has expired it will be refreshed.
    /// If no token has been requested a new one will be requested from the server.
    async fn get_token(&self) -> String {
        let success = self.attempt_refresh_authentication().await;
        if !success {
            // Stall until other auth flows are done
            while self.waiting_for_mfa.load(Ordering::SeqCst) {
                info!("Stalling N26 auth until a different MFA is accepted");
                thread::sleep(Duration::seconds(5).to_std().unwrap());
            }

            self.authenticate().await;
        }
        self.get_authentication()
            .expect("Failed to get token!")
            .access_token
    }
}

#[async_trait]
impl ImportAccount for N26 {
    type RealTransactionType = N26Transaction;

    async fn get_transactions(&self) -> Vec<Self::RealTransactionType> {
        let start = Instant::now();
        let token = self.get_token().await;
        let from = NaiveDate::from_ymd(2019, 1, 1).and_hms(0, 0, 0);

        let response =
            get_transactions_request(token, Some(from), None, Some(std::i32::MAX as u32), None)
                .await;
        info!("Fetch transactions from N26 took {:?}", start.elapsed());
        response
    }

    async fn get_balance(&self) -> Decimal {
        let start = Instant::now();
        let token = self.get_token().await;
        let response = get_accounts_request(token).await;
        info!("Fetch balance from N26 took {:?}", start.elapsed());
        response.available_balance
    }

    fn get_hledger_account(&self) -> &str {
        N26_ACCOUNT
    }
}

async fn initiate_authentication_flow(
    http_client: &reqwest::Client,
    username: &str,
    password: &str,
) -> String {
    info!("Requesting authentication flow for user {}", username);
    let values_token = [
        (GRANT_TYPE, GRANT_TYPE_PASSWORD),
        (USERNAME_KEY, username),
        (PASSWORD_KEY, password),
    ];

    let request_url: String = format!("{}/oauth/token", BASE_URL_GLOBAL);
    let response: reqwest::Response = http_client
        .post(&request_url)
        .basic_auth(BASIC_AUTH_USERNAME, Some(BASIC_AUTH_PASSWORD))
        .header(HEADER_KEY_DEVICE_TOKEN, HEADER_VALUE_DEVICE_TOKEN)
        .form(&values_token)
        .send()
        .await
        .unwrap();
    if response.status() != 403 {
        if response.status() == 429 {
            panic!(
                "Too many failed N26 logins: {:#?}",
                response.json::<Value>().await.unwrap()
            );
        }
        panic!(
            "Unexpected response for initial auth request: {:#?}",
            response
        );
    }

    let response_data = response.json::<Value>().await.unwrap();
    if let Some(error) = response_data.get("error") {
        if error == "mfa_required" {
            return response_data[MFA_TOKEN].as_str().unwrap().to_string();
        }
    }
    panic!("Unexpected response data");
}

/// Request an authentication token from the server.
async fn request_token(
    http_client: &reqwest::Client,
    waiting_for_2fa: &AtomicBool,
    username: &str,
    password: &str,
) -> Option<TokenData> {
    let mfa_token = initiate_authentication_flow(http_client, username, password).await;
    info!("Got MFA token {}", mfa_token);
    request_mfa_approval(http_client, &mfa_token).await;
    waiting_for_2fa.store(true, Ordering::SeqCst);
    let mut new_auth: Option<TokenData> = None;
    while new_auth.is_none() {
        thread::sleep(Duration::seconds(5).to_std().unwrap());
        new_auth = complete_authentication_flow(http_client, &mfa_token).await;
    }
    waiting_for_2fa.store(false, Ordering::SeqCst);
    new_auth
}

async fn request_token_refresh(
    http_client: &reqwest::Client,
    refresh_token: &str,
) -> Option<TokenData> {
    let values_token = [
        (GRANT_TYPE, GRANT_TYPE_REFRESH_TOKEN),
        (REFRESH_TOKEN_KEY, refresh_token),
    ];

    let request_url: String = format!("{}/oauth/token", BASE_URL_GLOBAL);
    let response: reqwest::Response = http_client
        .post(&request_url)
        .basic_auth(BASIC_AUTH_USERNAME, Some(BASIC_AUTH_PASSWORD))
        .header(HEADER_KEY_DEVICE_TOKEN, HEADER_VALUE_DEVICE_TOKEN)
        .form(&values_token)
        .send()
        .await
        .unwrap();
    match response.error_for_status() {
        Ok(response) => Some(
            response
                .json()
                .await
                .unwrap_or_else(|e| panic!("Failed to deserialize response: {}", e)),
        ),
        Err(err) => {
            info!("Refresh token request failed: {}", err);
            None
        }
    }
}

async fn request_mfa_approval(http_client: &reqwest::Client, mfa_token: &str) {
    info!("Requesting MFA approval using MFA token: {}", mfa_token);

    let mfa_data = json!({
        MFA_TOKEN: mfa_token,
        CHALLENGE_TYPE: CHALLENGE_TYPE_OOB,
    })
    .to_string();

    let request_url: String = format!("{}/api/mfa/challenge", BASE_URL_DE);
    let response: reqwest::Response = http_client
        .post(&request_url)
        .basic_auth(BASIC_AUTH_USERNAME, Some(BASIC_AUTH_PASSWORD))
        .header(HEADER_KEY_DEVICE_TOKEN, HEADER_VALUE_DEVICE_TOKEN)
        .header(USER_AGENT_KEY, USER_AGENT)
        .header(CONTENT_TYPE_KEY, CONTENT_TYPE_JSON)
        .body(mfa_data)
        .send()
        .await
        .unwrap();

    if response.status().is_success() {
        info!("Successfully requested MFA approval. Check your phone!")
    } else {
        panic!("Failed to request MFA approval!");
    }
}

async fn complete_authentication_flow(
    http_client: &reqwest::Client,
    mfa_token: &str,
) -> Option<TokenData> {
    info!(
        "Completing authentication flow for MFA token: {}",
        mfa_token
    );

    let mfa_data = [(MFA_TOKEN, mfa_token), (GRANT_TYPE, GRANT_TYPE_MFA_OOB)];

    let request_url: String = format!("{}/oauth/token", BASE_URL_DE);
    let response: reqwest::Response = http_client
        .post(&request_url)
        .basic_auth(BASIC_AUTH_USERNAME, Some(BASIC_AUTH_PASSWORD))
        .header(HEADER_KEY_DEVICE_TOKEN, HEADER_VALUE_DEVICE_TOKEN)
        .form(&mfa_data)
        .send()
        .await
        .unwrap();

    if response.status().is_success() {
        Some(response.json().await.unwrap())
    } else {
        None
    }
}
