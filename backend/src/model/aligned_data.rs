use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct AlignedData {
    pub x_values: Vec<serde_json::Number>,
    pub y_values: Vec<Vec<serde_json::Number>>,
}

impl Serialize for AlignedData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(std::iter::once(&self.x_values).chain(self.y_values.iter()))
    }
}
