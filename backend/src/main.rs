mod alpha_vantage;
mod api;
mod auth;
mod config;
mod db;
mod file_utils;
mod git;
mod hledger;
mod http;
mod ib;
mod import_account;
mod model;
mod n26;
mod prices;
mod saltedge;
mod templater;
mod transactions;

#[cfg(test)]
mod test_statics;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info");
    //std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    // Update repo if needed
    git::checkout();

    // Start Web server
    http::run_server().await
}
