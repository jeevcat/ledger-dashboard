use std::sync::Arc;

use actix_web::{dev::HttpServiceFactory, web, HttpResponse};

use super::CacheQuery;
use crate::{
    db::Database, hledger::Hledger, ib::Ib, import_account::ImportAccount,
    model::balance_response::BalanceResponse, n26::N26, saltedge::SaltEdge,
};

pub fn balance_routes() -> impl HttpServiceFactory {
    web::scope("/balance")
        .route("/n26", web::get().to(get_account_balance::<N26>))
        .route("/ing", web::get().to(get_account_balance::<SaltEdge>))
        .route("/ib", web::get().to(get_account_balance::<Ib>))
}

async fn get_account_balance<T>(
    import_account: web::Data<Arc<T>>,
    hledger: web::Data<Arc<Hledger>>,
    db: web::Data<Arc<Database>>,
    query: web::Query<CacheQuery>,
) -> HttpResponse
where
    T: ImportAccount + Sync,
{
    let real = import_account
        .get_balance_cached(&db, query.bypass_cache())
        .await;
    let account = import_account.get_hledger_account();
    let hledger = match hledger.get_account_balance(account).await {
        Some(hledger) => hledger,
        None => return HttpResponse::InternalServerError().finish(),
    };
    let response = BalanceResponse { hledger, real };
    HttpResponse::Ok().json(response)
}
