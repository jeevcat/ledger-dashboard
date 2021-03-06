use std::sync::Arc;

mod alpha_vantage;
mod api;
mod config;
mod db;
mod file_utils;
mod hledger;
mod http;
mod ib;
mod model;
mod n26;
mod prices;
mod templater;
mod transactions;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let db = Arc::new(db::Database::new());
    let n26 = Arc::new(n26::N26::new(db.clone()));

    // Start Web server
    http::run_server(db, n26).await
}
