use std::{
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use chrono::NaiveDate;
use csv::StringRecord;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::real_transaction::RealTransaction;
use crate::ib::deserialize_record;

const DATE_FMT: &str = "%Y-%m-%d";
const DATE_TIME_FMT: &str = "%Y-%m-%d";

#[derive(Debug, Deserialize)]
struct Field {
    #[serde(rename = "Field Name")]
    field_name: String,
    #[serde(rename = "Field Value")]
    field_value: String,
}

// https://guides.interactivebrokers.com/rg/reportguide/trades_default.htm
#[derive(Debug, Hash, Serialize, Deserialize)]
struct Trade {
    #[serde(rename = "Currency")]
    currency: String,
    // The symbol of the instrument you traded.
    #[serde(rename = "Symbol")]
    symbol: String,
    // The date and the time of the execution.
    #[serde(rename = "Date/Time")]
    date_time: String,
    // The number of units for the transaction.
    #[serde(rename = "Quantity")]
    quantity: i32,
    // The transaction price.
    #[serde(rename = "T. Price")]
    t_price: Decimal,
    // The closing price of the instrument.
    #[serde(rename = "C. Price")]
    c_price: Decimal,
    #[serde(rename = "Proceeds")]
    // Calculated by mulitplying the quantity and the transaction price. The proceeds figure will be negative for buys and positive for sales.
    proceeds: Decimal,
    // The total amount of commission and fees for the transaction.
    #[serde(rename = "Comm/Fee")]
    comm_fee: Decimal,
    // The basis of an opening trade is the inverse of proceeds plus commission and tax amount. For closing trades, the basis is the basis of the opening trade.
    #[serde(rename = "Basis")]
    basis: Decimal,
    // Calculated by adding the proceeds of the closing trade plus commissions and then adding the basis.
    #[serde(rename = "Realized P/L")]
    realized_p_l: Decimal,
    #[serde(rename = "Realized P/L %")]
    realized_p_l_percent: Decimal,
    // The difference between the transaction price and closing price multiplied by the quantity.
    #[serde(rename = "MTM P/L")]
    mtm_p_l: Decimal,
}

impl RealTransaction for Trade {
    fn get_id(&self) -> Cow<str> {
        Cow::Owned(hash(self))
    }

    fn get_date(&self) -> NaiveDate {
        NaiveDate::parse_from_str(&self.date_time, DATE_TIME_FMT).unwrap()
    }

    fn get_amount(&self) -> Decimal {
        self.t_price
    }

    fn get_currency(&self) -> &str {
        self.currency.as_str()
    }

    fn get_account(&self) -> &str {
        "Assets:Cash:IB"
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct SimpleTransaction {
    #[serde(rename = "Currency")]
    currency: String,
    #[serde(rename(deserialize = "Date", deserialize = "Settle Date", serialize = "Date"))]
    date: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Amount")]
    amount: Decimal,
}

impl RealTransaction for SimpleTransaction {
    fn get_id(&self) -> Cow<str> {
        Cow::Owned(hash(self))
    }

    fn get_date(&self) -> NaiveDate {
        NaiveDate::parse_from_str(&self.date, DATE_FMT).unwrap()
    }

    fn get_amount(&self) -> Decimal {
        self.amount
    }

    fn get_currency(&self) -> &str {
        self.currency.as_str()
    }

    fn get_account(&self) -> &str {
        "Assets:Cash:IB"
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum IbTransaction {
    SimpleTransaction(SimpleTransaction),
    Trade(Trade),
}

impl RealTransaction for IbTransaction {
    fn get_id(&self) -> Cow<str> {
        match self {
            IbTransaction::SimpleTransaction(t) => t.get_id(),
            IbTransaction::Trade(t) => t.get_id(),
        }
    }

    fn get_date(&self) -> NaiveDate {
        match self {
            IbTransaction::SimpleTransaction(t) => t.get_date(),
            IbTransaction::Trade(t) => t.get_date(),
        }
    }

    fn get_amount(&self) -> Decimal {
        match self {
            IbTransaction::SimpleTransaction(t) => t.get_amount(),
            IbTransaction::Trade(t) => t.get_amount(),
        }
    }

    fn get_currency(&self) -> &str {
        match self {
            IbTransaction::SimpleTransaction(t) => t.get_currency(),
            IbTransaction::Trade(t) => t.get_currency(),
        }
    }

    fn get_account(&self) -> &str {
        match self {
            IbTransaction::SimpleTransaction(t) => t.get_account(),
            IbTransaction::Trade(t) => t.get_account(),
        }
    }
}

#[derive(Debug, Default)]
pub struct IbReport {
    statement: Vec<Field>,
    account_info: Vec<Field>,
    nav_change: Vec<Field>,
    trades: Vec<Trade>,
    deposits_withdrawals: Vec<SimpleTransaction>,
    fees: Vec<SimpleTransaction>,
    dividends: Vec<SimpleTransaction>,
    withholding_tax: Vec<SimpleTransaction>,
}

impl IbReport {
    pub fn deserialize_to_report(&mut self, record: &StringRecord, headers: &StringRecord) {
        let data_type = &record[0];
        match data_type {
            "Statement" => deserialize_record(&record, &headers, &mut self.statement),
            "Account Information" => deserialize_record(&record, &headers, &mut self.account_info),
            "Change in NAV" => deserialize_record(&record, &headers, &mut self.nav_change),
            "Trades" => deserialize_record(&record, &headers, &mut self.trades),
            "Deposits & Withdrawals" => {
                deserialize_record(&record, &headers, &mut self.deposits_withdrawals)
            }
            "Fees" => deserialize_record(&record, &headers, &mut self.fees),
            "Dividends" => deserialize_record(&record, &headers, &mut self.dividends),
            "Withholding Tax" => deserialize_record(&record, &headers, &mut self.withholding_tax),
            _ => (),
        }
    }

    pub fn get_transactions(self) -> impl Iterator<Item = impl RealTransaction> {
        self.deposits_withdrawals
            .into_iter()
            .map(IbTransaction::SimpleTransaction)
            .chain(self.fees.into_iter().map(IbTransaction::SimpleTransaction))
            .chain(
                self.dividends
                    .into_iter()
                    .map(IbTransaction::SimpleTransaction),
            )
            .chain(
                self.withholding_tax
                    .into_iter()
                    .map(IbTransaction::SimpleTransaction),
            )
            .chain(self.trades.into_iter().map(IbTransaction::Trade))
    }
}

fn hash<T: Hash>(object: &T) -> String {
    let mut hasher = DefaultHasher::new();
    object.hash(&mut hasher);
    hasher.finish().to_string()
}
