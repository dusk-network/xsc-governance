use std::fs::File;
use std::io::{self, BufReader, Error as IoError, ErrorKind as IoErrorKind, Read};
use std::path::Path;

use dusk_pki::{PublicKey, SecretKey};
use rand::{rngs::StdRng, SeedableRng};
use serde_json::Value;

use crate::prelude::*;

/// Parse a json file, convert them to Vec<Transfer>
pub fn json_file<T: AsRef<Path>>(path: T) -> io::Result<TransferMap> {
    let mut data = String::new();
    let f = File::open(path.as_ref())?;

    let mut reader = BufReader::new(f);
    reader.read_to_string(&mut data)?;

    json_bytes(data.as_bytes())
}

/// Parse raw json bytes convert them to Vec<Transfer>
pub fn json_bytes<T: AsRef<[u8]>>(bytes: T) -> io::Result<TransferMap> {
    let json: Value = serde_json::from_slice(bytes.as_ref())?;

    if let Value::Object(obj) = json {
        // the account name and all events are the first key value pairs
        if let Some((account_name, events)) = obj.into_iter().next() {
            let events: Events = serde_json::from_value(events)?;
            let events = events.events;
            let from = public_key(account_name);
            // Transfers holds all our transfers
            let mut transfers = TransferMap::new();

            for Event {
                occurrence,
                changes,
                cause,
                ..
            } in events
            {
                for Change { security, size, .. } in changes {
                    let to = public_key(security.to_string());

                    let mut tx = Transfer::new(size, occurrence);
                    // set the security of the transfer
                    transfers.security = security;

                    match cause {
                        Cause::Rebalance => {
                            if size < 0.0 {
                                tx.amount(-size);
                                transfers.insert_tx(tx.withdraw(from));
                            } else {
                                transfers.insert_tx(tx.deposit(to));
                            }
                        }
                        Cause::Deposit => {
                            transfers.insert_tx(tx.deposit(to));
                        }
                        Cause::Withdraw => {
                            transfers.insert_tx(tx.withdraw(from));
                        }
                        Cause::Fee => transfers.insert_fee(tx.withdraw(from)),
                    }
                }
            }

            return Ok(transfers);
        }
    }

    Err(IoError::from(IoErrorKind::InvalidData))
}

pub fn public_key<T: AsRef<[u8]>>(phrase: T) -> PublicKey {
    let hash = blake3::hash(phrase.as_ref());

    let mut seed = StdRng::from_seed(*hash.as_bytes());
    let secret_key = SecretKey::random(&mut seed);
    PublicKey::from(&secret_key)
}

#[cfg(test)]
mod test {
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
                let mut events: Events =
                    serde_json::from_value(data.1).expect("error in serialising");

                // We only deal with deposit for now
                events.events.remove(0);

                let data = (data.0, events);

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
                                        size: -99814.8,
                                        security: SecurityDefinition::None,
                                        price: 1.0,
                                    },
                                    Change {
                                        change_type: ChangeType::Security,
                                        size: 984.0,
                                        security: SecurityDefinition::Tswe,
                                        price: 25.36,
                                    },
                                    Change {
                                        change_type: ChangeType::Security,
                                        size: 681.0,
                                        security: SecurityDefinition::Tret,
                                        price: 36.65,
                                    },
                                    Change {
                                        change_type: ChangeType::Security,
                                        size: 2131.0,
                                        security: SecurityDefinition::Tgbt,
                                        price: 11.71,
                                    },
                                    Change {
                                        change_type: ChangeType::Security,
                                        size: 1585.0,
                                        security: SecurityDefinition::Tcbt,
                                        price: 15.74,
                                    }
                                ]
                            }]
                        }
                    )
                )
            }
        }
    }
}
