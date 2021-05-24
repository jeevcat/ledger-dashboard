use std::time::Instant;

use async_trait::async_trait;
use cached::proc_macro::cached;
use log::info;
use serde::{de::DeserializeOwned, Deserialize};

use crate::{
    config,
    import_account::ImportAccount,
    model::{saltedge_account::SaltEdgeAccount, saltedge_transaction::SaltEdgeTransaction},
};

const ING_ACCOUNT: &str = "Assets:Cash:ING";

#[derive(Deserialize)]
struct SaltEdgeResponse<T> {
    data: T,
}

// TODO: Pagination https://docs.saltedge.com/general/#pagination
async fn request<T>(url: &str) -> T
where
    T: DeserializeOwned,
{
    let app_id = config::saltedge_app_id().expect("Salt Edge app id not set");
    let secret = config::saltedge_secret().expect("Salt Edge secret not set");
    let connection_id = config::saltedge_connection_id().expect("Salt Edge connection id not set");

    let response = reqwest::Client::new()
        .get(url)
        .header("App-id", app_id)
        .header("Secret", secret)
        .query(&[
            ("connection_id", connection_id),
            ("account_id", account_id()),
            ("per_page", 1000.to_string()),
        ])
        .send()
        .await
        .unwrap();

    response.json::<SaltEdgeResponse<T>>().await.unwrap().data
}

#[cached(time = 600)]
async fn fetch_transactions() -> Vec<SaltEdgeTransaction> {
    let url = "https://www.saltedge.com/api/v5/transactions";
    request(url).await
}

#[cached(time = 600)]
async fn fetch_accounts() -> Vec<SaltEdgeAccount> {
    let url = "https://www.saltedge.com/api/v5/accounts";
    request(url).await
}

pub struct SaltEdge;

#[async_trait]
impl ImportAccount for SaltEdge {
    type RealTransactionType = SaltEdgeTransaction;

    async fn get_transactions(&self) -> Vec<Self::RealTransactionType> {
        let start = Instant::now();
        let transactions = fetch_transactions().await;
        info!(
            "Fetched {} transactions from Salt Edge in {:?}",
            transactions.len(),
            start.elapsed()
        );
        transactions
    }

    async fn get_balance(&self) -> rust_decimal::Decimal {
        let accounts = fetch_accounts().await;
        accounts
            .iter()
            .find(|a| a.id == account_id())
            .unwrap()
            .balance
    }

    fn get_hledger_account(&self) -> &str {
        ING_ACCOUNT
    }

    fn get_id(&self) -> &str {
        "ing"
    }
}

fn account_id() -> String {
    config::saltedge_account_id().expect("Salt Edge account id not set")
}
