use std::time::Duration;

use actix::clock::sleep;
use async_trait::async_trait;
use chrono::NaiveDate;
use log::info;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;

use crate::{
    config,
    import_account::ImportAccount,
    model::{balance::RealBalance, real_transaction::RealTransaction},
};

const DATETIME_FMT: &str = "%Y%m%d;%H%M%S";
const DATE_FMT: &str = "%Y%m%d";
const MAX_RETRIES: u32 = 10;
const FIRST_RETRY_DELAY: u64 = 10;
const BASE_CURRENCY: &str = "EUR";

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
    fx_positions: Option<FxPositions>,
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
struct FxPositions {
    #[serde(rename = "FxPosition", default)]
    items: Vec<FxPosition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    currency: String,
    symbol: String,
    description: String,
    #[serde(rename = "transactionID")]
    transaction_id: String,
    date_time: String,
    quantity: i32,
    trade_price: Decimal,
    trade_money: Decimal,
    ib_commission: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    position: Decimal,
    mark_price: Decimal,
    position_value: Decimal,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FxPosition {
    /// The default account currency
    functional_currency: String,
    /// The currency of this forex position
    fx_currency: String,
    /// The amount of forex currency
    quantity: Decimal,
    /// The value of the forex currency in the default account currency
    value: Decimal,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum IbTransaction {
    Cash(CashTransaction),
    Trade(Trade),
}

impl RealTransaction for IbTransaction {
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

    fn get_default_amount_field_name(&self) -> &str {
        match self {
            IbTransaction::Cash(_) => "amount",
            IbTransaction::Trade(_) => "tradeMoney",
        }
    }

    fn get_default_currency_field_name(&self) -> &str {
        "currency"
    }
}

pub async fn get_balances() -> Vec<RealBalance> {
    let token = config::ib_flex_token();
    let query_id = config::ib_flex_balance_query_id();
    let balance = fetch_flex_statement(token, query_id).await;

    let positions = balance.open_positions.into_iter().flat_map(|x| {
        x.items.into_iter().map(|op| RealBalance {
            commodity: op.symbol,
            amount: op.position,
            base_amount: Some(op.position_value),
        })
    });

    let forex = balance.fx_positions.into_iter().flat_map(|x| {
        x.items.into_iter().flat_map(|fx| {
            if fx.functional_currency == BASE_CURRENCY {
                Some(RealBalance {
                    base_amount: if fx.functional_currency.as_str() == fx.fx_currency.as_str() {
                        None
                    } else {
                        Some(fx.value)
                    },
                    commodity: fx.fx_currency,
                    amount: fx.quantity,
                })
            } else {
                None
            }
        })
    });

    positions.chain(forex).collect()
}

async fn retried_request<'de, T: Deserialize<'de>>(url: &str) -> T {
    let mut retries = MAX_RETRIES;
    let mut wait = FIRST_RETRY_DELAY;
    loop {
        let text = reqwest::get(url).await.unwrap().text().await.unwrap();
        if let Ok(already_available) = from_str(&text) {
            return already_available;
        }
        let response: FlexStatatementStatusResponse = from_str(&text).unwrap();
        match response.status {
            FlexStatementStatus::Success => return from_str(&text).unwrap(),
            _ => {
                if retries > 0 {
                    info!("Flex not ready yet. Waiting {} sec...", wait);
                    retries -= 1;
                    sleep(Duration::from_secs(wait)).await;
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
    let token = config::ib_flex_token();
    let query_id = config::ib_flex_transactions_query_id();
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

async fn fetch_flex_statement(token: String, query_id: String) -> FlexStatement {
    let reference_code = enqueue_flex_statement_request(token.clone(), query_id).await;
    get_flex_statement(&reference_code, &token).await
}

/// Returns statement reference code
/// Cache this for a day so we avoid re-queueing flex statement requests
async fn enqueue_flex_statement_request(token: String, query_id: String) -> String {
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
    NaiveDate::parse_from_str(date_str, DATETIME_FMT).unwrap_or_else(|_| {
        NaiveDate::parse_from_str(date_str, DATE_FMT)
            .unwrap_or_else(|e| panic!("Couldn't parse {}. Error: {}", date_str, e))
    })
}

#[async_trait]
impl ImportAccount for Ib {
    type RealTransactionType = IbTransaction;

    async fn get_transactions(&self) -> Vec<Self::RealTransactionType> {
        get_transactions().await
    }

    async fn get_balances(&self) -> Vec<RealBalance> {
        get_balances().await
    }

    fn get_hledger_account(&self) -> &str {
        "Assets:Investments:IB"
    }

    fn get_id(&self) -> &str {
        "ib"
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::{prelude::FromPrimitive, Decimal};
    use serde_xml_rs::from_reader;

    use super::{get_transactions, IbTransaction, Trade};
    use crate::{
        ib::{get_balances, FlexStatementRequestResponse},
        model::{
            hledger_transaction::HledgerTransaction,
            rule::{RulePosting, RulePostingPrice},
        },
    };

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

    #[test]
    fn get_ib_postings() {
        let t = Trade {
            currency: "EUR".to_string(),
            symbol: "EMIM".to_string(),
            description: "ISHARES CORE EM IMI ACC".to_string(),
            transaction_id: "101876974".to_string(),
            date_time: "20210305;044915".to_string(),
            quantity: 322,
            trade_price: Decimal::from_f32(31.073).unwrap(),
            trade_money: Decimal::from_f32(10005.51).unwrap(),
            ib_commission: Decimal::from_f32(-10.00551).unwrap(),
        };
        let t = IbTransaction::Trade(t);
        let h = HledgerTransaction::new_with_postings(
            &t,
            "test",
            "test desc",
            &[RulePosting {
                amount_field_name: Some("quantity".to_string()),
                currency_field_name: Some("symbol".to_string()),
                price: Some(RulePostingPrice {
                    amount_field_name: "tradePrice".to_string(),
                    currency_field_name: "currency".to_string(),
                }),
                account: "Investments".to_string(),
                negate: false,
                comment: None,
            }],
        );
        println!("{:#?}", h);
    }

    #[actix_rt::test]
    #[ignore = "Contacts external service"]
    async fn check_get_balance() {
        dotenv::dotenv().ok();
        let bal = get_balances().await;
        println!("{:#?}", bal);
    }

    #[actix_rt::test]
    #[ignore = "Contacts external service"]
    async fn check_get_transactions() {
        dotenv::dotenv().ok();
        let t = get_transactions().await;
        println!("{:#?}", t);
    }
}
