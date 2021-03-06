use std::{collections::HashMap, thread, time};

use chrono::NaiveDate;
use log::info;
use rust_decimal::Decimal;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{config, prices::Price};

const BASE_URL: &str = "https://www.alphavantage.co/query";
const DATE_FMT: &str = "%Y-%m-%d";

#[derive(Debug, Deserialize)]
struct ThrottledHelper {
    #[serde(rename = "Error Message", alias = "Note")]
    error_message: String,
}

#[derive(Debug, Deserialize)]
struct MatchesHelper {
    #[serde(rename = "bestMatches")]
    best_matches: Vec<SearchHelper>,
}

#[derive(Debug, Deserialize)]
struct SearchHelper {
    #[serde(rename = "1. symbol")]
    symbol: String,
    #[serde(rename = "2. name")]
    name: String,
    #[serde(rename = "3. type")]
    symbol_type: String,
    #[serde(rename = "4. region")]
    regiob: String,
    #[serde(rename = "5. marketOpen")]
    market_open: String,
    #[serde(rename = "6. marketClose")]
    market_close: String,
    #[serde(rename = "7. timezone")]
    timezone: String,
    #[serde(rename = "8. currency")]
    currency: String,
    #[serde(rename = "9. matchScore")]
    match_score: String,
}

#[derive(Debug, Deserialize)]
struct EntryHelper {
    #[serde(alias = "4. close", alias = "4a. close (EUR)")]
    pub close: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct TimeSeriesHelper {
    #[serde(rename = "Error Message")]
    error: Option<String>,
    #[serde(rename = "Meta Data")]
    metadata: Option<HashMap<String, String>>,
    #[serde(
        alias = "Weekly Time Series",
        alias = "Time Series FX (Weekly)",
        alias = "Time Series (Digital Currency Weekly)"
    )]
    time_series: HashMap<String, EntryHelper>,
}

pub struct AlphaVantage {
    http_client: reqwest::Client,
}

impl AlphaVantage {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_weekly_stocks(
        &self,
        from_commodity: &str,
        to_commodity: &str,
    ) -> Result<Vec<Price>, Box<dyn std::error::Error>> {
        let request_symbol = format!("{}.DE", from_commodity);

        info!(
            "Looking up prices for stock {}->{} using Alpha Vantage...",
            request_symbol, to_commodity
        );

        let time_series: TimeSeriesHelper = self
            .alpha_vantage_request("TIME_SERIES_WEEKLY", &[("symbol", &request_symbol)])
            .await?;

        Ok(time_series
            .time_series
            .iter()
            .map(|(date_str, entry)| Price {
                date: NaiveDate::parse_from_str(date_str, DATE_FMT).unwrap(),
                from_commodity: from_commodity.to_string(),
                to_commodity: to_commodity.to_string(),
                amount: entry.close,
            })
            .collect())
    }

    pub async fn fetch_weekly_forex(
        &self,
        from_commodity: &str,
        to_commodity: &str,
    ) -> Result<Vec<Price>, Box<dyn std::error::Error>> {
        info!(
            "Looking up prices for forex {}->{} using Alpha Vantage...",
            from_commodity, to_commodity
        );

        let time_series: TimeSeriesHelper = self
            .alpha_vantage_request(
                "FX_WEEKLY",
                &[("from_symbol", from_commodity), ("to_symbol", to_commodity)],
            )
            .await?;

        Ok(time_series
            .time_series
            .iter()
            .map(|(date_str, entry)| Price {
                date: NaiveDate::parse_from_str(date_str, DATE_FMT).unwrap(),
                from_commodity: from_commodity.to_string(),
                to_commodity: to_commodity.to_string(),
                amount: entry.close,
            })
            .collect())
    }

    pub async fn fetch_weekly_crypto(
        &self,
        from_commodity: &str,
        to_commodity: &str,
    ) -> Result<Vec<Price>, Box<dyn std::error::Error>> {
        info!(
            "Looking up prices for crypto {}->{} using Alpha Vantage...",
            from_commodity, to_commodity
        );

        let time_series: TimeSeriesHelper = self
            .alpha_vantage_request(
                "DIGITAL_CURRENCY_WEEKLY",
                &[("symbol", from_commodity), ("market", to_commodity)],
            )
            .await?;

        Ok(time_series
            .time_series
            .iter()
            .map(|(date_str, entry)| Price {
                date: NaiveDate::parse_from_str(date_str, DATE_FMT).unwrap(),
                from_commodity: from_commodity.to_string(),
                to_commodity: to_commodity.to_string(),
                amount: entry.close,
            })
            .collect())
    }

    async fn alpha_vantage_request<Q: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        function: &str,
        query: &Q,
    ) -> Result<T, Box<dyn std::error::Error>> {
        loop {
            let req = self.alpha_vantage_request_internal(function, query).await;
            match &req {
                Ok(_) => return req,
                Err(err) => {
                    if err.downcast_ref::<reqwest::Error>().is_some() {
                        return req;
                    }
                    info!("Alpha Vantage request throttled. Waiting 10 seconds...");
                    info!("-> {:#?}", err);
                    thread::sleep(time::Duration::from_secs(10));
                }
            }
        }
    }

    async fn alpha_vantage_request_internal<Q: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        function: &str,
        query: &Q,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let response = self
            .http_client
            .get(BASE_URL)
            .query(&[
                ("function", function),
                (
                    "apikey",
                    &config::alpha_vantage_key().expect("Need to set ALPHA_VANTAGE_KEY"),
                ),
            ])
            .query(query)
            .send()
            .await?;
        response.error_for_status_ref()?;
        let full = response.bytes().await?;
        if let Ok(throttled) = serde_json::from_slice::<ThrottledHelper>(&full) {
            return Err(throttled.error_message.into());
        }
        Ok(serde_json::from_slice(&full)?)
    }
}
