use std::convert::TryInto;

use chrono::NaiveDate;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use serde::{Deserialize, Serialize};

use super::real_transaction::RealTransaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Quantity {
    floating_point: f64,
    decimal_places: u32,
    decimal_mantissa: i64,
}
impl From<Decimal> for Quantity {
    fn from(decimal: Decimal) -> Self {
        let floating_point = decimal.to_f64().unwrap_or_default();
        let decimal_places = decimal.scale();
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
        Decimal::new(quantity.decimal_mantissa, quantity.decimal_places)
    }
}

impl From<&Quantity> for Decimal {
    fn from(quantity: &Quantity) -> Self {
        Decimal::new(quantity.decimal_mantissa, quantity.decimal_places)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Precision {
    tag: String,
    contents: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AmountStyle {
    ascommodityside: String,
    ascommodityspaced: bool,
    asprecision: Precision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Amount {
    acommodity: String,
    aquantity: Quantity,
    aismultiplier: bool,
    astyle: AmountStyle,
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
    pub fn new(account: &str, commodity: &str, amount: Decimal) -> Self {
        Self {
            paccount: account.to_string(),
            pdate: None,
            pamount: vec![Amount {
                acommodity: commodity.to_string(),
                aquantity: amount.into(),
                aismultiplier: false,
                astyle: AmountStyle {
                    ascommodityside: String::from("R"),
                    ascommodityspaced: true,
                    asprecision: Precision {
                        tag: "Precision".to_string(),
                        contents: 2,
                    },
                },
            }],
            pstatus: String::from("Unmarked"),
            pcomment: String::new(),
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
#[serde(tag = "tag", content = "contents")]
enum SourcePos {
    GenericSourcePos(String, i32, i32),
    JournalSourcePos(String, (i32, i32)),
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
    tsourcepos: SourcePos,
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
            tsourcepos: SourcePos::GenericSourcePos(String::new(), 1, 1),
        }
    }

    pub fn new_with_postings(
        real_transaction: &impl RealTransaction,
        description: &str,
        account: &str,
    ) -> Self {
        Self::new(
            &description,
            real_transaction.get_date(),
            &real_transaction.get_id(),
        )
        .posting(Posting::new(
            real_transaction.get_account(),
            real_transaction.get_currency(),
            real_transaction.get_amount(),
        ))
        .posting(Posting::new(
            account,
            real_transaction.get_currency(),
            -real_transaction.get_amount(),
        ))
    }

    /// Add posting
    pub fn posting(mut self, posting: Posting) -> Self {
        self.tpostings.push(posting);
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

    use super::Quantity;

    #[test]
    fn check_decimal_conversion() {
        let decimal = Decimal::from_f32(163.17).unwrap();
        let quantity: Quantity = decimal.into();
        let decimal2: Decimal = quantity.into();
        assert_eq!(decimal, decimal2);
    }
}
