use std::fs::File;
use std::io::{self, BufReader, Error as IoError, ErrorKind as IoErrorKind, Read};
use std::mem::size_of;
use std::path::Path;

use canonical_derive::Canon;
use dusk_pki::{PublicKey, SecretKey};
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json::Value;

use self::events::*;

pub mod events;

#[derive(Debug, Clone, Canon, PartialEq, Eq)]
pub struct Transfer {
    pub from: PublicKey,
    pub to: PublicKey,
    pub amount: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Address(pub [u8; 64]);

impl Address {
    pub fn buffer() -> [u8; size_of::<Address>()] {
        [0; size_of::<Address>()]
    }
}

impl Default for Address {
    fn default() -> Self {
        Self(Self::buffer())
    }
}

/// Parse a json file, return (account_name, Events)
pub fn json_file<T: AsRef<Path>>(path: T) -> io::Result<Vec<Transfer>> {
    let mut data = String::new();
    let f = File::open(path.as_ref())?;

    let mut reader = BufReader::new(f);
    reader.read_to_string(&mut data)?;

    json_bytes(data.as_bytes())
}

/// Parse raw json bytes, return (account_name, Events)
pub fn json_bytes<T: AsRef<[u8]>>(bytes: T) -> io::Result<Vec<Transfer>> {
    let json: Value = serde_json::from_slice(bytes.as_ref())?;

    if let Value::Object(obj) = json {
        let mut account = obj.into_iter();

        if let Some((account_name, events)) = account.next() {
            let events: Events = serde_json::from_value(events)?;
            let mut transfers = Vec::new();

            for event in events.events {
                let timestamp = event.occurrence;

                // TODO: Add condition for other types of causes
                if event.cause == Cause::Deposit {
                    let from = public_key(&account_name);

                    for change in event.changes {
                        let to = public_key(change.security_definition);
                        let amount = change.size;

                        transfers.push(Transfer {
                            from,
                            to,
                            amount,
                            timestamp,
                        })
                    }
                }
            }

            return Ok(transfers);
        }
    }

    Err(IoError::from(IoErrorKind::InvalidData))
}

fn public_key<T: AsRef<[u8]>>(phrase: T) -> PublicKey {
    let hash = blake3::hash(phrase.as_ref());

    let mut seed = StdRng::from_seed(*hash.as_bytes());
    let secret_key = SecretKey::random(&mut seed);
    PublicKey::from(&secret_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    use chrono::{DateTime, Utc};
    use tai64::Tai64;

    #[test]
    fn json_from_file() {
        let mut bytes = String::new();
        let f = File::open(concat!("../assets/data.json")).expect("Cannot find file");

        let mut reader = BufReader::new(f);
        reader.read_to_string(&mut bytes).expect("Cannot read file");

        let json: Value = serde_json::from_slice(bytes.as_ref()).expect("error in serialising");

        let timestamp = Tai64::from_unix(
            DateTime::<Utc>::from_str("2022-09-26T12:00:00Z")
                .expect("Cannot convert timestamp to datetime")
                .timestamp(),
        );

        if let Value::Object(obj) = json {
            let mut account = obj.into_iter();

            if let Some(data) = account.next() {
                let data = (
                    data.0,
                    serde_json::from_value(data.1).expect("error in serialising"),
                );

                assert_eq!(
                    data,
                    (
                        String::from("TestAccount1"),
                        Events {
                            events: vec![Event {
                                occurrence: timestamp.0,
                                cause: Cause::Rebalance,
                                changes: vec![
                                    Change {
                                        change_type: ChangeType::Cash,
                                        size: ((-99814.8f64) * 1000000.0) as u64,
                                        security_definition: String::from("None"),
                                        price: ((1.0f64) * 1000000.0) as u64,
                                    },
                                    Change {
                                        change_type: ChangeType::Security,
                                        size: (984.0f64 * 1000000.0) as u64,
                                        security_definition: String::from("TSWE"),
                                        price: (25.36f64 * 1000000.0) as u64,
                                    },
                                    Change {
                                        change_type: ChangeType::Security,
                                        size: (681.0f64 * 1000000.0) as u64,
                                        security_definition: String::from("TRET"),
                                        price: (36.65f64 * 1000000.0) as u64,
                                    },
                                    Change {
                                        change_type: ChangeType::Security,
                                        size: (2131.0f64 * 1000000.0) as u64,
                                        security_definition: String::from("TGBT"),
                                        price: (11.71f64 * 1000000.0) as u64,
                                    },
                                    Change {
                                        change_type: ChangeType::Security,
                                        size: (1585.0f64 * 1000000.0) as u64,
                                        security_definition: String::from("TCBT"),
                                        price: (15.74f64 * 1000000.0) as u64,
                                    }
                                ]
                            }]
                        }
                    )
                )
            }
        }
        let timestamp = Tai64::from_unix(
            DateTime::<Utc>::from_str("2022-09-26T12:00:00Z")
                .expect("Cannot convert timestamp to datetime")
                .timestamp(),
        );
    }
}
