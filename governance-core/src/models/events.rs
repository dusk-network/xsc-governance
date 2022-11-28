use canonical::Canon;
use canonical_derive::Canon;
use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Deserializer};
use tai64::Tai64;

#[derive(Debug, Clone, Canon, PartialEq, Eq, Deserialize)]
pub struct Events {
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Canon, PartialEq, Eq, Deserialize)]
pub struct Event {
    pub cause: Cause,
    pub changes: Vec<Change>,
    #[serde(deserialize_with = "to_tai64_timestamp")]
    pub occurrence: u64,
}

#[derive(Debug, Clone, Canon, PartialEq, Eq, Deserialize)]
pub enum Cause {
    Deposit,
    Withdraw,
    Rebalance,
    Fee,
}

#[derive(Debug, Clone, Canon, PartialEq, Eq, Deserialize)]
pub struct Change {
    #[serde(rename = "type")]
    pub change_type: ChangeType,
    #[serde(deserialize_with = "to_float_bytes")]
    pub size: u64,
    #[serde(rename = "securityDefinition")]
    pub security_definition: String,
    #[serde(deserialize_with = "to_float_bytes")]
    pub price: u64,
}

#[derive(Debug, Clone, Canon, PartialEq, Eq, Deserialize)]
pub enum ChangeType {
    Cash,
    Security,
}

fn to_float_bytes<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: f64 = Deserialize::deserialize(deserializer)?;
    let x = s * 1000000.0;

    Ok(x as u64)
}

fn to_tai64_timestamp<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: DateTime<Utc> = Deserialize::deserialize(deserializer)?;

    let timestamp = Tai64::from_unix(s.timestamp());

    Ok(timestamp.0)
}
