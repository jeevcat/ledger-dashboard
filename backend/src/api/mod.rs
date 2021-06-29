pub mod accounts;
pub mod balance;
pub mod journal;
pub mod prices;
pub mod reports;
pub mod rules;
pub mod transactions;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CacheQuery {
    bypass_cache: Option<bool>,
}

impl CacheQuery {
    fn bypass_cache(&self) -> bool {
        self.bypass_cache.unwrap_or(false)
    }
}
