use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Data {
    pub city: String,
    pub country: String,
    #[serde(rename = "popcount")]
    pub pop_count: u64,
}
