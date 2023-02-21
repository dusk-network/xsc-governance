use std::fs::File;
use std::io::{BufReader, Read};

use governance_core::models::*;
use serde_json::Value;

#[test]
fn valid() {
    let path = "../assets/data.json";

    let mut data = String::new();
    let f = File::open(path).unwrap();

    let mut reader = BufReader::new(f);
    reader.read_to_string(&mut data).unwrap();

    let json = serde_json::from_slice(data.as_bytes()).unwrap();

    if let Value::Object(obj) = json {
        if let Some((_, events)) = obj.into_iter().next() {
            serde_json::from_value::<Events>(events)
                .unwrap()
                .events
                .into_iter()
                .for_each(|event| {
                    event.changes.into_iter().for_each(|change| {
                        let security = change.security_definition;

                        match event.cause {
                            Cause::Rebalance => {
                                assert!(security != SecurityDefinition::Cash);
                            }
                            Cause::Deposit | Cause::Withdraw | Cause::Fee => {
                                assert!(security == SecurityDefinition::Cash);
                            }
                        }
                    })
                });
        }
    };
}
