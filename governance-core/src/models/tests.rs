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
            let mut events: Events = serde_json::from_value(data.1).expect("error in serialising");

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
                                    security_definition: SecurityDefinition::None,
                                    price: 1.0,
                                },
                                Change {
                                    change_type: ChangeType::Security,
                                    size: 984.0,
                                    security_definition: SecurityDefinition::Tswe,
                                    price: 25.36,
                                },
                                Change {
                                    change_type: ChangeType::Security,
                                    size: 681.0,
                                    security_definition: SecurityDefinition::Tret,
                                    price: 36.65,
                                },
                                Change {
                                    change_type: ChangeType::Security,
                                    size: 2131.0,
                                    security_definition: SecurityDefinition::Tgbt,
                                    price: 11.71,
                                },
                                Change {
                                    change_type: ChangeType::Security,
                                    size: 1585.0,
                                    security_definition: SecurityDefinition::Tcbt,
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
