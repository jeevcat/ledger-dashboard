use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::model::real_transaction::RealTransaction;

#[async_trait]
pub trait ImportAccount {
    type RealTransactionType: RealTransaction;

    async fn get_transactions(&self) -> Vec<Self::RealTransactionType>;
    async fn get_balance(&self) -> Decimal;

    fn get_id(&self) -> &str;

    // hledger account which should have their transactions considered for this ImportAccount
    fn get_hledger_account(&self) -> &str;
}
