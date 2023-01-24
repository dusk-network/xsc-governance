use std::fmt::{self, Display, Formatter};

use canonical::Canon;
use canonical_derive::Canon;
use chrono::DateTime;
use chrono::Utc;
use dusk_pki::PublicKey;
use serde::{Deserialize, Deserializer};
use tai64::Tai64;

use super::public_key;

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
    pub security_definition: SecurityDefinition,
    #[serde(deserialize_with = "to_float_bytes")]
    pub price: u64,
}

#[derive(Debug, Clone, Canon, PartialEq, Eq, Deserialize)]
pub enum ChangeType {
    Cash,
    Security,
}

// Contract IDs are specified for each Security as they are smart contracts
#[derive(Debug, Clone, Canon, Hash, PartialEq, Eq, Deserialize)]
pub enum SecurityDefinition {
    Cash = 0x1000,
    #[serde(rename = "TSWE")]
    Tswe = 0x1001,
    #[serde(rename = "TRET")]
    Tret = 0x1002,
    #[serde(rename = "TGBT")]
    Tgbt = 0x1003,
    #[serde(rename = "TCBT")]
    Tcbt = 0x1004,
    None = 0x000,
}

impl SecurityDefinition {
    pub fn to_public_key(self) -> PublicKey {
        public_key(self.to_string())
    }
}

impl Display for SecurityDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        use self::SecurityDefinition::*;

        let x = match self {
            Cash => "Cash",
            Tswe => "TSWE",
            Tret => "TRET",
            Tgbt => "TGBT",
            Tcbt => "TCBT",
            None => "None",
        };

        write!(f, "{x}")
    }
}

// We need this because Canon is not implemented for f64
fn to_float_bytes<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: f64 = Deserialize::deserialize(deserializer)?;
    // TODO: Find out why I wrote this
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
