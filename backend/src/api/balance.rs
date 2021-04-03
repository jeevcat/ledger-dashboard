use std::sync::Arc;

use actix_web::{dev::HttpServiceFactory, web, HttpResponse};
use rust_decimal::Decimal;

use crate::{hledger::Hledger, model::balance_response::BalanceResponse};

pub fn balance_routes() -> impl HttpServiceFactory {
    web::resource("/balance/{account_id}").route(web::get().to(get_account_balance))
}

async fn get_account_balance(
    account_id: web::Path<String>,
    hledger: web::Data<Arc<Hledger>>,
) -> HttpResponse {
    // TODO: don't hardcode this
    let account = match account_id.as_str() {
        "n26" => "Assets:Cash:N26",
        "ing" => "Assets:Cash:ING",
        "ib" => "Assets:Cash:IB",
        _ => return HttpResponse::BadRequest().finish(),
    };
    let recorded = match hledger.get_account_balance(account).await {
        Some(recorded) => recorded,
        None => return HttpResponse::InternalServerError().finish(),
    };
    let response = BalanceResponse {
        recorded,
        real: Decimal::from(0),
    };
    HttpResponse::Ok().json(response)
}
