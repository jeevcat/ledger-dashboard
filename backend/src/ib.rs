use std::time::Duration;

use actix::clock::delay_for;
use actix_web::web::Buf;
use async_trait::async_trait;
use cached::proc_macro::cached;
use chrono::NaiveDate;
use log::info;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_xml_rs::from_reader;

use crate::{
    config,
    import_account::ImportAccount,
    model::real_transaction::{DefaultPostingTransaction, IdentifiableTransaction},
};

const DATE_FMT: &str = "%Y%m%d;%H%M%S";
const MAX_RETRIES: u32 = 10;
const FIRST_RETRY_DELAY: u32 = 10;

pub struct Ib;

#[derive(Debug, Deserialize)]
enum FlexStatementStatus {
    Success,
    Warn,
    Fail,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FlexStatatementStatusResponse {
    status: FlexStatementStatus,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FlexStatementRequestResponse {
    reference_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FlexStatementGetResponse {
    flex_statements: Option<FlexStatements>,
}

#[derive(Debug, Deserialize)]
struct FlexStatements {
    #[serde(rename = "FlexStatement", default)]
    items: Vec<FlexStatement>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FlexStatement {
    trades: Option<Trades>,
    cash_transactions: Option<CashTransactions>,
    open_positions: Option<OpenPositions>,
    cash_report: Option<CashReports>,
}

#[derive(Debug, Deserialize)]
struct Trades {
    #[serde(rename = "Trade", default)]
    items: Vec<Trade>,
}

#[derive(Debug, Deserialize)]
struct CashTransactions {
    #[serde(rename = "CashTransaction", default)]
    items: Vec<CashTransaction>,
}

#[derive(Debug, Deserialize)]
struct OpenPositions {
    #[serde(rename = "OpenPosition", default)]
    items: Vec<OpenPosition>,
}

#[derive(Debug, Deserialize)]
struct CashReports {
    #[serde(rename = "CashReportCurrency", default)]
    items: Vec<CashReport>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    currency: String,
    symbol: String,
    description: String,
    #[serde(rename = "transactionID")]
    transaction_id: String,
    date_time: String,
    quantity: u32,
    trade_price: Decimal,
    trade_money: Decimal,
    ib_commission: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashTransaction {
    currency: String,
    description: String,
    #[serde(rename = "transactionID")]
    transaction_id: String,
    date_time: String,
    amount: Decimal,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenPosition {
    currency: String,
    symbol: String,
    description: String,
    position: u32,
    mark_price: Decimal,
    position_value: Decimal,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CashReport {
    currency: String,
    ending_cash: Decimal,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum IbTransaction {
    Cash(CashTransaction),
    Trade(Trade),
}

impl IdentifiableTransaction for IbTransaction {
    fn get_id(&self) -> std::borrow::Cow<str> {
        match self {
            IbTransaction::Cash(c) => c.transaction_id.as_str().into(),
            IbTransaction::Trade(t) => t.transaction_id.as_str().into(),
        }
    }

    fn get_date(&self) -> chrono::NaiveDate {
        match self {
            IbTransaction::Cash(c) => ib_date(&c.date_time),
            IbTransaction::Trade(t) => ib_date(&t.date_time),
        }
    }
}
impl DefaultPostingTransaction for IbTransaction {
    fn get_amount(&self) -> Decimal {
        Decimal::ZERO
    }

    fn get_currency(&self) -> &str {
        "FAKE"
    }

    fn get_account(&self) -> &str {
        "FAKE:ACCOUNT"
    }
}

#[cached(time = 600)]
pub async fn get_balance() -> Decimal {
    let token = &config::ib_flex_token().expect("Need to set IB_FLEX_TOKEN");
    let query_id =
        &config::ib_flex_balance_query_id().expect("Need to set IB_FLEX_BALANCE_QUERY_ID");
    let balance = fetch_flex_statement(token, query_id).await;
    let position_sum = balance
        .open_positions
        .unwrap()
        .items
        .into_iter()
        .fold(Decimal::ZERO, |acc, p| acc + p.position_value);
    let cash_sum = balance
        .cash_report
        .unwrap()
        .items
        .into_iter()
        .find(|c| c.currency == "BASE_SUMMARY")
        .unwrap()
        .ending_cash;
    position_sum + cash_sum
}

async fn retried_request<'de, T: Deserialize<'de>>(url: &str) -> T {
    let mut retries = MAX_RETRIES;
    let mut wait = FIRST_RETRY_DELAY;
    loop {
        let text = reqwest::get(url).await.unwrap().bytes().await.unwrap();
        let response: FlexStatatementStatusResponse = from_reader(text.bytes()).unwrap();
        match response.status {
            FlexStatementStatus::Success => return from_reader(text.bytes()).unwrap(),
            _ => {
                if retries > 0 {
                    info!("Flex not ready yet. Waiting {} sec...", wait);
                    retries -= 1;
                    delay_for(Duration::from_secs(wait)).await;
                    wait *= 2;
                } else {
                    panic!(
                        "Still couldn't request flex statement after {} retries",
                        MAX_RETRIES
                    );
                }
            }
        }
    }
}

async fn get_transactions() -> Vec<IbTransaction> {
    let token = &config::ib_flex_token().expect("Need to set IB_FLEX_TOKEN");
    let query_id = &config::ib_flex_transactions_query_id()
        .expect("Need to set IB_FLEX_TRANSACTIONS_QUERY_ID");
    let statement = fetch_flex_statement(token, query_id).await;
    let trades = statement
        .trades
        .into_iter()
        .flat_map(|t| t.items)
        .map(IbTransaction::Trade);
    let cash = statement
        .cash_transactions
        .into_iter()
        .flat_map(|t| t.items)
        .map(IbTransaction::Cash);
    trades.chain(cash).collect()
}

async fn fetch_flex_statement(token: &str, query_id: &str) -> FlexStatement {
    let reference_code = enqueue_flex_statement_request(token, query_id).await;
    get_flex_statement(&reference_code, token).await
}

/// Returns statement reference code
/// Cache this for a day so we avoid re-queueing flex statement requests
#[cached(time = 86400)]
async fn enqueue_flex_statement_request(token: &str, query_id: &str) -> String {
    let url = format!("https://gdcdyn.interactivebrokers.com/Universal/servlet/FlexStatementService.SendRequest?t={}&q={}&v=3", token, query_id);
    let response: FlexStatementRequestResponse = retried_request(&url).await;
    response.reference_code
}

async fn get_flex_statement(reference_code: &str, token: &str) -> FlexStatement {
    let url = format!("https://gdcdyn.interactivebrokers.com/Universal/servlet/FlexStatementService.GetStatement?q={}&t={}&v=3", reference_code, token);
    info!("Getting fetch statement using {}", &url);
    let response: FlexStatementGetResponse = retried_request(&url).await;
    response
        .flex_statements
        .unwrap()
        .items
        .into_iter()
        .next()
        .unwrap()
}

fn ib_date(date_str: &str) -> NaiveDate {
    NaiveDate::parse_from_str(date_str, DATE_FMT).unwrap()
}

#[async_trait]
impl ImportAccount for Ib {
    type RealTransactionType = IbTransaction;

    async fn get_transactions(&self) -> Vec<Self::RealTransactionType> {
        get_transactions().await
    }

    async fn get_balance(&self) -> Decimal {
        get_balance().await
    }

    fn get_hledger_account(&self) -> &str {
        "Assets:Investments:IB"
    }
}

#[cfg(test)]
mod tests {
    use serde_xml_rs::from_reader;

    use super::get_transactions;
    use crate::ib::{get_balance, FlexStatementRequestResponse};

    #[test]
    fn deserialize_flex_response() {
        let xml = r#"<FlexStatementResponse timestamp='30 April, 2021 08:37 AM EDT'>
<Status>Success</Status>
<ReferenceCode>1234567890</ReferenceCode>
<Url>https://gdcdyn.interactivebrokers.com/Universal/servlet/FlexStatementService.GetStatement</Url>
</FlexStatementResponse>
"#;
        let response: FlexStatementRequestResponse = from_reader(xml.as_bytes()).unwrap();
        assert_eq!(response.reference_code, "1234567890");
    }

    #[actix_rt::test]
    async fn check_get_balance() {
        dotenv::dotenv().ok();
        let bal = get_balance().await;
        println!("{:#?}", bal);
    }

    #[actix_rt::test]
    async fn check_get_transactions() {
        dotenv::dotenv().ok();
        get_transactions().await;
    }
}
