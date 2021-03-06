use std::collections::{HashMap, HashSet};

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

pub fn get_generated_transactions<'a, ReaIter, Rea>(
    recorded_transactions: &[RecordedTransaction],
    real_transactions: ReaIter,
    rules: &[Rule],
) -> Vec<TransactionResponse>
where
    ReaIter: IntoIterator<Item = &'a Rea>,
    Rea: RealTransaction,
    Rea: 'a,
{
    let templater = Templater::from_rules(rules);

    // Optimization. Collect unique ids so we can quickly check if a transaction HASN'T been recorded.
    let recorded_ids: HashSet<&str> = recorded_transactions.iter().flat_map(|t| t.ids()).collect();

    real_transactions
        .into_iter()
        // Only real transactions which haven't already been recorded
        .filter(|real| !recorded_ids.contains(&*real.get_id()))
        // Apply any matching rules to the real transactions
        .filter_map(|real| {
            rules.iter().find_map(|rule| {
                rule.apply(&templater, real).map(|gen| TransactionResponse {
                    real_transaction: real.to_json_value(),
                    recorded_transaction: Some(gen),
                    rule: Some(rule.to_owned()),
                })
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use lazy_static::lazy_static;
    use regex::Regex;

    use super::get_generated_transactions;
    use crate::model::{
        n26transaction::N26Transaction, real_transaction::RealTransaction,
        recorded_transaction::RecordedTransaction, rule::Rule,
    };

    lazy_static! {
        static ref RULES: Vec<Rule> = vec![Rule {
            match_field_name: "partnerName".to_string(),
            match_field_regex: Regex::new("(?i)amazon").unwrap(),
            account: "Expenses:Personal:Fun".to_string(),
            description_template: "Test {{{partnerName}}} with {{{referenceText}}}".to_string(),
            ..Rule::default()
        }];
        static ref REAL: Vec<N26Transaction> = serde_json::from_str(
            r#"[
            {
                "id": "1fc7d65c-de7c-415f-bf17-94de40c2e5d2",
                "amount": 219.56,
                "currencyCode": "EUR",
                "visibleTS": 1597308032422,
                "partnerName": "Amazon",
                "referenceText": "Buy item 1"
            },
            {
                "id": "b33d6f8f-733c-4bf8-bef5-206cb3c27171",
                "amount": 123.45,
                "currencyCode": "EUR",
                "visibleTS": 1597308032422,
                "partnerName": "Supermarket",
                "referenceText": "Buy item 2"
            },
            {
                "id": "02946eaf-8320-4d2d-b44c-54c473771e68",
                "amount": 3,
                "currencyCode": "USD",
                "visibleTS": 1597308032422,
                "partnerName": "Amazon",
                "referenceText": "Buy item 3"
            }
        ]"#,
        )
        .unwrap();
        static ref RECORDED: Vec<RecordedTransaction> =
            vec![RecordedTransaction::new_with_postings(
                &REAL[0],
                "My Description",
                "Expenses:Personal:Test"
            )];
    }

    #[test]
    fn generated() {
        let gen = get_generated_transactions(&*RECORDED, &*REAL, &*RULES);
        // 1st item is filtered as already recorded, 2nd item doesn't match rule
        assert_eq!(gen.len(), 1);
        let gen = &gen[0];
        let t = gen.recorded_transaction.as_ref().unwrap();
        assert_eq!(gen.rule.as_ref().unwrap().id, RULES[0].id);
        assert_eq!(
            gen.real_transaction.as_object().unwrap()["id"]
                .as_str()
                .unwrap(),
            REAL[2].get_id()
        );
        assert_eq!(t.tdescription, "Test Amazon with Buy item 3");
        assert_eq!(t.tdate.year(), 2020);
        assert_eq!(t.tdate.month(), 8);
        assert_eq!(t.tdate.day(), 13);
        assert_eq!(t.ttags[0][0], "uuid");
        assert_eq!(t.ttags[0][1], REAL[2].get_id());
        assert_eq!(t.tpostings[0].paccount, "Assets:Cash:N26");
        assert_eq!(t.tpostings[1].paccount, "Expenses:Personal:Fun");
    }
}
