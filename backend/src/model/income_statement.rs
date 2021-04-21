use serde::Serialize;

use super::{aligned_data::AlignedData, hledger_transaction::HledgerTransaction};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeStatementResponse {
    pub data: AlignedData,
    pub top_revenues: Vec<Vec<HledgerTransaction>>,
    pub top_expenses: Vec<Vec<HledgerTransaction>>,
}
