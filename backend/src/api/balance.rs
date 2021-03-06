use std::{cmp::Reverse, sync::Arc};

use actix_web::{dev::HttpServiceFactory, web, HttpResponse};
use rust_decimal::Decimal;

use super::CacheQuery;
use crate::{
    db::Database,
    hledger::Hledger,
    ib::Ib,
    import_account::ImportAccount,
    model::balance::{BalanceResponse, BalancesResponse},
    n26::N26,
    saltedge::SaltEdge,
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
    let hledger = hledger.get_account_balance(account).await;
    if hledger.is_empty() {
        return HttpResponse::InternalServerError().finish();
    }
    let mut balances: Vec<BalanceResponse> = real
        .into_iter()
        .map(|b| BalanceResponse {
            hledger: *hledger
                .get(b.commodity.as_str())
                .unwrap_or(&Decimal::new(0, 0)),
            commodity: b.commodity,
            real: b.amount,
            real_euro: b.base_amount,
        })
        .collect();
    balances.sort_by_key(|x| Reverse(x.real_euro));
    let response = BalancesResponse { balances };

    HttpResponse::Ok().json(response)
}
