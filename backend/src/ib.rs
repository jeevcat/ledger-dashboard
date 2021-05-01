use std::time::Duration;

use actix::clock::delay_for;
use actix_web::web::Buf;
use async_trait::async_trait;
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

pub struct Ib;

#[derive(Debug, Deserialize)]
enum FlexStatementStatus {
    Success,
    Warn,
    Fail,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FlexStatementRequestResponse {
    status: FlexStatementStatus,
    reference_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FlexStatementGetResponse {
    status: Option<FlexStatementStatus>,
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

impl Ib {
    pub async fn get_balance() -> Decimal {
        let token = &config::ib_flex_token().expect("Need to set IB_FLEX_TOKEN");
        let query_id =
            &config::ib_flex_balance_query_id().expect("Need to set IB_FLEX_BALANCE_QUERY_ID");
        let balance = Ib::fetch_flex_statement(token, query_id).await;
        println!("{:#?}", balance);
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

    pub async fn get_transactions() -> Vec<IbTransaction> {
        let token = &config::ib_flex_token().expect("Need to set IB_FLEX_TOKEN");
        let query_id = &config::ib_flex_transactions_query_id()
            .expect("Need to set IB_FLEX_TRANSACTIONS_QUERY_ID");
        let statement = Ib::fetch_flex_statement(token, query_id).await;
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
        let reference_code = Ib::request_flex_statement(token, query_id).await;

        let mut retries: i32 = 3;
        let mut wait: u64 = 1;
        loop {
            let statement = Ib::get_flex_statement(&reference_code, token).await;
            match statement.status {
                Some(status) => {
                    if let FlexStatementStatus::Warn = status {
                        if retries > 0 {
                            info!("Statement not ready yet. Waiting {} sec...", wait);
                            retries -= 1;
                            delay_for(Duration::from_secs(wait)).await;
                            wait *= 2;
                        } else {
                            panic!("Still couldn't get flex statement after 3 retries");
                        }
                    }
                }
                _ => {
                    return statement
                        .flex_statements
                        .unwrap()
                        .items
                        .into_iter()
                        .next()
                        .unwrap()
                }
            }
        }
    }

    /// Returns statement reference code
    async fn request_flex_statement(token: &str, query_id: &str) -> String {
        let url = format!("https://gdcdyn.interactivebrokers.com/Universal/servlet/FlexStatementService.SendRequest?t={}&q={}&v=3", token, query_id);
        let mut retries: i32 = 3;
        let mut wait: u64 = 1;
        loop {
            let text = reqwest::get(&url).await.unwrap().bytes().await.unwrap();
            let response: FlexStatementRequestResponse = from_reader(text.bytes()).unwrap();
            match response.status {
                FlexStatementStatus::Success => return response.reference_code.unwrap(),
                _ => {
                    if retries > 0 {
                        info!("Statement not ready yet. Waiting {} sec...", wait);
                        retries -= 1;
                        delay_for(Duration::from_secs(wait)).await;
                        wait *= 2;
                    }
                }
            }
        }
    }

    async fn get_flex_statement(reference_code: &str, token: &str) -> FlexStatementGetResponse {
        let url = format!("https://gdcdyn.interactivebrokers.com/Universal/servlet/FlexStatementService.GetStatement?q={}&t={}&v=3", reference_code, token);
        let text = reqwest::get(&url).await.unwrap().bytes().await.unwrap();
        from_reader(text.bytes()).unwrap()
    }
}

#[async_trait]
impl ImportAccount for Ib {
    type RealTransactionType = IbTransaction;

    async fn get_transactions(&self) -> Vec<Self::RealTransactionType> {
        Ib::get_transactions().await
    }

    async fn get_balance(&self) -> Decimal {
        Ib::get_balance().await
    }

    fn get_hledger_account(&self) -> &str {
        "Assets:Investments:IB"
    }
}

#[cfg(test)]
mod tests {
    use serde_xml_rs::from_reader;

    use super::Ib;
    use crate::ib::FlexStatementRequestResponse;

    #[test]
    fn deserialize_flex_response() {
        let xml = r#"<FlexStatementResponse timestamp='30 April, 2021 08:37 AM EDT'>
<Status>Success</Status>
<ReferenceCode>1234567890</ReferenceCode>
<Url>https://gdcdyn.interactivebrokers.com/Universal/servlet/FlexStatementService.GetStatement</Url>
</FlexStatementResponse>
"#;
        let response: FlexStatementRequestResponse = from_reader(xml.as_bytes()).unwrap();
        assert_eq!(response.reference_code.unwrap(), "1234567890");
    }

    #[actix_rt::test]
    async fn get_balance() {
        dotenv::dotenv().ok();
        let bal = Ib::get_balance().await;
        println!("{:#?}", bal);
    }

    #[actix_rt::test]
    async fn get_transactions() {
        dotenv::dotenv().ok();
        Ib::get_transactions().await;
    }
}

fn ib_date(date_str: &str) -> NaiveDate {
    NaiveDate::parse_from_str(date_str, DATE_FMT).unwrap()
}
