use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Mutex,
    },
    time,
};

use chrono::{Datelike, NaiveDate};
use log::{error, info, warn};
use rust_decimal::{prelude::ToPrimitive, Decimal};

use crate::{
    file_utils::{get_default_ledger_file, get_ledger_year_files},
    model::{aligned_data::AlignedData, hledger_transaction::HledgerTransaction},
};

const CONTENT_TYPE: &str = "Content-Type";
const CONTENT_TYPE_JSON: &str = "application/json";
const READ_PORT: i32 = 5001;
const BASE_URL: &str = "http://127.0.0.1";
const DATE_FMT: &str = "%Y-%m-%d";

pub struct HledgerProcess {
    journal_file: PathBuf,
    process: Mutex<Option<Child>>,
    ready: AtomicBool,
    port: i32,
}

impl HledgerProcess {
    fn new(journal_file: &Path, port: i32) -> Self {
        let h = Self {
            journal_file: journal_file.to_path_buf(),
            ready: AtomicBool::new(false),
            port,
            process: Mutex::new(None),
        };
        h.spawn_process();
        h
    }

    /// Leave the json as a string as we just pass it back to our own API
    async fn get_accounts(&self) -> String {
        self.wait_for_hledger_process();

        let request_url = format!("{}:{}/accountnames", BASE_URL, self.port);
        reqwest::get(request_url.as_str())
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    }

    async fn get_commodities(&self) -> Vec<String> {
        self.wait_for_hledger_process();

        let request_url = format!("{}:{}/commodities", BASE_URL, self.port);
        let commodities = reqwest::get(request_url.as_str())
            .await
            .unwrap()
            .json::<Vec<String>>()
            .await
            .unwrap();

        commodities
            .into_iter()
            .filter(|c| c != "AUTO" && !c.contains(' '))
            .collect()
    }

    async fn fetch_transactions(&self, account_names: &[&str]) -> Vec<HledgerTransaction> {
        self.wait_for_hledger_process();

        // Fetch transactions from hledger-web API
        let request_url = format!("{}:{}/transactions", BASE_URL, self.port);
        let response = reqwest::get(request_url.as_str()).await.unwrap();
        let all: Vec<HledgerTransaction> = response.json().await.unwrap();

        // Filter transactions by given account name
        all.into_iter()
            .rev()
            .filter(|t| {
                !t.tdescription.contains("opening balances")
                    && !t.tdescription.contains("closing balances")
                    && t.tpostings
                        .iter()
                        .any(|p| account_names.contains(&p.paccount.as_str()))
            })
            .collect()
    }

    async fn write_transaction(
        &self,
        http_client: &reqwest::Client,
        recorded: &HledgerTransaction,
    ) -> bool {
        let json = serde_json::to_string(recorded).unwrap();
        let request_url = format!("{}:{}/add", BASE_URL, self.port);
        info!(
            "Writing transaction ({}) to hledger file {:#?} using url {}",
            recorded.tdescription, self.journal_file, &request_url
        );
        let response = match http_client
            .put(request_url.as_str())
            .header(CONTENT_TYPE, CONTENT_TYPE_JSON)
            .body(json)
            .timeout(time::Duration::from_secs(2))
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => return false,
        };

        if response.status().is_success() {
            info!("{:#?}", response);
        } else {
            error!("{}", serde_json::to_string_pretty(recorded).unwrap());
            error!("{:#?}", response);
            error!("{}", response.text().await.unwrap());
            return false;
        }
        true
    }

    fn wait_for_hledger_process(&self) {
        while !self.ready.load(Relaxed) {
            info!(
                "Waiting for hledger-api process for {:#?} on {}...",
                self.journal_file, self.port
            )
        }
    }

    fn spawn_process(&self) {
        let mut process = Command::new("hledger-web")
            .arg("--serve-api")
            .arg("--port")
            .arg(self.port.to_string())
            .arg("-f")
            .arg(&self.journal_file)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Couldn't start hledger command");
        let output = process
            .stdout
            .as_mut()
            .expect("Couldn't capture hledger-web stdout");
        let mut reader = BufReader::new(output);
        let mut line = String::new();
        while !line.contains("Press ctrl-c to quit") {
            line.clear();
            reader.read_line(&mut line).unwrap();
        }
        info!(
            "hledger-web successfully launched for {:#?} at {}:{} with PID {}",
            self.journal_file,
            BASE_URL,
            self.port,
            process.id(),
        );
        self.ready.store(true, Relaxed);
        *self.process.lock().unwrap() = Some(process)
    }

    fn restart_hledger(&self) {
        self.ready.store(false, Relaxed);
        if let Some(process) = &mut *self.process.lock().unwrap() {
            info!("killing hledger-web {:#?}...", self.journal_file);
            process
                .kill()
                .expect("Couldn't kill hledger-web as it wasn't running");
            info!("Waiting for hledger-web to close...");
            let exit_code = process
                .wait()
                .expect("Couldn't wait hledger-web as it wasn't running");
            info!("hledger-web closed with exit code: {}", exit_code);
        }
        self.spawn_process();
    }
}

pub struct Hledger {
    http_client: reqwest::Client,
    read_process: HledgerProcess,
    write_processes: HashMap<i32, HledgerProcess>,
}

impl Hledger {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            read_process: HledgerProcess::new(&get_default_ledger_file(), READ_PORT),
            write_processes: get_ledger_year_files()
                .into_iter()
                .map(|(y, f)| (y, HledgerProcess::new(&f, y + READ_PORT - 2000)))
                .collect(),
        }
    }

    pub async fn get_accounts(&self) -> String {
        self.read_process.get_accounts().await
    }

    pub async fn get_commodities(&self) -> Vec<String> {
        self.read_process.get_commodities().await
    }

    pub async fn fetch_transactions(&self, account_names: &[&str]) -> Vec<HledgerTransaction> {
        self.read_process.fetch_transactions(account_names).await
    }

    // TODO: proper errors
    pub async fn write_single_transaction(&self, recorded: &HledgerTransaction) -> bool {
        if let Some(process) = self.write_processes.get(&recorded.get_date(None).year()) {
            process.wait_for_hledger_process();

            if !process.write_transaction(&self.http_client, recorded).await {
                return false;
            }

            process.restart_hledger();

            return true;
        }
        false
    }

    // TODO: proper errors
    pub async fn write_transactions(&self, recorded: &[HledgerTransaction]) -> bool {
        for t in recorded {
            let year = &t.get_date(None).year();
            if let Some(process) = self.write_processes.get(year) {
                process.wait_for_hledger_process();
                if !process.write_transaction(&self.http_client, t).await {
                    warn!("Couldn't write transacation. Restarting hledger...");
                    // Restart hledger and try again
                    process.restart_hledger();
                    if !process.write_transaction(&self.http_client, t).await {
                        return false;
                    }
                }
            } else {
                error!(
                    "Couldn't find hledger process for year {} in {}",
                    year,
                    self.write_processes
                        .keys()
                        .map(|y: &i32| y.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        self.read_process.restart_hledger();

        true
    }

    pub async fn get_account_balance(&self, account: &str) -> Option<Decimal> {
        let command = "bal";
        let account_arg = format!("^{}$", account); // Ensure we only get exact account matches
        let args = &[account_arg.as_str()];
        let stdout = self.hledger_csv_command(command, args).await;
        get_total_from_csv(stdout)
    }

    pub async fn get_income_statement(&self) -> AlignedData {
        let command = "is";
        let args = &["--monthly", "--depth", "1"];
        let stdout = self.hledger_csv_command(command, args).await;

        get_income_statement_from_csv(stdout)
    }

    async fn hledger_csv_command(&self, command: &str, args: &[&str]) -> impl std::io::Read {
        let stdout = Command::new("hledger")
            .arg(command)
            .arg("-V")
            .arg("--output-format")
            .arg("csv")
            .arg("-f")
            .arg(get_default_ledger_file())
            .args(args)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Couldn't start hledger command")
            .stdout
            .unwrap();
        stdout
    }
}

fn get_total_from_csv(reader: impl std::io::Read) -> Option<Decimal> {
    let mut reader = csv::Reader::from_reader(reader);
    for result in reader.records() {
        if let Ok(record) = result {
            if let Some(account) = record.get(0) {
                if account != "total" {
                    continue;
                }
                if let Some(total) = record.get(1) {
                    return currency_amount_to_decimal(total);
                }
            }
        }
    }
    None
}

fn get_income_statement_from_csv(reader: impl std::io::Read) -> AlignedData {
    enum ParseState {
        Description,
        Months,
        Revenues,
        Expenses,
        Done,
    }

    let mut parse_state = ParseState::Description;
    let mut start_date: NaiveDate = NaiveDate::from_ymd(1, 1, 1);
    let mut dates: Vec<NaiveDate> = vec![];
    let mut revenues: Vec<Decimal> = vec![];
    let mut expenses: Vec<Decimal> = vec![];

    fn record_prices(record: csv::StringRecord) -> Vec<Decimal> {
        record
            .iter()
            .skip(1)
            .map(|amount| {
                assert!(amount == "0" || amount.contains("EUR"));
                currency_amount_to_decimal(amount).unwrap()
            })
            .collect()
    }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(reader);
    for result in reader.records() {
        if let Ok(record) = result {
            match parse_state {
                ParseState::Description => {
                    if let Some(description) = record.get(0) {
                        if description.starts_with("Income Statement") {
                            let date_range = description.split(' ').nth(2).unwrap();
                            let mut split = date_range.split("..");

                            let start = split.next().unwrap();
                            start_date = NaiveDate::parse_from_str(start, DATE_FMT).unwrap();
                            parse_state = ParseState::Months;
                        }
                    }
                }
                ParseState::Months => {
                    if let Some(title) = record.get(0) {
                        if title == "Account" {
                            let mut date = last_day_of_month(start_date);
                            for _ in record.iter().skip(1) {
                                dates.push(date);
                                date = last_day_of_next_month(date);
                            }
                            parse_state = ParseState::Revenues;
                        }
                    }
                }
                ParseState::Revenues => {
                    if let Some(title) = record.get(0) {
                        if title == "Total:" {
                            revenues = record_prices(record);
                            assert_eq!(dates.len(), revenues.len());
                            parse_state = ParseState::Expenses;
                        }
                    }
                }
                ParseState::Expenses => {
                    if let Some(title) = record.get(0) {
                        if title == "Total:" {
                            expenses = record_prices(record);
                            assert_eq!(dates.len(), expenses.len());
                            parse_state = ParseState::Done;
                        }
                    }
                }
                ParseState::Done => {
                    break;
                }
            }
        }
    }
    let decimal_to_number =
        |v: &Decimal| serde_json::Number::from_f64(v.to_f64().unwrap()).unwrap();
    AlignedData {
        x_values: dates
            .iter()
            .map(|d| d.and_hms(0, 0, 0).timestamp().into())
            .collect(),
        y_values: vec![
            revenues.iter().map(decimal_to_number).collect(),
            expenses.iter().map(decimal_to_number).collect(),
        ],
    }
}

fn last_day_of_month(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd(date.year() + 1, 1, 1))
        .pred()
}

fn last_day_of_next_month(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month() + 2, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd(date.year() + 1, date.month() - 10, 1))
        .pred()
}

fn currency_amount_to_decimal(amount: &str) -> Option<Decimal> {
    if let Some(amount) = amount.split_ascii_whitespace().next() {
        let amount = amount.replace(",", "");
        return Decimal::from_str(&amount).ok();
    }
    None
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime};
    use rust_decimal::{prelude::FromPrimitive, Decimal};

    use super::{
        currency_amount_to_decimal, get_income_statement_from_csv, get_total_from_csv,
        last_day_of_next_month,
    };

    #[test]
    fn currency_convert_simple() {
        assert_eq!(
            currency_amount_to_decimal("100 EUR").unwrap(),
            Decimal::from_f32(100.).unwrap()
        )
    }

    #[test]
    fn currency_convert_cents() {
        assert_eq!(
            currency_amount_to_decimal("123.05 EUR").unwrap(),
            Decimal::from_f32(123.05).unwrap()
        )
    }

    #[test]
    fn currency_convert_comma() {
        assert_eq!(
            currency_amount_to_decimal("-5,230.99 EUR").unwrap(),
            Decimal::from_f32(-5230.99).unwrap()
        )
    }

    #[test]
    fn total_from_csv() {
        let data = r#"
"account","balance"
"Assets:Cash:N26","-3,490.81 EUR"
"total","-3,490.81 EUR"
"#;
        assert_eq!(
            get_total_from_csv(data.as_bytes()),
            Decimal::from_f32(-3490.81)
        )
    }

    #[test]
    fn next_month() {
        assert_eq!(
            last_day_of_next_month(NaiveDate::from_ymd(2000, 2, 1)),
            NaiveDate::from_ymd(2000, 3, 31)
        );
        assert_eq!(
            last_day_of_next_month(NaiveDate::from_ymd(2000, 3, 31)),
            NaiveDate::from_ymd(2000, 4, 30)
        );
        assert_eq!(
            last_day_of_next_month(NaiveDate::from_ymd(2000, 11, 30)),
            NaiveDate::from_ymd(2000, 12, 31)
        );
        assert_eq!(
            last_day_of_next_month(NaiveDate::from_ymd(2000, 12, 1)),
            NaiveDate::from_ymd(2001, 1, 31)
        );
    }

    #[test]
    fn income_statement() {
        let data = r#"
"Income Statement 2016-05-01..2021-04-30, valued at period ends","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""
"Account","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec","Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec","Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec","Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec","Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec","Jan","Feb","Mar","Apr"
"Revenues","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""
"Income","25.91 EUR","305.37 EUR","4,129.35 EUR","3,080.55 EUR","4,204.66 EUR","2,737.13 EUR","3,333.65 EUR","560.78 EUR","4,931.64 EUR","4,677.54 EUR","2,990.86 EUR","5,004.71 EUR","5,240.15 EUR","4,801.33 EUR","7,064.53 EUR","3,710.17 EUR","1.86 EUR","6,295.03 EUR","5,350.02 EUR","8,857.10 EUR","4,931.82 EUR","5,333.69 EUR","4,001.27 EUR","4,002.12 EUR","4,137.05 EUR","4,144.81 EUR","4,001.30 EUR","8,447.70 EUR","4,760.57 EUR","6,691.28 EUR","2,585.82 EUR","6,474.17 EUR","5,397.55 EUR","5,021.21 EUR","32,540.74 EUR","930.76 EUR","5,632.21 EUR","12,040.00 EUR","4,500.00 EUR","5,840.00 EUR","0","5,360.00 EUR","14,575.00 EUR","10,706.95 EUR","4,510.44 EUR","4,675.44 EUR","6,577.50 EUR","5,199.46 EUR","5,199.46 EUR","5,199.46 EUR","3,228.31 EUR","5,728.31 EUR","12,618.21 EUR","5,074.98 EUR","5,272.06 EUR","5,712.34 EUR","26,874.86 EUR","6,344.86 EUR","7,806.13 EUR","0"
"Total:","25.91 EUR","305.37 EUR","4,129.35 EUR","3,080.55 EUR","4,204.66 EUR","2,737.13 EUR","3,333.65 EUR","560.78 EUR","4,931.64 EUR","4,677.54 EUR","2,990.86 EUR","5,004.71 EUR","5,240.15 EUR","4,801.33 EUR","7,064.53 EUR","3,710.17 EUR","1.86 EUR","6,295.03 EUR","5,350.02 EUR","8,857.10 EUR","4,931.82 EUR","5,333.69 EUR","4,001.27 EUR","4,002.12 EUR","4,137.05 EUR","4,144.81 EUR","4,001.30 EUR","8,447.70 EUR","4,760.57 EUR","6,691.28 EUR","2,585.82 EUR","6,474.17 EUR","5,397.55 EUR","5,021.21 EUR","32,540.74 EUR","930.76 EUR","5,632.21 EUR","12,040.00 EUR","4,500.00 EUR","5,840.00 EUR","0","5,360.00 EUR","14,575.00 EUR","10,706.95 EUR","4,510.44 EUR","4,675.44 EUR","6,577.50 EUR","5,199.46 EUR","5,199.46 EUR","5,199.46 EUR","3,228.31 EUR","5,728.31 EUR","12,618.21 EUR","5,074.98 EUR","5,272.06 EUR","5,712.34 EUR","26,874.86 EUR","6,344.86 EUR","7,806.13 EUR","0"
"Expenses","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""
"Expenses","0","498.69 EUR","1,523.51 EUR","2,969.29 EUR","4,413.99 EUR","3,276.31 EUR","1,294.94 EUR","1,638.35 EUR","1,585.99 EUR","2,269.89 EUR","1,894.67 EUR","1,948.98 EUR","1,330.70 EUR","8,742.17 EUR","2,249.22 EUR","2,335.52 EUR","1,676.18 EUR","1,536.85 EUR","2,802.16 EUR","3,770.63 EUR","2,041.65 EUR","2,248.31 EUR","1,972.25 EUR","2,118.01 EUR","4,408.57 EUR","2,549.15 EUR","1,632.98 EUR","1,447.28 EUR","1,532.70 EUR","2,556.69 EUR","5,494.46 EUR","5,133.41 EUR","2,849.54 EUR","3,687.17 EUR","3,792.58 EUR","10,526.35 EUR","2,783.59 EUR","4,730.28 EUR","3,228.42 EUR","2,452.95 EUR","3,041.50 EUR","3,786.23 EUR","15,201.55 EUR","6,263.17 EUR","4,371.43 EUR","3,048.34 EUR","2,833.89 EUR","4,780.26 EUR","4,277.65 EUR","2,644.22 EUR","21,581.48 EUR","4,247.29 EUR","4,024.44 EUR","4,682.87 EUR","15,472.40 EUR","5,579.70 EUR","3,671.48 EUR","2,957.47 EUR","326.27 EUR","2,688.94 EUR"
"Total:","0","498.69 EUR","1,523.51 EUR","2,969.29 EUR","4,413.99 EUR","3,276.31 EUR","1,294.94 EUR","1,638.35 EUR","1,585.99 EUR","2,269.89 EUR","1,894.67 EUR","1,948.98 EUR","1,330.70 EUR","8,742.17 EUR","2,249.22 EUR","2,335.52 EUR","1,676.18 EUR","1,536.85 EUR","2,802.16 EUR","3,770.63 EUR","2,041.65 EUR","2,248.31 EUR","1,972.25 EUR","2,118.01 EUR","4,408.57 EUR","2,549.15 EUR","1,632.98 EUR","1,447.28 EUR","1,532.70 EUR","2,556.69 EUR","5,494.46 EUR","5,133.41 EUR","2,849.54 EUR","3,687.17 EUR","3,792.58 EUR","10,526.35 EUR","2,783.59 EUR","4,730.28 EUR","3,228.42 EUR","2,452.95 EUR","3,041.50 EUR","3,786.23 EUR","15,201.55 EUR","6,263.17 EUR","4,371.43 EUR","3,048.34 EUR","2,833.89 EUR","4,780.26 EUR","4,277.65 EUR","2,644.22 EUR","21,581.48 EUR","4,247.29 EUR","4,024.44 EUR","4,682.87 EUR","15,472.40 EUR","5,579.70 EUR","3,671.48 EUR","2,957.47 EUR","326.27 EUR","2,688.94 EUR"
"Net:","25.91 EUR","-193.32 EUR","2,605.83 EUR","111.26 EUR","-209.32 EUR","-539.18 EUR","2,038.71 EUR","-1,077.57 EUR","3,345.65 EUR","2,407.65 EUR","1,096.19 EUR","3,055.73 EUR","3,909.46 EUR","-3,940.84 EUR","4,815.31 EUR","1,374.64 EUR","-1,674.31 EUR","4,758.19 EUR","2,547.86 EUR","5,086.47 EUR","2,890.17 EUR","3,085.38 EUR","2,029.02 EUR","1,884.10 EUR","-271.52 EUR","1,595.66 EUR","2,368.32 EUR","7,000.42 EUR","3,227.87 EUR","4,134.59 EUR","-2,908.64 EUR","1,340.76 EUR","2,548.01 EUR","1,334.04 EUR","28,748.16 EUR","-9,595.59 EUR","2,848.62 EUR","7,309.72 EUR","1,271.58 EUR","3,387.05 EUR","-3,041.50 EUR","1,573.77 EUR","-626.55 EUR","4,443.78 EUR","139.01 EUR","1,627.10 EUR","3,743.61 EUR","419.20 EUR","921.81 EUR","2,555.24 EUR","-18,353.17 EUR","1,481.02 EUR","8,593.77 EUR","392.11 EUR","-10,200.34 EUR","132.64 EUR","23,203.38 EUR","3,387.39 EUR","7,479.86 EUR","-2,688.94 EUR"
"#;
        let is = get_income_statement_from_csv(data.as_bytes());
        println!("{:#?}", is);
        fn to_date(n: &serde_json::Number) -> NaiveDate {
            NaiveDateTime::from_timestamp(n.as_i64().unwrap(), 0).date()
        }
        assert_eq!(
            to_date(is.x_values.first().unwrap()),
            NaiveDate::from_ymd(2016, 5, 31)
        );
        assert_eq!(
            to_date(is.x_values.last().unwrap()),
            NaiveDate::from_ymd(2021, 4, 30)
        );
        assert_eq!(
            is.y_values[0][0],
            serde_json::Number::from_f64(25.91).unwrap()
        );
        assert_eq!(
            is.y_values[0][1],
            serde_json::Number::from_f64(305.37).unwrap()
        );
        assert_eq!(is.y_values[1][0], serde_json::Number::from_f64(0.).unwrap());
        assert_eq!(
            is.y_values[1][1],
            serde_json::Number::from_f64(498.69).unwrap()
        );
        assert_eq!(
            is.y_values[0].last().unwrap(),
            &serde_json::Number::from_f64(0.).unwrap()
        );
        assert_eq!(
            is.y_values[1].last().unwrap(),
            &serde_json::Number::from_f64(2688.94).unwrap()
        );
    }
}
