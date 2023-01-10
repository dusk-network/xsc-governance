#[cfg(test)]
mod tests;

pub mod address;
pub mod events;
pub mod transfer;

pub use self::address::*;
pub use self::events::*;
pub use self::transfer::*;

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Error as IoError, ErrorKind as IoErrorKind, Read};
use std::path::Path;

use dusk_pki::{PublicKey, SecretKey};
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json::Value;

pub type Transfers = HashMap<SecurityDefinition, Vec<Transfer>>;

/// Parse a json file, convert them to Vec<Transfer>
pub fn json_file<T: AsRef<Path>>(path: T) -> io::Result<Transfers> {
    let mut data = String::new();
    let f = File::open(path.as_ref())?;

    let mut reader = BufReader::new(f);
    reader.read_to_string(&mut data)?;

    json_bytes(data.as_bytes())
}

/// Parse raw json bytes convert them to Vec<Transfer>
pub fn json_bytes<T: AsRef<[u8]>>(bytes: T) -> io::Result<Transfers> {
    let json: Value = serde_json::from_slice(bytes.as_ref())?;

    if let Value::Object(obj) = json {
        let mut account = obj.into_iter();

        if let Some((account_name, events)) = account.next() {
            let events: Events = serde_json::from_value(events)?;
            let mut transfers: Transfers = HashMap::new();

            for event in events.events {
                let timestamp = event.occurrence;

                match event.cause {
                    Cause::Deposit => {
                        let from = public_key(&account_name);

                        for change in event.changes {
                            let to = public_key(change.security_definition.to_string());
                            let amount = change.size;

                            let vec = transfers.get_mut(&SecurityDefinition::Cash).unwrap();

                            vec.push(Transfer {
                                from,
                                to,
                                amount,
                                timestamp,
                            })
                        }
                    }
                    // TODO: Finish other causes
                    Cause::Rebalance => (),
                    Cause::Withdraw => (),
                    Cause::Fee => (),
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
