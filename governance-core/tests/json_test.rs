// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use governance_core::prelude::*;
use serde_json::Value;

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
        let bytes = include_bytes!("data.json");

        let json: Value = serde_json::from_slice(bytes.as_ref())
            .expect("error in serialising");

        if let Value::Object(obj) = json {
            let mut account = obj.into_iter();

            while let Some((name, events)) = account.next() {
                // check only for Dusk1 account
                if name == "Dusk1" {
                    let events: Events = serde_json::from_value(events)
                        .expect("error in serialising");

                    assert_eq!(
                        events,
                        Events {
                            events: vec![
                                Event {
                                    occurrence: timestamp(
                                        "2022-09-25T10:00:00Z"
                                    ),
                                    cause: Cause::Deposit,
                                    changes: vec![Change {
                                        account_external_id: String::from(
                                            "TestAccount1"
                                        ),
                                        change_type: ChangeType::Cash,
                                        size: 100000.0,
                                        security: SecurityDefinition::None,
                                        price: 1.0,
                                    },]
                                },
                                Event {
                                    occurrence: timestamp(
                                        "2022-09-26T12:00:00Z"
                                    ),
                                    cause: Cause::Rebalance,
                                    changes: vec![
                                        Change {
                                            account_external_id: String::from(
                                                "TestAccount1"
                                            ),
                                            change_type: ChangeType::Cash,
                                            size: -99814.8,
                                            security: SecurityDefinition::None,
                                            price: 1.0,
                                        },
                                        Change {
                                            account_external_id: String::from(
                                                "TestAccount1"
                                            ),
                                            change_type: ChangeType::Security,
                                            size: 984.0,
                                            security: SecurityDefinition::Tswe,
                                            price: 25.36,
                                        },
                                        Change {
                                            account_external_id: String::from(
                                                "TestAccount1"
                                            ),
                                            change_type: ChangeType::Security,
                                            size: 681.0,
                                            security: SecurityDefinition::Tret,
                                            price: 36.65,
                                        },
                                        Change {
                                            account_external_id: String::from(
                                                "TestAccount1"
                                            ),
                                            change_type: ChangeType::Security,
                                            size: 2131.0,
                                            security: SecurityDefinition::Tgbt,
                                            price: 11.71,
                                        },
                                        Change {
                                            account_external_id: String::from(
                                                "TestAccount1"
                                            ),
                                            change_type: ChangeType::Security,
                                            size: 1585.0,
                                            security: SecurityDefinition::Tcbt,
                                            price: 15.74,
                                        }
                                    ]
                                },
                                Event {
                                    occurrence: timestamp(
                                        "2023-01-27T14:59:11.439Z"
                                    ),
                                    cause: Cause::Deposit,
                                    changes: vec![Change {
                                        account_external_id: String::from(
                                            "TestAccount1"
                                        ),
                                        change_type: ChangeType::Cash,
                                        size: 3000.0,
                                        security: SecurityDefinition::None,
                                        price: 1.0,
                                    },]
                                },
                                Event {
                                    occurrence: timestamp(
                                        "2023-01-27T15:00:44.117Z"
                                    ),
                                    cause: Cause::Rebalance,
                                    changes: vec![
                                        Change {
                                            account_external_id: String::from(
                                                "TestAccount1"
                                            ),
                                            change_type: ChangeType::Security,
                                            size: -10.9919,
                                            security: SecurityDefinition::Tswe,
                                            price: 27.3,
                                        },
                                        Change {
                                            account_external_id: String::from(
                                                "TestAccount1"
                                            ),
                                            change_type: ChangeType::Security,
                                            size: 30.3859,
                                            security: SecurityDefinition::Tret,
                                            price: 37.35,
                                        },
                                        Change {
                                            account_external_id: String::from(
                                                "TestAccount1"
                                            ),
                                            change_type: ChangeType::Security,
                                            size: 57.1205,
                                            security: SecurityDefinition::Tcbt,
                                            price: 16.18,
                                        },
                                        Change {
                                            account_external_id: String::from(
                                                "TestAccount1"
                                            ),
                                            change_type: ChangeType::Security,
                                            size: 95.2472,
                                            security: SecurityDefinition::Tgbt,
                                            price: 11.94,
                                        },
                                        Change {
                                            account_external_id: String::from(
                                                "TestAccount1"
                                            ),
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
