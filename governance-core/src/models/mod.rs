use std::fs::File;
use std::io::{BufReader, Error as IoError, ErrorKind as IoErrorKind, Read, Result};
use std::mem::size_of;
use std::path::Path;

use serde_json::Value;

use self::events::*;

pub mod events;

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
pub fn json_file<T: AsRef<Path>>(path: T) -> Result<(String, Events)> {
    let mut data = String::new();
    let f = File::open(path.as_ref())?;

    let mut reader = BufReader::new(f);
    reader.read_to_string(&mut data)?;

    json_bytes(data.as_bytes())
}

/// Parse raw json bytes, return (account_name, Events)
pub fn json_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<(String, Events)> {
    let json: Value = serde_json::from_slice(bytes.as_ref())?;

    if let Value::Object(obj) = json {
        let mut account = obj.into_iter();

        if let Some((account_name, events)) = account.next() {
            return Ok((account_name, serde_json::from_value(events)?));
        }
    }

    Err(IoError::from(IoErrorKind::InvalidData))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    use chrono::DateTime;

    #[test]
    fn json_from_file() {
        let file = json_file(concat!("../assets/data.json")).expect("Cannot parse file");

        assert_eq!(
            file,
            (
                String::from("TestAccount1"),
                Events {
                    events: vec![Event {
                        occurrence: DateTime::from_str("2022-09-26T12:00:00Z").unwrap(),
                        cause: Cause::Rebalance,
                        changes: vec![
                            Change {
                                change_type: ChangeType::Cash,
                                size: -99814.8,
                                security_definition: SecurityDefinition::None,
                                price: 1.0,
                            },
                            Change {
                                change_type: ChangeType::Security,
                                size: 984.0,
                                security_definition: SecurityDefinition::TSWE,
                                price: 25.36,
                            },
                            Change {
                                change_type: ChangeType::Security,
                                size: 681.0,
                                security_definition: SecurityDefinition::TRET,
                                price: 36.65,
                            },
                            Change {
                                change_type: ChangeType::Security,
                                size: 2131.0,
                                security_definition: SecurityDefinition::TGBT,
                                price: 11.71,
                            },
                            Change {
                                change_type: ChangeType::Security,
                                size: 1585.0,
                                security_definition: SecurityDefinition::TCBT,
                                price: 15.74,
                            }
                        ]
                    }]
                }
            )
        )
    }
}
