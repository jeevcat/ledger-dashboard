use std::{io, sync::Arc};

use actix_cors::Cors;
use actix_web::{middleware, App, HttpServer};

use crate::{alpha_vantage, api, db, hledger, ib::Ib, n26, prices, saltedge};

pub async fn run_server() -> io::Result<()> {
    let db = Arc::new(db::Database::new());
    let n26 = Arc::new(n26::N26::new(db.clone()));
    let saltedge = Arc::new(saltedge::SaltEdge::new());
    let ib = Arc::new(Ib {});
    let hledger = Arc::new(hledger::Hledger::new());
    let alpha_vantage = Arc::new(alpha_vantage::AlphaVantage::new());
    let prices = Arc::new(prices::Prices::new(alpha_vantage.clone()));

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .supports_credentials()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .data(n26.clone())
            .data(saltedge.clone())
            .data(ib.clone())
            .data(hledger.clone())
            .data(db.clone())
            .data(prices.clone())
            .service(api::rules::rules_routes())
            .service(api::rules::rule_routes())
            .service(api::transactions::routes::transactions_routes())
            .service(api::accounts::accounts_routes())
            .service(api::balance::balance_routes())
            .service(api::reports::reports_routes())
            .service(api::prices::prices_routes())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
