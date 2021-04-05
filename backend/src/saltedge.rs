use async_trait::async_trait;
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

pub struct SaltEdge {
    http_client: reqwest::Client,
}

impl SaltEdge {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    async fn request<T>(&self, url: &str) -> T
    where
        T: DeserializeOwned,
    {
        let app_id = config::saltedge_app_id().expect("Salt Edge app id not set");
        let secret = config::saltedge_secret().expect("Salt Edge secret not set");
        let connection_id =
            config::saltedge_connection_id().expect("Salt Edge connection id not set");

        let response = self
            .http_client
            .get(url)
            .header("App-id", app_id)
            .header("Secret", secret)
            .query(&[
                ("connection_id", connection_id),
                ("account_id", account_id()),
            ])
            .send()
            .await
            .unwrap();

        response.json::<SaltEdgeResponse<T>>().await.unwrap().data
    }
}

#[async_trait]
impl ImportAccount for SaltEdge {
    type RealTransactionType = SaltEdgeTransaction;

    async fn get_transactions(&self) -> Vec<Self::RealTransactionType> {
        let url = "https://www.saltedge.com/api/v5/transactions";
        self.request::<Vec<SaltEdgeTransaction>>(url).await
    }

    async fn get_balance(&self) -> rust_decimal::Decimal {
        let url = "https://www.saltedge.com/api/v5/accounts";
        let accounts = self.request::<Vec<SaltEdgeAccount>>(url).await;
        accounts
            .iter()
            .find(|a| a.id == account_id())
            .unwrap()
            .balance
    }

    fn get_hledger_account(&self) -> &str {
        ING_ACCOUNT
    }
}

fn account_id() -> String {
    config::saltedge_account_id().expect("Salt Edge account id not set")
}
