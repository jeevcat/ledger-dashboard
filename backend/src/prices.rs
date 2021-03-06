use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::{self, File},
    io,
    str::FromStr,
    sync::Arc,
};

use chrono::NaiveDate;
use io::BufRead;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{alpha_vantage::AlphaVantage, file_utils};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Price {
    pub date: NaiveDate,
    pub from_commodity: String,
    pub to_commodity: String,
    pub amount: Decimal,
}

const DATE_FMT: &str = "%Y/%m/%d";
const NATIVE_COMMODITY: &str = "EUR";

impl Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "P {} {} {} {}",
            self.date.format(DATE_FMT),
            format_commodity(&self.from_commodity),
            self.amount,
            format_commodity(&self.to_commodity),
        )
    }
}

// TODO: Better errors
impl FromStr for Price {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("Empty input string".into());
        }

        let parts: Vec<&str> = s.split(' ').collect();
        if parts.len() < 5 {
            return Err("Expected 5 elements in price string".into());
        }

        let from_commodity = parse_commodity(parts[2]);
        let to_commodity = parse_commodity(parts[4]);
        let amount = parts[3].parse::<Decimal>()?;
        let date = NaiveDate::parse_from_str(parts[1], DATE_FMT)?;
        Ok(Price {
            date,
            from_commodity,
            to_commodity,
            amount,
        })
    }
}

pub struct Prices {
    alpha_vantage: Arc<AlphaVantage>,
    currencies: HashSet<String>,
    cryptocurrencies: HashSet<String>,
}

impl Prices {
    pub fn new(alpha_vantage: Arc<AlphaVantage>) -> Self {
        let currencies_json = include_str!("currencies.json");
        let cryptocurrencies_json = include_str!("cryptocurrencies.json");
        Self {
            alpha_vantage,
            currencies: Self::load_currencies_from_disk(currencies_json),
            cryptocurrencies: Self::load_currencies_from_disk(cryptocurrencies_json),
        }
    }

    // Disk operations

    pub async fn update_prices(&self, commodities: &[String]) {
        let mut prices = Prices::read_prices();
        for commodity in commodities {
            let mut fetched_prices = self.fetch_prices(commodity, NATIVE_COMMODITY).await;
            prices.append(&mut fetched_prices);
        }
        prices.sort_by(|a, b| {
            a.date
                .partial_cmp(&b.date)
                .unwrap()
                .then(a.from_commodity.partial_cmp(&b.from_commodity).unwrap())
        });

        let start_date = NaiveDate::from_ymd(2017, 1, 1);
        prices.retain(|p| p.date > start_date);
        prices.dedup();

        Prices::write_prices(prices.iter()).unwrap();
    }

    fn read_prices() -> Vec<Price> {
        let filename = file_utils::get_prices_file().unwrap();
        let file = File::open(filename).unwrap();
        io::BufReader::new(file)
            .lines()
            .filter_map(|line| line.ok().map(|line| line.parse::<Price>().ok()).flatten())
            .collect()
    }

    fn write_prices<'a, T>(prices: T) -> io::Result<()>
    where
        T: IntoIterator<Item = &'a Price>,
    {
        let prices_string = prices
            .into_iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join("\n")
            // hledger requires newline at the end of a document
            + "\n";
        let filename = file_utils::get_prices_file().unwrap();
        fs::write(filename, prices_string)
    }

    fn load_currencies_from_disk(json: &str) -> HashSet<String> {
        let currencies: HashMap<String, String> =
            serde_json::from_str(json).unwrap_or_else(|_| panic!("Coudn't deserialize {}", json));
        currencies.into_iter().map(|(c, _)| c).collect()
    }

    // Web operations
    async fn fetch_prices(&self, from_commodity: &str, to_commodity: &str) -> Vec<Price> {
        if self.is_currency(from_commodity) || self.is_cryptocurrency(from_commodity) {
            vec![]
        } else {
            self.alpha_vantage
                .fetch_weekly_stocks(from_commodity, to_commodity)
                .await
                .unwrap()
        }
    }

    fn is_currency(&self, commodity: &str) -> bool {
        self.currencies.iter().any(|c| c == commodity)
    }

    fn is_cryptocurrency(&self, commodity: &str) -> bool {
        self.cryptocurrencies.iter().any(|c| c == commodity)
    }
}

fn format_commodity(commodity: &str) -> String {
    if commodity.chars().any(|c| c.is_numeric()) {
        return format!("\"{}\"", commodity);
    }
    commodity.to_string()
}

fn parse_commodity(commodity: &str) -> String {
    commodity.trim_matches('"').to_string()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    use super::{Price, Prices};

    #[test]
    fn price_to_string() {
        let price = Price {
            date: NaiveDate::from_ymd(2020, 12, 5),
            from_commodity: "USD".to_string(),
            to_commodity: "EUR".to_string(),
            amount: Decimal::new(123, 2),
        };
        assert_eq!(price.to_string(), "P 2020/12/05 USD 1.23 EUR");
    }

    #[test]
    fn price_from_string() {
        let price_string = "P 1995/03/12 BTC 1012.49 EUR";
        let price = price_string.parse::<Price>().unwrap();

        assert_eq!(price.date, NaiveDate::from_ymd(1995, 3, 12));
        assert_eq!(price.from_commodity, "BTC");
        assert_eq!(price.to_commodity, "EUR");
        assert_eq!(price.amount, Decimal::new(101249, 2));
    }

    #[test]
    fn price_from_string_quotes() {
        let price_string = r#"P 1995/03/12 "X010" 100 EUR"#;
        let price = price_string.parse::<Price>().unwrap();

        assert_eq!(price.date, NaiveDate::from_ymd(1995, 3, 12));
        assert_eq!(price.from_commodity, "X010");
        assert_eq!(price.to_commodity, "EUR");
        assert_eq!(price.amount, Decimal::new(10000, 2));
    }

    #[test]
    #[ignore = "writes to prices.ledger file"]
    fn read_write_prices_file() {
        let initial_prices = Prices::read_prices();
        Prices::write_prices(initial_prices.iter()).unwrap();
        let next_prices = Prices::read_prices();
        assert!(initial_prices
            .iter()
            .zip(next_prices.iter())
            .all(|(a, b)| a == b));
    }
}
