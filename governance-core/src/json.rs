// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::fs::File;
use std::io::{
    self, BufReader, Error as IoError, ErrorKind as IoErrorKind, Read,
};
use std::path::Path;

use dusk_pki::{PublicKey, SecretKey};
use rand::{rngs::StdRng, SeedableRng};
use serde_json::Value;

use crate::prelude::*;

/// Parse a json file, convert them to a map of Transfers
pub fn json_file<T: AsRef<Path>>(
    path: T,
    timestamp: Option<u64>,
) -> io::Result<TransferMap> {
    let mut data = String::new();
    let f = File::open(path.as_ref())?;

    let mut reader = BufReader::new(f);
    reader.read_to_string(&mut data)?;

    json_bytes(data.as_bytes(), timestamp)
}

/// Parse raw json bytes convert them to a map of Transfers
pub fn json_bytes<T: AsRef<[u8]>>(
    bytes: T,
    timestamp: Option<u64>,
) -> io::Result<TransferMap> {
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
                    mut security,
                    size,
                    change_type,
                    ..
                } in changes
                {
                    match change_type {
                        ChangeType::Reservation => continue,
                        ChangeType::Cash => security = SecurityDefinition::Cash,
                        ChangeType::Security => {
                            assert!(security != SecurityDefinition::None)
                        }
                    }

                    let to = public_key(security.to_string());

                    let ts = timestamp.unwrap_or(occurrence);
                    let mut tx = Transfer::new(size, ts);
                    match cause {
                        Cause::Rebalance => {
                            if size < 0.0 {
                                tx.amount(-size);
                                map.insert_tx(security, tx.withdraw(from));
                            } else {
                                map.insert_tx(security, tx.deposit(to));
                            }
                        }
                        Cause::Deposit => {
                            map.insert_tx(security, tx.deposit(to))
                        }
                        Cause::Withdrawal => {
                            map.insert_tx(security, tx.withdraw(from))
                        }
                        Cause::Fee => {
                            map.insert_fee(security, tx.withdraw(from))
                        }
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
