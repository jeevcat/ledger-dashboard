use std::sync::Arc;

use actix_web::{web, HttpResponse};

use crate::{
    hledger::Hledger,
    ib::Ib,
    model::{real_transaction::RealTransaction, transaction_response::TransactionResponse},
    transactions,
};

const IB_ACCOUNTS: &[&str; 2] = &["Assets:Investments:IB", "Assets:Cash:IB"];

// Get transactions whose ids match
pub async fn get_existing_transactions(hledger: web::Data<Arc<Hledger>>) -> HttpResponse {
    // Get real transactions
    let ib_report = Ib::read_report();

    // Get recorded transactions
    let hledger_transactions = hledger.get_transactions(IB_ACCOUNTS).await;

    let existing = transactions::get_existing_transactions(
        hledger_transactions.iter(),
        ib_report.get_transactions(),
    );

    HttpResponse::Ok().json(existing)
}

// Get remaining real transactions which no rules matched
pub async fn get_unmatched_transactions() -> HttpResponse {
    // Get real transactions
    let ib_report = Ib::read_report();

    let unmatched: Vec<TransactionResponse> = ib_report
        .get_transactions()
        .into_iter()
        .map(|real| TransactionResponse {
            real_transaction: real.to_json_value(),
            recorded_transaction: None,
            rule: None,
        })
        .collect();

    HttpResponse::Ok().json(unmatched)
}
