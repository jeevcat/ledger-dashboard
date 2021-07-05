use rust_decimal::{prelude::ToPrimitive, Decimal};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalancesResponse {
    pub balances: Vec<BalanceResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    pub commodity: String,
    pub hledger: Decimal,
    pub real: Decimal,
    pub real_euro: Option<Decimal>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct RealBalance {
    /// The commodity of this balance
    pub commodity: String,
    /// The amount of this balance
    #[serde(serialize_with = "decimal_to_f64")]
    pub amount: Decimal,
    /// The amount of this balance converted to the base currency
    pub base_amount: Option<Decimal>,
}

fn decimal_to_f64<S>(d: &Decimal, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let v = d
        .to_f64()
        .ok_or_else(|| serde::ser::Error::custom("Couldn't convert Decimal to f64"))?;
    s.serialize_f64(v)
}
