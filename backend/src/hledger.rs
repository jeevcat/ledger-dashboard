use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Mutex,
    },
    time,
};

use log::{info, warn};

use crate::{
    api::transactions::TransactionCollection, file_utils::get_imported_ledger_file,
    model::recorded_transaction::RecordedTransaction,
};

const CONTENT_TYPE: &str = "Content-Type";
const CONTENT_TYPE_JSON: &str = "application/json";
const PORT: &str = "5001";
const BASE_URL: &str = "http://127.0.0.1";

pub struct Hledger {
    cache: Mutex<HashMap<String, TransactionCollection>>,
    cache_valid: AtomicBool,
    http_client: reqwest::Client,
    process: Mutex<Child>,
    process_ready: AtomicBool,
}

impl Hledger {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            cache_valid: AtomicBool::new(false),
            http_client: reqwest::Client::new(),
            process: Mutex::new(Self::spawn_hledger_process()),
            process_ready: AtomicBool::new(true),
        }
    }

    /// Leave the json as a string as we just pass it back to our own API
    pub async fn get_accounts(&self) -> String {
        self.wait_for_hledger_process();

        let request_url = format!("{}:{}/accountnames", BASE_URL, PORT);
        reqwest::get(request_url.as_str())
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    }

    pub async fn get_commodities(&self) -> Vec<String> {
        self.wait_for_hledger_process();

        let request_url = format!("{}:{}/commodities", BASE_URL, PORT);
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

    pub async fn get_transactions(&self, account_names: &[&str]) -> TransactionCollection {
        // Early return cached transactions
        let cache_key = account_names.join("-");
        if self.is_cache_valid() {
            info!("hledger using cache!");
            return self.get_cached_transactions(&cache_key);
        }

        self.wait_for_hledger_process();

        // Fetch transactions from hledger-web API
        let request_url = format!("{}:{}/transactions", BASE_URL, PORT);
        let response = reqwest::get(request_url.as_str()).await.unwrap();
        let all: Vec<RecordedTransaction> = response.json().await.unwrap();

        // Filter transactions by given account name
        let transactions: TransactionCollection = all
            .into_iter()
            .rev()
            .filter(|t| {
                t.tpostings
                    .iter()
                    .any(|p| account_names.iter().any(|n| p.paccount.as_str() == *n))
            })
            .collect();

        // Write to cache
        self.cache_transactions(&cache_key, &transactions);

        transactions
    }

    // TODO: proper errors
    pub async fn write_single_transaction(&self, recorded: &RecordedTransaction) -> bool {
        self.wait_for_hledger_process();

        self.invalidate_cache();

        if !self.write_transaction(recorded).await {
            return false;
        }

        self.restart_hledger();

        true
    }

    // TODO: proper errors
    pub async fn write_transactions(&self, recorded: &[RecordedTransaction]) -> bool {
        self.wait_for_hledger_process();

        self.invalidate_cache();
        for t in recorded {
            if !self.write_transaction(t).await {
                warn!("Couldn't write transacation. Restarting hledger...");
                // Restart hledger and try again
                self.restart_hledger();
                if !self.write_transaction(t).await {
                    return false;
                }
            }
        }
        self.restart_hledger();

        true
    }

    pub async fn write_transaction(&self, recorded: &RecordedTransaction) -> bool {
        let json = serde_json::to_string(recorded).unwrap();
        let request_url = format!("{}:{}/add", BASE_URL, PORT);
        let response = match self
            .http_client
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
            println!("{}", serde_json::to_string_pretty(recorded).unwrap());
            println!("{:#?}", response);
            println!("{}", response.text().await.unwrap());
            return false;
        }
        true
    }

    fn restart_hledger(&self) {
        self.process_ready.store(false, Relaxed);
        let mut process = self.process.lock().unwrap();
        info!("killing hledger-web...");
        process
            .kill()
            .expect("Couldn't kill hledger-web as it wasn't running");
        info!("Waiting for hledger-web to close...");
        let exit_code = process
            .wait()
            .expect("Couldn't wait hledger-web as it wasn't running");
        info!("hledger-web closed with exit code: {}", exit_code);
        *process = Self::spawn_hledger_process();
        self.process_ready.store(true, Relaxed);
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

    fn wait_for_hledger_process(&self) {
        while !self.process_ready.load(Relaxed) {
            info!("Waiting for hledger-api process...")
        }
    }

    fn spawn_hledger_process() -> Child {
        let journal_file = get_imported_ledger_file().unwrap();
        info!("Path of ledger journal file is: {}", journal_file.display());
        info!("Starting hledger-web...");
        let mut process = Command::new("hledger-web")
            .arg("--serve-api")
            .arg("--port")
            .arg(PORT)
            .arg("-f")
            .arg(journal_file)
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
        process
    }
}
