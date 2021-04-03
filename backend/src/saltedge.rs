use async_trait::async_trait;
use serde::Deserialize;

use crate::{
    config, import_account::ImportAccount, model::saltedge_transaction::SaltEdgeTransaction,
};

const ING_ACCOUNTS: &[&str; 1] = &["Assets:Cash:ING"];

#[derive(Deserialize)]
struct SaltEdgeResponse<T> {
    data: T,
}

pub struct SaltEdge {
    http_client: reqwest::Client,
}

impl SaltEdge {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ImportAccount for SaltEdge {
    type RealTransactionType = SaltEdgeTransaction;

    async fn get_transactions(&self) -> Vec<Self::RealTransactionType> {
        let request_url = "https://www.saltedge.com/api/v5/transactions";

        let app_id = config::saltedge_app_id().expect("Salt Edge app id not set");
        let secret = config::saltedge_secret().expect("Salt Edge secret not set");
        let connection_id =
            config::saltedge_connection_id().expect("Salt Edge connection id not set");
        let account_id = config::saltedge_account_id().expect("Salt Edge account id not set");

        let response = self
            .http_client
            .get(request_url)
            .header("App-id", app_id)
            .header("Secret", secret)
            .query(&[("connection_id", connection_id), ("account_id", account_id)])
            .send()
            .await
            .unwrap();

        let response = response
            .json::<SaltEdgeResponse<Vec<SaltEdgeTransaction>>>()
            .await
            .unwrap()
            .data;
        response
    }

    fn get_hledger_accounts(&self) -> &[&str] {
        ING_ACCOUNTS
    }
}
