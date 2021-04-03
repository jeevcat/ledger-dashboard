use async_trait::async_trait;

use crate::model::real_transaction::RealTransaction;

#[async_trait]
pub trait ImportAccount {
    type RealTransactionType: RealTransaction;

    async fn get_transactions(&self) -> Vec<Self::RealTransactionType>;

    // hledger accounts which should have their transactions considered for this ImportAccount
    fn get_hledger_accounts(&self) -> &[&str];
}
