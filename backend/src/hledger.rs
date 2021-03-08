use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Mutex,
    },
    time,
};

use chrono::Datelike;
use log::{error, info, warn};

use crate::{
    api::transactions::TransactionCollection,
    file_utils::{get_imported_ledger_file, get_ledger_year_files},
    model::recorded_transaction::RecordedTransaction,
};

const CONTENT_TYPE: &str = "Content-Type";
const CONTENT_TYPE_JSON: &str = "application/json";
const READ_PORT: i32 = 5001;
const BASE_URL: &str = "http://127.0.0.1";

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

    async fn get_transactions(&self, account_names: &[&str]) -> TransactionCollection {
        self.wait_for_hledger_process();

        // Fetch transactions from hledger-web API
        let request_url = format!("{}:{}/transactions", BASE_URL, self.port);
        let response = reqwest::get(request_url.as_str()).await.unwrap();
        let all: Vec<RecordedTransaction> = response.json().await.unwrap();

        // Filter transactions by given account name
        all.into_iter()
            .rev()
            .filter(|t| {
                t.tpostings
                    .iter()
                    .any(|p| account_names.iter().any(|n| p.paccount.as_str() == *n))
            })
            .collect()
    }

    async fn write_transaction(
        &self,
        http_client: &reqwest::Client,
        recorded: &RecordedTransaction,
    ) -> bool {
        let json = serde_json::to_string(recorded).unwrap();
        let request_url = format!("{}:{}/add", BASE_URL, self.port);
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

        if !response.status().is_success() {
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
        info!(
            "Path of ledger journal file is: {}",
            self.journal_file.display()
        );
        info!("Starting hledger-web...");
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
        info!("hledger-web successfully launched!");
        self.ready.store(true, Relaxed);
        *self.process.lock().unwrap() = Some(process)
    }

    fn restart_hledger(&self) {
        self.ready.store(false, Relaxed);
        if let Some(process) = &mut *self.process.lock().unwrap() {
            info!("killing hledger-web...");
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
    cache: Mutex<HashMap<String, TransactionCollection>>,
    cache_valid: AtomicBool,
    http_client: reqwest::Client,
    read_process: HledgerProcess,
    write_processes: HashMap<i32, HledgerProcess>,
}

impl Hledger {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            cache_valid: AtomicBool::new(false),
            http_client: reqwest::Client::new(),
            read_process: HledgerProcess::new(&get_imported_ledger_file().unwrap(), READ_PORT),
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

    pub async fn get_transactions(&self, account_names: &[&str]) -> TransactionCollection {
        // Early return cached transactions
        let cache_key = account_names.join("-");
        if self.is_cache_valid() {
            info!("hledger using cache!");
            return self.get_cached_transactions(&cache_key);
        }

        let transactions = self.read_process.get_transactions(account_names).await;

        // Write to cache
        self.cache_transactions(&cache_key, &transactions);

        transactions
    }

    // TODO: proper errors
    pub async fn write_single_transaction(&self, recorded: &RecordedTransaction) -> bool {
        if let Some(process) = self.write_processes.get(&recorded.tdate.year()) {
            process.wait_for_hledger_process();

            self.invalidate_cache();

            if !process.write_transaction(&self.http_client, recorded).await {
                return false;
            }

            process.restart_hledger();

            return true;
        }
        false
    }

    // TODO: proper errors
    pub async fn write_transactions(&self, recorded: &[RecordedTransaction]) -> bool {
        self.invalidate_cache();
        for t in recorded {
            if let Some(process) = self.write_processes.get(&t.tdate.year()) {
                process.wait_for_hledger_process();
                if !process.write_transaction(&self.http_client, t).await {
                    warn!("Couldn't write transacation. Restarting hledger...");
                    // Restart hledger and try again
                    process.restart_hledger();
                    if !process.write_transaction(&self.http_client, t).await {
                        return false;
                    }
                }
            }
        }
        self.read_process.restart_hledger();

        true
    }

    // Cache

    fn is_cache_valid(&self) -> bool {
        self.cache_valid.load(Relaxed)
    }

    fn get_cached_transactions(&self, cache_key: &str) -> TransactionCollection {
        self.cache.lock().unwrap()[cache_key].clone()
    }

    fn cache_transactions(&self, cache_key: &str, transactions: &[RecordedTransaction]) {
        self.cache_valid.store(true, Relaxed);
        self.cache
            .lock()
            .unwrap()
            .insert(cache_key.to_string(), transactions.into());
    }

    fn invalidate_cache(&self) {
        self.cache_valid.store(false, Relaxed)
    }
}
