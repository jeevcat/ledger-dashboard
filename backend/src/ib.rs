use csv::StringRecord;
use serde::Deserialize;

use crate::{file_utils::get_database_file, model::ib_report::IbReport};

pub struct Ib;

impl Ib {
    pub fn read_report() -> IbReport {
        let filename = "ib.csv";
        let file_path = get_database_file(filename).unwrap();
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_path(file_path)
            .unwrap();
        let mut report = IbReport::default();
        let mut record = StringRecord::new();
        while rdr.read_record(&mut record).unwrap_or_default() {
            match &record[1] {
                "Header" => rdr.set_headers(record.clone()),
                "Data" => {
                    let headers = rdr.headers().unwrap();
                    report.deserialize_to_report(&record, headers);
                }
                _ => (),
            }
        }
        report
    }
}

pub fn deserialize_record<'a, T>(
    record: &'a StringRecord,
    headers: &'a StringRecord,
    target_vec: &mut Vec<T>,
) where
    T: Deserialize<'a>,
{
    if record[2].to_ascii_lowercase().contains("total") {
        // Some rows are totals, which we should skip
        return;
    }
    match record.deserialize::<'a, T>(Some(headers)) {
        Ok(row) => target_vec.push(row),
        Err(e) => println!("{:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::Ib;

    #[test]
    fn example() {
        let report = Ib::read_report();
        println!("{:#?}", report);
    }
}
