use std::{collections::HashSet, sync::Arc};

use actix_web::{web, HttpResponse};
use log::info;

use crate::{
    db::Database,
    hledger::Hledger,
    model::{
        real_transaction::RealTransaction, recorded_transaction::RecordedTransaction,
        transaction_response::TransactionResponse,
    },
    saltedge::SaltEdge,
    transactions::{self},
};
pub type TransactionCollection = Vec<RecordedTransaction>;

const ING_ACCOUNTS: &[&str; 1] = &["Assets:Cash:ING"];

// Get transactions whose ids match
pub async fn get_existing_transactions(
    saltedge: web::Data<Arc<SaltEdge>>,
    hledger: web::Data<Arc<Hledger>>,
) -> HttpResponse {
    // Get real transactions
    let real_transactions = saltedge.get_transactions().await;

    // Get recorded transactions
    let recorded_transactions: TransactionCollection = hledger.get_transactions(ING_ACCOUNTS).await;

    let existing =
        transactions::get_existing_transactions(&recorded_transactions, real_transactions);

    HttpResponse::Ok().json(existing)
}

// Get transactions which were able to be generated from rules
pub async fn get_generated_transactions(
    saltedge: web::Data<Arc<SaltEdge>>,
    hledger: web::Data<Arc<Hledger>>,
    db: web::Data<Arc<Database>>,
) -> HttpResponse {
    // Get real transactions
    let real_transactions = saltedge.get_transactions().await;

    // Get recorded transactions
    let recorded_transactions: TransactionCollection = hledger.get_transactions(ING_ACCOUNTS).await;

    // Get rules
    let rules = db.get_all_rules();

    let generated = transactions::get_generated_transactions(
        &recorded_transactions,
        &real_transactions,
        &rules,
    );

    HttpResponse::Ok().json(generated)
}

pub async fn write_generated_transactions(
    saltedge: web::Data<Arc<SaltEdge>>,
    hledger: web::Data<Arc<Hledger>>,
    db: web::Data<Arc<Database>>,
) -> HttpResponse {
    // Get real transactions
    let real_transactions = saltedge.get_transactions().await;

    // Get recorded transactions
    let hledger_transactions: TransactionCollection = hledger.get_transactions(ING_ACCOUNTS).await;

    // Get rules
    let rules = db.get_all_rules();

    let mut generated: Vec<RecordedTransaction> =
        transactions::get_generated_transactions(&hledger_transactions, &real_transactions, &rules)
            .into_iter()
            .filter_map(|t| t.recorded_transaction)
            .collect();

    generated.sort_by(|a, b| a.tdate.cmp(&b.tdate));
    info!("Writing {} transactions to hledger", generated.len());
    let result = hledger.write_transactions(&generated).await;

    if result {
        HttpResponse::Created().finish()
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

// Get remaining real transactions which no rules matched
pub async fn get_unmatched_transactions(
    saltedge: web::Data<Arc<SaltEdge>>,
    hledger: web::Data<Arc<Hledger>>,
    db: web::Data<Arc<Database>>,
) -> HttpResponse {
    // Get real transactions
    let real_transactions = saltedge.get_transactions().await;

    // Get recorded transactions
    let hledger_transactions: TransactionCollection = hledger.get_transactions(ING_ACCOUNTS).await;
    // Optimization. Collect unique ids so we can quickly check if a transaction HASN'T been recorded.
    let recorded_ids: HashSet<&str> = hledger_transactions.iter().flat_map(|t| t.ids()).collect();

    // Get rules
    let rules = db.get_all_rules();

    let unmatched: Vec<TransactionResponse> = real_transactions
        .into_iter()
        // Only real transactions which haven't already been recorded
        .filter(|real| {
            !recorded_ids.contains(&*real.get_id()) && !rules.iter().any(|rule| rule.matches(real))
        })
        .map(|real| TransactionResponse {
            real_transaction: real.to_json_value(),
            recorded_transaction: None,
            rule: None,
        })
        .collect();

    HttpResponse::Ok().json(unmatched)
}