use std::sync::Arc;

use actix_web::{dev::HttpServiceFactory, web, HttpResponse};

use crate::{hledger::Hledger, prices::Prices};

pub fn prices_routes() -> impl HttpServiceFactory {
    web::resource("/prices").route(web::post().to(update_prices))
}

async fn update_prices(
    prices: web::Data<Arc<Prices>>,
    hledger: web::Data<Arc<Hledger>>,
) -> HttpResponse {
    let commodities = hledger.get_commodities().await;
    println!("{:#?}", commodities);
    prices.update_prices(&commodities).await;
    HttpResponse::Ok().finish()
}
