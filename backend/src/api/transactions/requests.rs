use std::{
    collections::{BTreeMap, HashMap, HashSet},
    iter::FromIterator,
    sync::Arc,
};

use actix_web::{web, HttpResponse};
use log::info;
use serde_json::json;

use crate::{
    db::Database,
    hledger::Hledger,
    import_account::ImportAccount,
    model::{
        real_transaction::RealTransaction, recorded_transaction::RecordedTransaction, rule::Rule,
        transaction_request::TransactionRequest, transaction_response::TransactionResponse,
    },
    templater::Templater,
    transactions,
};

type TransactionCollection = Vec<RecordedTransaction>;

/// Get transactions whose ids match
pub async fn get_existing_transactions<T>(
    import_account: web::Data<Arc<T>>,
    hledger: web::Data<Arc<Hledger>>,
) -> HttpResponse
where
    T: ImportAccount,
{
    // Get real transactions
    let real_transactions = import_account.get_transactions().await;

    // Get recorded transactions
    let recorded_transactions: TransactionCollection = hledger
        .get_transactions(&[import_account.get_hledger_account()])
        .await;

    let existing =
        transactions::get_existing_transactions(&recorded_transactions, real_transactions);

    HttpResponse::Ok().json(existing)
}

// Get transactions which were able to be generated from rules
pub async fn get_generated_transactions<T>(
    import_account: web::Data<Arc<T>>,
    hledger: web::Data<Arc<Hledger>>,
    db: web::Data<Arc<Database>>,
) -> HttpResponse
where
    T: ImportAccount,
{
    // Get real transactions
    let real_transactions = import_account.get_transactions().await;

    // Get recorded transactions
    let recorded_transactions: TransactionCollection = hledger
        .get_transactions(&[import_account.get_hledger_account()])
        .await;

    // Get rules
    let rules = get_rules(&db, &***import_account);

    let generated = transactions::get_generated_transactions(
        &recorded_transactions,
        &real_transactions,
        &rules,
    );

    HttpResponse::Ok().json(generated)
}

pub async fn write_generated_transactions<T>(
    import_account: web::Data<Arc<T>>,
    hledger: web::Data<Arc<Hledger>>,
    db: web::Data<Arc<Database>>,
) -> HttpResponse
where
    T: ImportAccount,
{
    // Get real transactions
    let real_transactions = import_account.get_transactions().await;

    // Get recorded transactions
    let hledger_transactions: TransactionCollection = hledger
        .get_transactions(&[import_account.get_hledger_account()])
        .await;

    // Get rules
    let rules = get_rules(&db, &***import_account);

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
pub async fn get_unmatched_transactions<T>(
    import_account: web::Data<Arc<T>>,
    hledger: web::Data<Arc<Hledger>>,
    db: web::Data<Arc<Database>>,
) -> HttpResponse
where
    T: ImportAccount,
{
    // Get real transactions
    let real_transactions = import_account.get_transactions().await;

    // Get recorded transactions
    let hledger_transactions: TransactionCollection = hledger
        .get_transactions(&[import_account.get_hledger_account()])
        .await;
    // Optimization. Collect unique ids so we can quickly check if a transaction HASN'T been recorded.
    let recorded_ids: HashSet<&str> = hledger_transactions.iter().flat_map(|t| t.ids()).collect();

    // Get rules
    let rules = get_rules(&db, &***import_account);

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
            errors: vec![]
        })
        .collect();

    HttpResponse::Ok().json(unmatched)
}

pub async fn generate_single_transaction(
    request: web::Json<TransactionRequest>,
    hledger: web::Data<Arc<Hledger>>,
) -> HttpResponse {
    let description = Templater::new()
        .render_description(&request.description_template, &request.source_transaction);
    match description {
        Ok(description) => {
            let transaction = RecordedTransaction::new_with_postings(
                &request.source_transaction,
                &description,
                &request.account,
            );
            if request.should_write.unwrap_or(false) {
                hledger.write_single_transaction(&transaction).await;
            }
            HttpResponse::Ok().json(transaction)
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

pub async fn get_transaction_stats<T>(n26: web::Data<Arc<T>>) -> HttpResponse
where
    T: ImportAccount,
{
    // Get real transactions
    let real_transactions = n26.get_transactions().await;
    let mut fields_map = HashMap::<String, BTreeMap<String, u32>>::new();
    for t in real_transactions.iter() {
        for (key, value) in t.to_json_value().as_object().unwrap().iter() {
            let counter = fields_map
                .entry(key.clone())
                .or_default()
                .entry(value.to_string())
                .or_default();
            *counter += 1;
        }
    }
    let mut next_fields_map = HashMap::<String, Vec<(serde_json::Value, u32)>>::new();
    for (key, value) in fields_map.into_iter() {
        let mut sorted = Vec::from_iter(value);
        sorted.sort_by(|&(_, a), &(_, b)| b.cmp(&a));
        let jsoned: Vec<(serde_json::Value, u32)> = sorted
            .into_iter()
            .map(|(key, value)| {
                (
                    serde_json::from_str::<serde_json::Value>(&key).unwrap(),
                    value,
                )
            })
            .collect();
        next_fields_map.insert(key, jsoned);
    }
    HttpResponse::Ok().json(next_fields_map)
}

pub async fn check<T>(
    import_account: web::Data<Arc<T>>,
    hledger: web::Data<Arc<Hledger>>,
) -> HttpResponse
where
    T: ImportAccount,
{
    // Get recorded transactions
    let recorded_transactions: TransactionCollection = hledger
        .get_transactions(&[import_account.get_hledger_account()])
        .await;

    let mut recorded_ids: HashSet<&str> = HashSet::new();
    let mut dupe_ids: HashSet<&str> = HashSet::new();
    for t in recorded_transactions.iter() {
        for id in t.ids() {
            let was_first = recorded_ids.insert(id);
            if !was_first {
                dupe_ids.insert(id);
            }
        }
    }

    HttpResponse::Ok().json(json!({ "dupe_ids": dupe_ids }))
}

fn get_rules(db: &Database, import_account: &impl ImportAccount) -> Vec<Rule> {
    db.get_all_rules(Some(import_account.get_hledger_account()))
}
