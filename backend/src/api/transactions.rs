use std::{
    collections::{BTreeMap, HashMap},
    iter::FromIterator,
    sync::{Arc, Mutex},
};

use actix_web::{dev::HttpServiceFactory, error::InternalError, web, HttpResponse};

use super::transactions_ib;
use crate::{
    api::transactions_n26,
    hledger::Hledger,
    model::{
        real_transaction::RealTransaction, recorded_transaction::RecordedTransaction,
        transaction_request::TransactionRequest,
    },
    n26::N26,
    templater::Templater,
};

pub type TransactionCollection = Vec<RecordedTransaction>;

pub fn transactions_routes() -> impl HttpServiceFactory {
    web::scope("/transactions")
        .route("/stats", web::get().to(get_transaction_stats))
        .route("/new", web::post().to(generate_single_transaction))
        .route(
            "/existing/n26",
            web::get().to(transactions_n26::get_existing_transactions),
        )
        .route(
            "/generated/n26",
            web::get().to(transactions_n26::get_generated_transactions),
        )
        .route(
            "/unmatched/n26",
            web::get().to(transactions_n26::get_unmatched_transactions),
        )
        .route(
            "/write/n26",
            web::post().to(transactions_n26::write_generated_transactions),
        )
        .route(
            "/existing/ib",
            web::get().to(transactions_ib::get_existing_transactions),
        )
        .route(
            "/unmatched/ib",
            web::get().to(transactions_ib::get_unmatched_transactions),
        )
        .app_data(web::JsonConfig::default().error_handler(|err, _req| {
            let reponse = HttpResponse::BadRequest().json(err.to_string());
            println!("{}", err.to_string());
            InternalError::from_response(err, reponse).into()
        }))
}

async fn generate_single_transaction(
    request: web::Json<TransactionRequest>,
    hledger: web::Data<Arc<Hledger>>,
    templater: web::Data<Arc<Mutex<Templater<'_>>>>,
) -> HttpResponse {
    let description = templater
        .lock()
        .unwrap()
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

async fn get_transaction_stats(n26: web::Data<Arc<N26>>) -> HttpResponse {
    // Get real transactions
    let n26_transactions = n26.get_transactions().await;
    let mut fields_map = HashMap::<String, BTreeMap<String, u32>>::new();
    for t in n26_transactions.iter() {
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
