use std::sync::Arc;

use actix_web::{dev::HttpServiceFactory, web, HttpResponse};

use crate::{
    hledger::Hledger, import_account::ImportAccount, model::balance_response::BalanceResponse,
    n26::N26, saltedge::SaltEdge,
};

pub fn balance_routes() -> impl HttpServiceFactory {
    web::scope("/balance")
        .route("/n26", web::get().to(get_account_balance::<N26>))
        .route("/ing", web::get().to(get_account_balance::<SaltEdge>))
}

async fn get_account_balance<T>(
    import_account: web::Data<Arc<T>>,
    hledger: web::Data<Arc<Hledger>>,
) -> HttpResponse
where
    T: ImportAccount,
{
    let real = import_account.get_balance().await;
    let account = import_account.get_hledger_account();
    let recorded = match hledger.get_account_balance(account).await {
        Some(recorded) => recorded,
        None => return HttpResponse::InternalServerError().finish(),
    };
    let response = BalanceResponse { recorded, real };
    HttpResponse::Ok().json(response)
}
