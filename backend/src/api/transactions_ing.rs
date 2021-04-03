use std::sync::Arc;

use actix_web::{web, HttpResponse};

use super::transactions_utils;
use crate::{db::Database, hledger::Hledger, saltedge::SaltEdge};

// Get transactions whose ids match
pub async fn get_existing_transactions(
    saltedge: web::Data<Arc<SaltEdge>>,
    hledger: web::Data<Arc<Hledger>>,
) -> HttpResponse {
    transactions_utils::get_existing_transactions(&**saltedge.into_inner(), &hledger).await
}

// Get transactions which were able to be generated from rules
pub async fn get_generated_transactions(
    saltedge: web::Data<Arc<SaltEdge>>,
    hledger: web::Data<Arc<Hledger>>,
    db: web::Data<Arc<Database>>,
) -> HttpResponse {
    transactions_utils::get_generated_transactions(&**saltedge.into_inner(), &hledger, &db).await
}

pub async fn write_generated_transactions(
    saltedge: web::Data<Arc<SaltEdge>>,
    hledger: web::Data<Arc<Hledger>>,
    db: web::Data<Arc<Database>>,
) -> HttpResponse {
    transactions_utils::write_generated_transactions(&**saltedge.into_inner(), &hledger, &db).await
}

// Get remaining real transactions which no rules matched
pub async fn get_unmatched_transactions(
    saltedge: web::Data<Arc<SaltEdge>>,
    hledger: web::Data<Arc<Hledger>>,
    db: web::Data<Arc<Database>>,
) -> HttpResponse {
    transactions_utils::get_unmatched_transactions(&**saltedge.into_inner(), &hledger, &db).await
}
