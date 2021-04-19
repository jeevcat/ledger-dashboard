use std::sync::Arc;

use actix_web::{dev::HttpServiceFactory, web, HttpResponse};

use crate::hledger::Hledger;

pub fn reports_routes() -> impl HttpServiceFactory {
    web::scope("/reports").route("/income_statement", web::get().to(get_income_statement))
}

async fn get_income_statement(hledger: web::Data<Arc<Hledger>>) -> HttpResponse {
    let response = hledger.get_income_statement().await;
    HttpResponse::Ok().json(response)
}
