use async_trait::async_trait;

use crate::{db::Database, model::real_transaction::RealTransaction};

#[async_trait]
pub trait ImportAccount {
    type RealTransactionType: RealTransaction;

    async fn get_transactions_cached(
        &self,
        db: &Database,
        bypass_cache: bool,
    ) -> Vec<Self::RealTransactionType> {
        if bypass_cache {
            let t = self.get_transactions().await;
            db.cache_transactions(self.get_id(), &t).await.unwrap();
            t
        } else {
            db.get_transactions(self.get_id()).await.unwrap()
        }
    }
    async fn get_balance_cached(&self, db: &Database, bypass_cache: bool) -> f64 {
        if bypass_cache {
            let b = self.get_balance().await;
            db.cache_balance(self.get_id(), b).await.unwrap();
            b
        } else {
            db.get_balance(self.get_id()).await.unwrap()
        }
    }
    async fn get_transactions(&self) -> Vec<Self::RealTransactionType>;
    async fn get_balance(&self) -> f64;

    fn get_id(&self) -> &str;

    // hledger account which should have their transactions considered for this ImportAccount
    fn get_hledger_account(&self) -> &str;
}
