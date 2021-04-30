use std::{time::Duration};

use actix::clock::delay_for;
use actix_web::web::Buf;
use async_trait::async_trait;
use csv::StringRecord;
use log::{error, info};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_xml_rs::from_reader;

use crate::{
    config,
    file_utils::get_database_file,
    import_account::ImportAccount,
    model::ib_report::{IbReport, IbTransaction},
};

pub struct Ib;

#[derive(Debug, Deserialize)]
enum FlexStatementStatus {
    Success,
    Warn,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FlexStatementRequestResponse {
    status: FlexStatementStatus,
    reference_code: String,
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
    trades: Trades,
    cash_transactions: CashTransactions,
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
#[serde(rename_all = "camelCase")]
struct Trade {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CashTransaction {
    currency: String,
    description: String,
    #[serde(rename = "transactionID")]
    transaction_id: String,
    date_time: String,
    amount: Decimal,
}

impl Ib {
    pub fn read_report() -> IbReport {
        let filename = "ib.csv";
        let file_path = get_database_file(filename).unwrap();
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_path(file_path)
            .unwrap();
        let mut report = IbReport::default();
        let mut record = StringRecord::new();
        while rdr.read_record(&mut record).unwrap_or_default() {
            match &record[1] {
                "Header" => rdr.set_headers(record.clone()),
                "Data" => {
                    let headers = rdr.headers().unwrap();
                    report.deserialize_to_report(&record, headers);
                }
                _ => (),
            }
        }
        report
    }

    pub async fn get_statement() -> Decimal {
        let token = &config::ib_flex_token().expect("Need to set IB_FLEX_TOKEN");
        let query_id = &config::ib_flex_query_id().expect("Need to set IB_FLEX_QUERY_ID");
        let url = format!("https://gdcdyn.interactivebrokers.com/Universal/servlet/FlexStatementService.SendRequest?t={}&q={}&v=3", token, query_id);
        let text = reqwest::get(&url).await.unwrap().bytes().await.unwrap();
        let response: FlexStatementRequestResponse = from_reader(text.bytes()).unwrap();
        println!("{:#?}", response);

        let mut retries: i32 = 3;
        let mut wait: u64 = 1;
        let res = loop {
            let statement = Ib::get_flex_statement(&response.reference_code, token).await;
            match statement.status {
                Some(status) => {
                    if let FlexStatementStatus::Warn = status {
                        if retries > 0 {
                            info!("Statement not ready yet. Waiting {} sec...", wait);
                            retries -= 1;
                            delay_for(Duration::from_secs(wait)).await;
                            wait *= 2;
                        }
                    }
                }
                _ => break statement.flex_statements,
            }
        };
        println!("{:#?}", res);
        Decimal::new(0, 0)
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
        todo!()
    }

    async fn get_balance(&self) -> Decimal {
        todo!()
    }

    fn get_hledger_account(&self) -> &str {
        todo!()
    }
}

pub fn deserialize_record<'a, T>(
    record: &'a StringRecord,
    headers: &'a StringRecord,
    target_vec: &mut Vec<T>,
) where
    T: Deserialize<'a>,
{
    if record[2].to_ascii_lowercase().contains("total") {
        // Some rows are totals, which we should skip
        return;
    }
    match record.deserialize::<'a, T>(Some(headers)) {
        Ok(row) => target_vec.push(row),
        Err(e) => error!("{:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use serde_xml_rs::from_reader;

    use crate::ib::FlexStatementRequestResponse;

    use super::Ib;

    #[test]
    #[ignore]
    fn example() {
        let report = Ib::read_report();
        println!("{:#?}", report);
    }

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
    async fn get_balance() {
        dotenv::dotenv().ok();
        Ib::get_statement().await;
    }
}
