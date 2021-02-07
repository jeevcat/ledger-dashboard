use std::collections::HashMap;

use crate::{
    model::{
        real_transaction::RealTransaction, recorded_transaction::RecordedTransaction, rule::Rule,
        transaction_response::TransactionResponse,
    },
    templater::Templater,
};

pub fn get_existing_transactions<'a, I, J, K>(
    recorded_transactions: I,
    real_transactions: J,
) -> Vec<TransactionResponse>
where
    I: IntoIterator<Item = &'a RecordedTransaction>,
    J: IntoIterator<Item = K>,
    K: RealTransaction,
{
    let real_transactions: HashMap<_, _> = real_transactions
        .into_iter()
        .map(move |t| (t.get_id().to_string(), t))
        .collect();
    recorded_transactions
        .into_iter()
        .flat_map(|rec: &RecordedTransaction| {
            rec.ids()
                .filter_map(|id| real_transactions.get(id))
                .map(move |real| TransactionResponse {
                    real_transaction: real.to_json_value(),
                    recorded_transaction: Some(rec.to_owned()),
                    rule: None,
                })
        })
        .collect()
}

pub fn get_generated_transactions<'a, RecIter, ReaIter, RulIter, Rea>(
    recorded_transactions: RecIter,
    real_transactions: ReaIter,
    rules: RulIter,
    templater: &'a Templater,
) -> Vec<TransactionResponse>
where
    RecIter: IntoIterator<Item = &'a RecordedTransaction>,
    RulIter: IntoIterator<Item = &'a Rule>,
    ReaIter: IntoIterator<Item = &'a Rea>,
    Rea: RealTransaction,
    Rea: 'a,
{
    let mut recorded_iter = recorded_transactions.into_iter();
    let mut rules_iter = rules.into_iter();
    real_transactions
        .into_iter()
        // Only real transactions which haven't already been recorded
        // TODO Slow?
        .filter(|real| {
            !recorded_iter
                .any(|rec: &RecordedTransaction| rec.ids().any(|h_id| h_id == real.get_id()))
        })
        // Apply any matching rules to the real transactions
        .filter_map(|real| {
            rules_iter.find_map(|rule| {
                rule.apply(templater, real).map(|gen| TransactionResponse {
                    real_transaction: real.to_json_value(),
                    recorded_transaction: Some(gen),
                    rule: Some(rule.to_owned()),
                })
            })
        })
        .collect()
}
