use std::{
    io,
    sync::{Arc, Mutex},
};

use actix_cors::Cors;
use actix_web::{middleware, App, HttpServer};

use crate::{alpha_vantage, api, db, hledger, n26, prices, templater};

pub async fn run_server(db: Arc<db::Database>, n26: Arc<n26::N26>) -> io::Result<()> {
    let templater = Arc::new(Mutex::new(templater::Templater::new(&db).unwrap()));
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
            .data(hledger.clone())
            .data(db.clone())
            .data(templater.clone())
            .data(prices.clone())
            .service(api::rules::rules_routes())
            .service(api::rules::rule_routes())
            .service(api::transactions::transactions_routes())
            .service(api::accounts::accounts_routes())
            .service(api::prices::prices_routes())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
