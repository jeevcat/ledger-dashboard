use std::sync::Arc;

use actix_web::{dev::HttpServiceFactory, web, HttpResponse};

use crate::hledger::Hledger;

pub fn accounts_routes() -> impl HttpServiceFactory {
    web::resource("/accounts").route(web::get().to(get_accounts))
}

async fn get_accounts(hledger: web::Data<Arc<Hledger>>) -> HttpResponse {
    let accounts = hledger.get_accounts().await;
    // Forward response directly, but manually set JSON
    HttpResponse::Ok()
        .content_type("application/json")
        .body(accounts)
}
