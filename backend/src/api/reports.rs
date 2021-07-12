use std::sync::Arc;

use actix_web::{dev::HttpServiceFactory, web, HttpResponse};
use chrono::NaiveDate;
use serde::Deserialize;

use crate::hledger::Hledger;

pub fn reports_routes() -> impl HttpServiceFactory {
    web::scope("/reports")
        .route("/income_statement", web::get().to(get_income_statement))
        .route("/net_worth", web::get().to(get_net_worth))
}

#[derive(Deserialize)]
struct IncomeStatementQuery {
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
}

async fn get_income_statement(
    hledger: web::Data<Arc<Hledger>>,
    query: web::Query<IncomeStatementQuery>,
) -> HttpResponse {
    let response = hledger.get_income_statement(query.from, query.to).await;
    HttpResponse::Ok().json(response)
}

async fn get_net_worth(
    hledger: web::Data<Arc<Hledger>>,
    query: web::Query<IncomeStatementQuery>,
) -> HttpResponse {
    let response = hledger.get_net_worth(query.from, query.to).await;
    HttpResponse::Ok().json(response)
}
