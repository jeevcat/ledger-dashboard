use std::collections::HashSet;

use actix_web::HttpResponse;
use log::info;

use super::transactions::TransactionCollection;
use crate::{
    db::Database,
    hledger::Hledger,
    import_account::ImportAccount,
    model::{
        real_transaction::RealTransaction, recorded_transaction::RecordedTransaction, rule::Rule,
        transaction_response::TransactionResponse,
    },
    transactions,
};

pub async fn get_existing_transactions(
    import_account: &impl ImportAccount,
    hledger: &Hledger,
) -> HttpResponse {
    // Get real transactions
    let real_transactions = import_account.get_transactions().await;

    // Get recorded transactions
    let recorded_transactions: TransactionCollection = hledger
        .get_transactions(import_account.get_hledger_accounts())
        .await;

    let existing =
        transactions::get_existing_transactions(&recorded_transactions, real_transactions);

    HttpResponse::Ok().json(existing)
}

// Get transactions which were able to be generated from rules
pub async fn get_generated_transactions(
    import_account: &impl ImportAccount,
    hledger: &Hledger,
    db: &Database,
) -> HttpResponse {
    // Get real transactions
    let real_transactions = import_account.get_transactions().await;

    // Get recorded transactions
    let recorded_transactions: TransactionCollection = hledger
        .get_transactions(import_account.get_hledger_accounts())
        .await;

    // Get rules
    let rules = get_rules(db, import_account);

    let generated = transactions::get_generated_transactions(
        &recorded_transactions,
        &real_transactions,
        &rules,
    );

    HttpResponse::Ok().json(generated)
}

pub async fn write_generated_transactions(
    import_account: &impl ImportAccount,
    hledger: &Hledger,
    db: &Database,
) -> HttpResponse {
    // Get real transactions
    let real_transactions = import_account.get_transactions().await;

    // Get recorded transactions
    let hledger_transactions: TransactionCollection = hledger
        .get_transactions(import_account.get_hledger_accounts())
        .await;

    // Get rules
    let rules = get_rules(db, import_account);

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
    import_account: &impl ImportAccount,
    hledger: &Hledger,
    db: &Database,
) -> HttpResponse {
    // Get real transactions
    let real_transactions = import_account.get_transactions().await;

    // Get recorded transactions
    let hledger_transactions: TransactionCollection = hledger
        .get_transactions(import_account.get_hledger_accounts())
        .await;
    // Optimization. Collect unique ids so we can quickly check if a transaction HASN'T been recorded.
    let recorded_ids: HashSet<&str> = hledger_transactions.iter().flat_map(|t| t.ids()).collect();

    // Get rules
    let rules = get_rules(db, import_account);

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

fn get_rules(db: &Database, import_account: &impl ImportAccount) -> Vec<Rule> {
    db.get_all_rules(Some(import_account.get_hledger_accounts()[0]))
}
