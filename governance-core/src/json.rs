// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::fs::File;
use std::io::{self, BufReader, Error as IoError, ErrorKind as IoErrorKind, Read};
use std::path::Path;

use dusk_pki::{PublicKey, SecretKey};
use rand::{rngs::StdRng, SeedableRng};
use serde_json::Value;

use crate::prelude::*;

/// Parse a json file, convert them to a map of Transfers
pub fn json_file<T: AsRef<Path>>(path: T) -> io::Result<TransferMap> {
    let mut data = String::new();
    let f = File::open(path.as_ref())?;

    let mut reader = BufReader::new(f);
    reader.read_to_string(&mut data)?;

    json_bytes(data.as_bytes())
}

/// Parse raw json bytes convert them to a map of Transfers
pub fn json_bytes<T: AsRef<[u8]>>(bytes: T) -> io::Result<TransferMap> {
    let json: Value = serde_json::from_slice(bytes.as_ref())?;

    if let Value::Object(obj) = json {
        // Transfers holds all our transfers
        let mut map = TransferMap::default();
        // the account name and all events are the first key value pairs
        for (account_name, events) in obj {
            let events: Events = serde_json::from_value(events)?;
            let events = events.events;
            let from = public_key(account_name);

            for Event {
                occurrence,
                changes,
                cause,
                ..
            } in events
            {
                for Change {
                    security,
                    size,
                    change_type,
                    ..
                } in changes
                {
                    if change_type == ChangeType::Reservation {
                        continue;
                    }

                    let to = public_key(security.to_string());

                    let mut tx = Transfer::new(size, occurrence);
                    match cause {
                        Cause::Rebalance => {
                            if size < 0.0 {
                                tx.amount(-size);
                                map.insert_tx(security, tx.withdraw(from));
                            } else {
                                map.insert_tx(security, tx.deposit(to));
                            }
                        }
                        Cause::Deposit => map.insert_tx(security, tx.deposit(to)),
                        Cause::Withdrawal => map.insert_tx(security, tx.withdraw(from)),
                        Cause::Fee => map.insert_fee(security, tx.withdraw(from)),
                    }
                }
            }
        }
        return Ok(map);
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

    fn timestamp(stamp: &str) -> u64 {
        let timestamp = Tai64::from_unix(
            DateTime::<Utc>::from_str(stamp)
                .expect("Cannot convert timestamp to datetime")
                .timestamp(),
        );

        timestamp.0
    }

    #[test]
    fn json_from_file() {
        let mut bytes = String::new();
        let f = File::open(concat!("../assets/data.json")).expect("Cannot find file");

        let mut reader = BufReader::new(f);
        reader.read_to_string(&mut bytes).expect("Cannot read file");

        let json: Value = serde_json::from_slice(bytes.as_ref()).expect("error in serialising");

        if let Value::Object(obj) = json {
            let mut account = obj.into_iter();

            while let Some((name, events)) = account.next() {
                // check only for Dusk1 account
                if name == "Dusk1" {
                    let events: Events =
                        serde_json::from_value(events).expect("error in serialising");

                    assert_eq!(
                        events,
                        Events {
                            events: vec![
                                Event {
                                    occurrence: timestamp("2022-09-25T10:00:00Z"),
                                    cause: Cause::Deposit,
                                    changes: vec![Change {
                                        account_external_id: String::from("TestAccount1"),
                                        change_type: ChangeType::Cash,
                                        size: 100000.0,
                                        security: SecurityDefinition::None,
                                        price: 1.0,
                                    },]
                                },
                                Event {
                                    occurrence: timestamp("2022-09-26T12:00:00Z"),
                                    cause: Cause::Rebalance,
                                    changes: vec![
                                        Change {
                                            account_external_id: String::from("TestAccount1"),
                                            change_type: ChangeType::Cash,
                                            size: -99814.8,
                                            security: SecurityDefinition::None,
                                            price: 1.0,
                                        },
                                        Change {
                                            account_external_id: String::from("TestAccount1"),
                                            change_type: ChangeType::Security,
                                            size: 984.0,
                                            security: SecurityDefinition::Tswe,
                                            price: 25.36,
                                        },
                                        Change {
                                            account_external_id: String::from("TestAccount1"),
                                            change_type: ChangeType::Security,
                                            size: 681.0,
                                            security: SecurityDefinition::Tret,
                                            price: 36.65,
                                        },
                                        Change {
                                            account_external_id: String::from("TestAccount1"),
                                            change_type: ChangeType::Security,
                                            size: 2131.0,
                                            security: SecurityDefinition::Tgbt,
                                            price: 11.71,
                                        },
                                        Change {
                                            account_external_id: String::from("TestAccount1"),
                                            change_type: ChangeType::Security,
                                            size: 1585.0,
                                            security: SecurityDefinition::Tcbt,
                                            price: 15.74,
                                        }
                                    ]
                                },
                                Event {
                                    occurrence: timestamp("2023-01-27T14:59:11.439Z"),
                                    cause: Cause::Deposit,
                                    changes: vec![Change {
                                        account_external_id: String::from("TestAccount1"),
                                        change_type: ChangeType::Cash,
                                        size: 3000.0,
                                        security: SecurityDefinition::None,
                                        price: 1.0,
                                    },]
                                },
                                Event {
                                    occurrence: timestamp("2023-01-27T15:00:44.117Z"),
                                    cause: Cause::Rebalance,
                                    changes: vec![
                                        Change {
                                            account_external_id: String::from("TestAccount1"),
                                            change_type: ChangeType::Security,
                                            size: -10.9919,
                                            security: SecurityDefinition::Tswe,
                                            price: 27.3,
                                        },
                                        Change {
                                            account_external_id: String::from("TestAccount1"),
                                            change_type: ChangeType::Security,
                                            size: 30.3859,
                                            security: SecurityDefinition::Tret,
                                            price: 37.35,
                                        },
                                        Change {
                                            account_external_id: String::from("TestAccount1"),
                                            change_type: ChangeType::Security,
                                            size: 57.1205,
                                            security: SecurityDefinition::Tcbt,
                                            price: 16.18,
                                        },
                                        Change {
                                            account_external_id: String::from("TestAccount1"),
                                            change_type: ChangeType::Security,
                                            size: 95.2472,
                                            security: SecurityDefinition::Tgbt,
                                            price: 11.94,
                                        },
                                        Change {
                                            account_external_id: String::from("TestAccount1"),
                                            change_type: ChangeType::Cash,
                                            size: -2896.09,
                                            security: SecurityDefinition::None,
                                            price: 1.0,
                                        },
                                    ]
                                }
                            ]
                        }
                    );
                }
            }
        }
    }
}
