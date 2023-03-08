// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use governance_core::models::*;
use serde_json::Value;

#[test]
fn valid() {
    let bytes = include_bytes!("data.json");

    let json = serde_json::from_slice(bytes).unwrap();

    if let Value::Object(obj) = json {
        obj.into_iter().for_each(|(_, events)| {
            serde_json::from_value::<Events>(events)
                .unwrap()
                .events
                .into_iter()
                .for_each(|event| {
                    event.changes.into_iter().for_each(|change| {
                        let security = change.security;

                        match event.cause {
                            Cause::Rebalance => {
                                assert!(security != SecurityDefinition::Cash);
                            }
                            Cause::Deposit | Cause::Withdrawal | Cause::Fee => {
                                assert!(
                                    security == SecurityDefinition::Cash
                                        || security == SecurityDefinition::None
                                );
                            }
                        }
                    })
                });
        });
    };
}
