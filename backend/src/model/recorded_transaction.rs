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
impl Quantity {
    fn new(decimal: Decimal) -> Quantity {
        let floating_point = decimal.to_f64().unwrap_or_default();
        let decimal_places = decimal.scale();
        let decimal_mantissa = (10f64.powf(decimal_places as f64) * floating_point) as i64;
        Quantity {
            floating_point,
            decimal_places,
            decimal_mantissa,
        }
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
            pamount: vec![Amount {
                acommodity: commodity.to_string(),
                aquantity: Quantity::new(amount),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tag", content = "contents")]
enum SourcePos {
    GenericSourcePos(String, i32, i32),
    JournalSourcePos(String, (i32, i32)),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedTransaction {
    pub tdescription: String,
    pub tdate: NaiveDate,
    pub ttags: Vec<Vec<String>>,
    pub tpostings: Vec<Posting>,
    tcode: String,
    tcomment: String,
    tprecedingcomment: String,
    tdate2: Option<NaiveDate>,
    tstatus: String,
    tindex: i32,
    tsourcepos: SourcePos,
}

impl RecordedTransaction {
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

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.tpostings
            .iter()
            .filter_map(|p| get_uuid_from_tags(&p.ptags))
            .chain(get_uuid_from_tags(&self.ttags).into_iter())
    }
}

fn get_uuid_from_tags(tags: &[Vec<String>]) -> Option<&str> {
    tags.iter()
        .find(|t| t[0] == "uuid")
        .map(|found| found[1].as_str())
}
