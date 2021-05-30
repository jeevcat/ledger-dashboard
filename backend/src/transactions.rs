use std::collections::{HashMap, HashSet};

use rust_decimal::Decimal;

use crate::{
    hledger::Hledger,
    import_account::ImportAccount,
    model::{
        hledger_transaction::HledgerTransaction,
        real_transaction::RealTransaction,
        rule::Rule,
        transaction_response::{ExistingTransactionResponse, TransactionResponse},
    },
    templater::Templater,
};

pub async fn get_existing_transactions<J, K>(
    import_account: &impl ImportAccount,
    hledger: &Hledger,
    real_transactions: J,
) -> Vec<ExistingTransactionResponse>
where
    J: IntoIterator<Item = K>,
    K: RealTransaction,
{
    let real_transactions: HashMap<_, _> = real_transactions
        .into_iter()
        .map(move |t| (t.get_id().to_string(), t))
        .collect();

    let import_hledger_account = import_account.get_hledger_account();

    // TODO fix this account array stuff
    let import_hledger_accounts = &[import_hledger_account];
    let hledger_transactions = hledger
        .fetch_account_transactions(import_hledger_accounts)
        .await;

    // Collect unique ids so we can check for duplicates
    let mut distinct_recorded_ids = HashMap::<&str, u8>::new();
    for t in &hledger_transactions {
        for id in t.get_all_ids(import_hledger_account) {
            let counter = distinct_recorded_ids.entry(id).or_insert(0);
            *counter += 1;
        }
    }

    let balance = hledger
        .get_account_balance(import_hledger_account)
        .await
        .unwrap_or_default();

    // TODO: Remove need for this clone?
    let mut hledger_transactions = hledger_transactions.clone();
    hledger_transactions.sort_by_key(|t| (t.get_date(Some(import_hledger_account))));
    hledger_transactions
        .iter()
        .rev()
        .flat_map(|h| {
            let ids: Vec<&str> = h.get_all_ids(import_hledger_account).collect();
            if ids.is_empty() {
                return vec![(h, None)];
            } else {
                ids.into_iter().map(|id| (h, Some(id))).collect::<Vec<_>>()
            }
        })
        .scan((balance, balance), |(h_sum, r_sum), (h, id)| {
            // This unwrap is safe. We can be sure that there will always be an amount.
            let h_amount = h.get_amount(id, import_hledger_account).unwrap();
            *h_sum -= h_amount;

            let mut real = None;
            if let Some(id) = id {
                real = real_transactions.get(id);
                if let Some(real) = real {
                    if let Some(amount) =
                        real.get_field::<Decimal>(real.get_default_amount_field_name())
                    {
                        *r_sum -= amount;
                    }
                }
            }
            Some(((h, real), (*h_sum, *r_sum)))
        })
        .map(|((h, r), (recorded_cumulative, real_cumulative))| {
            let real_json = r.map_or(serde_json::Value::Null, |real| real.to_json_value());
            ExistingTransactionResponse {
                real_transaction: real_json,
                hledger_transaction: h.to_owned(),
                real_cumulative,
                hledger_cumulative: recorded_cumulative,
                errors: get_errors(import_account, &distinct_recorded_ids, &r, &h),
            }
        })
        .collect()
}

pub fn get_generated_transactions(
    hledger_account: &str,
    hledger_transactions: &[HledgerTransaction],
    real_transactions: &[impl RealTransaction],
    rules: &[Rule],
) -> Vec<TransactionResponse> {
    let templater = Templater::from_rules(rules);

    // Optimization. Collect unique ids so we can quickly check if a transaction HASN'T been recorded.
    let recorded_ids: HashSet<&str> = hledger_transactions
        .iter()
        .flat_map(|t| t.get_all_ids(hledger_account))
        .collect();

    real_transactions
        .iter()
        // Only real transactions which haven't already been recorded
        .filter(|real| !recorded_ids.contains(&*real.get_id()))
        // Apply any matching rules to the real transactions
        .filter_map(|real| {
            rules.iter().find_map(|rule| {
                rule.apply(&templater, hledger_account, real)
                    .map(|gen| TransactionResponse {
                        real_transaction: real.to_json_value(),
                        hledger_transaction: Some(gen),
                        rule: Some(rule.to_owned()),
                    })
            })
        })
        .collect()
}

fn get_errors(
    import_account: &impl ImportAccount,
    distinct_recorded_ids: &HashMap<&str, u8>,
    real_transaction: &Option<&impl RealTransaction>,
    hledger_transaction: &HledgerTransaction,
) -> Vec<String> {
    let import_hledger_account = import_account.get_hledger_account();
    let mut errors: Vec<String> = hledger_transaction
        .get_all_ids(import_hledger_account)
        .filter_map(|id| {
            distinct_recorded_ids.get(id).map(|count| {
                if count > &1 {
                    Some(format!("Duplicate ID {}", id))
                } else {
                    None
                }
            })
        })
        .flatten()
        .collect();
    if let Some(r) = real_transaction {
        let hledger_amount: Decimal = hledger_transaction
            .get_amount(Some(&r.get_id()), import_hledger_account)
            .unwrap();
        if let Some(real_amount) = r.get_field(r.get_default_amount_field_name()) {
            if hledger_amount != real_amount {
                errors.push("Amounts don't match".to_string());
            }
        }
        let h_date = hledger_transaction.get_date(Some(import_hledger_account));
        if h_date != r.get_date() {
            errors.push(format!(
                "Dates don't match. hledger: {}. Real: {}",
                h_date,
                r.get_date()
            ));
        }
    } else {
        errors.push("Recorded transaction without corresponding Real".to_string());
    }

    errors
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use rust_decimal::{prelude::FromPrimitive, Decimal};

    use super::get_generated_transactions;
    use crate::{
        model::real_transaction::RealTransaction,
        test_statics::{ASSET_ACCOUNT, EXPENSE_ACCOUNT, REAL, RECORDED, RULES},
    };

    #[test]
    fn generated() {
        let gen = get_generated_transactions(ASSET_ACCOUNT, &*RECORDED, &*REAL, &*RULES);
        // 1st item is filtered as already recorded, 2nd item doesn't match rule
        assert_eq!(gen.len(), 1);
        let gen = &gen[0];
        let t = gen.hledger_transaction.as_ref().unwrap();

        println!("{:#?}", t);

        assert_eq!(gen.rule.as_ref().unwrap().id, RULES[0].id);
        assert_eq!(
            gen.real_transaction.as_object().unwrap()["id"]
                .as_str()
                .unwrap(),
            REAL[2].get_id()
        );
        assert_eq!(t.tdescription, "Test Amazon with Buy item 3");
        let date = t.get_date(None);
        assert_eq!(date.year(), 2020);
        assert_eq!(date.month(), 8);
        assert_eq!(date.day(), 13);
        assert_eq!(t.ttags[0][0], "uuid");
        assert_eq!(t.ttags[0][1], REAL[2].get_id());
        assert_eq!(t.tpostings[0].paccount, ASSET_ACCOUNT);
        assert_eq!(t.tpostings[1].paccount, EXPENSE_ACCOUNT);
        assert_eq!(t.get_amount(None, ASSET_ACCOUNT), Decimal::from_f64(-3.));
        assert_eq!(t.get_amount(None, EXPENSE_ACCOUNT), Decimal::from_f64(3.));
    }
}
