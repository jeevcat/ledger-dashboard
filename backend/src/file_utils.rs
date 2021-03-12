use std::{env, path::PathBuf};

use log::info;

use crate::config;

pub fn get_imported_ledger_file() -> Option<PathBuf> {
    Some(get_journal_path()?.join("imported.ledger"))
}

pub fn get_ledger_year_file(year: i32) -> Option<PathBuf> {
    Some(get_journal_path()?.join(match year {
        2016 => "2016.ledger",
        2017 => "2017.ledger",
        2018 => "2018.ledger",
        2019 => "2019.ledger",
        2020 => "2020/autofilled.ledger",
        2021 => "2021.ledger",
        _ => return None,
    }))
}

pub fn get_ledger_year_files() -> Vec<(i32, PathBuf)> {
    (2016..=2021)
        .filter_map(|year| get_ledger_year_file(year).map(|f| (year, f)))
        .collect()
}

pub fn get_prices_file() -> Option<PathBuf> {
    Some(get_journal_path()?.join("prices.ledger"))
}

pub fn get_database_file(filename: &str) -> Option<PathBuf> {
    Some(get_database_path()?.join(filename))
}

fn get_backend_path() -> Option<PathBuf> {
    // Support for running inside cargo directory structure
    if let Some(cargo_project_root) = option_env!("CARGO_MANIFEST_DIR") {
        info!("We in running inside cargo: {}", cargo_project_root);
        return Some(PathBuf::from(cargo_project_root));
    }
    // Default to exe directory
    Some(env::current_exe().ok()?.parent()?.to_path_buf())
}

fn get_base_path() -> Option<PathBuf> {
    get_backend_path()?
        .parent()?
        .parent()
        .map(|p| p.to_path_buf())
}

fn get_journal_path() -> Option<PathBuf> {
    Some(match config::journal_path() {
        Some(path) => PathBuf::from(path),
        None => get_base_path()?.join("journal"),
    })
}

fn get_database_path() -> Option<PathBuf> {
    match config::database_path() {
        Some(path) => Some(PathBuf::from(path)),
        None => get_backend_path(),
    }
}
