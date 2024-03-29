use std::convert::TryInto;

use chrono::NaiveDate;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use serde::{Deserialize, Serialize};

use super::{real_transaction::RealTransaction, rule::RulePosting};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Quantity {
    floating_point: f64,
    decimal_places: i32,
    decimal_mantissa: i64,
}
impl From<Decimal> for Quantity {
    fn from(decimal: Decimal) -> Self {
        let floating_point = decimal.to_f64().unwrap_or_default();
        let decimal_places = decimal.scale() as i32;
        let decimal_mantissa = decimal.mantissa().try_into().unwrap();
        Quantity {
            floating_point,
            decimal_places,
            decimal_mantissa,
        }
    }
}

impl From<Quantity> for Decimal {
    fn from(quantity: Quantity) -> Self {
        Decimal::new(quantity.decimal_mantissa, quantity.decimal_places as u32)
    }
}

impl From<&Quantity> for Decimal {
    fn from(quantity: &Quantity) -> Self {
        Decimal::new(quantity.decimal_mantissa, quantity.decimal_places as u32)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tag", content = "contents")]
pub enum Price {
    UnitPrice(Amount),
    TotalPrice(Amount),
}

impl Price {
    pub fn new(commodity: &str, quantity: Decimal) -> Self {
        Self::UnitPrice(Amount::new(commodity, quantity))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum Precision {
    Precision(u8),
    NaturalPrecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AmountStyle {
    ascommodityside: String,
    ascommodityspaced: bool,
    asprecision: Precision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amount {
    aprice: Option<Box<Price>>,
    acommodity: String,
    aquantity: Quantity,
    astyle: AmountStyle,
}

impl Amount {
    fn new(commodity: &str, quantity: Decimal) -> Self {
        Self::new_priced(commodity, quantity, None)
    }

    fn new_priced(commodity: &str, quantity: Decimal, price: Option<Price>) -> Self {
        Self {
            acommodity: commodity.to_string(),
            aquantity: quantity.into(),
            astyle: AmountStyle {
                ascommodityside: String::from("R"),
                ascommodityspaced: true,
                asprecision: Precision::Precision(quantity.scale() as u8),
            },
            aprice: price.map(Box::new),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Posting {
    pub paccount: String,
    pdate: Option<NaiveDate>,
    pamount: Vec<Amount>,
    pstatus: String,
    pcomment: String,
    ptype: String,
    ptags: Vec<Vec<String>>,
}

impl Posting {
    pub fn new(
        account: &str,
        commodity: &str,
        amount: Decimal,
        price: Option<Price>,
        comment: Option<&str>,
    ) -> Self {
        Self {
            paccount: account.to_string(),
            pdate: None,
            pamount: vec![Amount::new_priced(commodity, amount, price)],
            pstatus: String::from("Unmarked"),
            pcomment: comment.unwrap_or_default().to_string(),
            ptype: String::from("RegularPosting"),
            ptags: vec![],
        }
    }

    fn get_id(&self) -> Option<&str> {
        get_uuid_from_tags(&self.ptags)
    }

    fn get_amount(&self) -> Option<Decimal> {
        match self.pamount.len() {
            1 => Some((&self.pamount[0].aquantity).into()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourcePos {
    source_name: String,
    source_line: u32,
    source_column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HledgerTransaction {
    pub tdescription: String,
    pub ttags: Vec<Vec<String>>,
    pub tpostings: Vec<Posting>,
    tdate: NaiveDate,
    tcode: String,
    tcomment: String,
    tprecedingcomment: String,
    tdate2: Option<NaiveDate>,
    tstatus: String,
    tindex: i32,
    tsourcepos: (SourcePos, SourcePos),
}

impl HledgerTransaction {
    pub fn new(description: &str, date: NaiveDate, id: &str) -> Self {
        Self {
            tdescription: description.to_string(),
            tdate: date,
            ttags: vec![vec!["uuid".to_string(), id.to_string()]],
            tpostings: Vec::<Posting>::new(),
            tcode: String::new(),
            tcomment: format!("uuid:{}", id),
            tprecedingcomment: String::new(),
            tdate2: None,
            tstatus: String::from("Unmarked"),
            tindex: 1,
            tsourcepos: (
                SourcePos {
                    source_name: String::new(),
                    source_line: 1,
                    source_column: 1,
                },
                SourcePos {
                    source_name: String::new(),
                    source_line: 1,
                    source_column: 1,
                },
            ),
        }
    }

    pub fn new_with_postings<T>(
        real_transaction: &T,
        hledger_account: &str,
        description: &str,
        postings: &[RulePosting],
    ) -> Self
    where
        T: RealTransaction,
    {
        Self::new(
            description,
            real_transaction.get_date(),
            &real_transaction.get_id(),
        )
        .postings(&mut real_transaction.get_postings(hledger_account, postings))
    }

    /// Add posting
    pub fn _posting(mut self, posting: Posting) -> Self {
        self.tpostings.push(posting);
        self
    }

    // TODO: This mut vec stuff is ugly
    pub fn postings(mut self, posting: &mut Vec<Posting>) -> Self {
        self.tpostings.append(posting);
        self
    }

    pub fn get_all_ids(&self, account: &str) -> impl Iterator<Item = &str> {
        self.get_postings(account)
            .into_iter()
            .filter_map(Posting::get_id)
            .chain(self.get_id().into_iter())
    }

    pub fn get_amount(&self, id: Option<&str>, account: &str) -> Option<Decimal> {
        if let Some(id) = id {
            let amount = self.get_amount_for_posting_id(id);
            if amount.is_some() {
                return amount;
            }
        }
        self.find_amount_from_account(account)
    }

    pub fn get_date(&self, account: Option<&str>) -> NaiveDate {
        if let Some(account) = account {
            if let Some(date) = self.get_postings(account).into_iter().find_map(|p| p.pdate) {
                return date;
            }
        }
        self.tdate
    }

    pub fn get_id(&self) -> Option<&str> {
        get_uuid_from_tags(&self.ttags)
    }

    pub fn has_account(&self, account: &str) -> bool {
        !self.get_postings(account).is_empty()
    }

    // When id is non-posting id, then need to search for amount via account
    fn find_amount_from_account(&self, account: &str) -> Option<Decimal> {
        self.get_postings(account)
            .into_iter()
            .find_map(|p| p.get_amount())
    }

    fn get_amount_for_posting_id(&self, id: &str) -> Option<Decimal> {
        self.get_posting(id).map(Posting::get_amount).flatten()
    }

    fn get_posting(&self, id: &str) -> Option<&Posting> {
        self.tpostings.iter().find(|p| match p.get_id() {
            Some(pid) => pid == id,
            None => false,
        })
    }

    fn get_postings(&self, account: &str) -> Vec<&Posting> {
        self.tpostings
            .iter()
            .filter(|p| p.paccount.contains(account))
            .collect()
    }
}

fn get_uuid_from_tags(tags: &[Vec<String>]) -> Option<&str> {
    tags.iter()
        .find(|t| t[0] == "uuid")
        .map(|found| found[1].as_str())
}

#[cfg(test)]
mod tests {
    use rust_decimal::{prelude::FromPrimitive, Decimal};

    use super::{Amount, HledgerTransaction, Quantity};
    use crate::model::hledger_transaction::Precision;

    #[test]
    fn check_decimal_conversion() {
        let decimal = Decimal::from_f32(163.17).unwrap();
        let quantity: Quantity = decimal.into();
        let decimal2: Decimal = quantity.into();
        assert_eq!(decimal, decimal2);
    }

    #[test]
    fn check_amount_precision() {
        let quantity = Decimal::from_f32(123.4567).unwrap();
        let amt = Amount::new_priced("EUR", quantity, None);
        match amt.astyle.asprecision {
            Precision::Precision(p) => assert_eq!(p, 4),
            Precision::NaturalPrecision => panic!(),
        }
    }

    #[test]
    fn deserialize() {
        let json = r#"{
    "tcomment": "uuid:1234\n",
    "tstatus": "Unmarked",
    "ttags": [["uuid", "1234"]],
    "tprecedingcomment": "",
    "tpostings": [
      {
        "pdate": null,
        "poriginal": null,
        "ptags": [],
        "paccount": "Assets:Cash:N26",
        "pdate2": null,
        "ptype": "RegularPosting",
        "pbalanceassertion": null,
        "pstatus": "Unmarked",
        "pamount": [
          {
            "aprice": null,
            "acommodity": "EUR",
            "astyle": {
              "asprecision": 2,
              "asdecimalpoint": ".",
              "ascommodityspaced": true,
              "asdigitgroups": null,
              "ascommodityside": "R"
            },
            "aquantity": { "floatingPoint": -3.57, "decimalPlaces": 2, "decimalMantissa": -357 }
          }
        ],
        "ptransaction_": "1",
        "pcomment": ""
      },
      {
        "pdate": null,
        "poriginal": null,
        "ptags": [],
        "paccount": "Expenses:Personal:Food:Groceries",
        "pdate2": null,
        "ptype": "RegularPosting",
        "pbalanceassertion": null,
        "pstatus": "Unmarked",
        "pamount": [
          {
            "aprice": null,
            "acommodity": "EUR",
            "astyle": {
              "asprecision": 2,
              "asdecimalpoint": ".",
              "ascommodityspaced": true,
              "asdigitgroups": null,
              "ascommodityside": "R"
            },
            "aquantity": { "floatingPoint": 3.57, "decimalPlaces": 2, "decimalMantissa": 357 }
          }
        ],
        "ptransaction_": "1",
        "pcomment": ""
      }
    ],
    "tcode": "",
    "tdate": "2020-01-02",
    "tsourcepos": [
      {
        "sourceLine": 1,
        "sourceName": "abc.ledger",
        "sourceColumn": 1
      },
      {
        "sourceLine": 4,
        "sourceName": "abc.ledger",
        "sourceColumn": 1
      }
    ],
    "tindex": 1,
    "tdescription": "My Groceries",
    "tdate2": null
}"#;

        let t: HledgerTransaction = serde_json::from_str(json).unwrap();
        assert_eq!(
            t.get_amount(None, "Groceries").unwrap(),
            Decimal::from_f32(3.57).unwrap()
        );
    }
}
