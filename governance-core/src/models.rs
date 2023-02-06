#[cfg(test)]
mod tests;

pub mod events;
pub mod transfer;

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

/// Type of the hashmap we use to store our Transfers
type TransfersMap = HashMap<SecurityDefinition, Vec<Transfer>>;
/// List of transfers we send to the blockchain
#[derive(Debug)]
pub struct Transfers {
    transfers: TransfersMap,
    from: PublicKey,
}

impl Transfers {
    pub fn new(from: PublicKey) -> Self {
        Self {
            transfers: HashMap::new(),
            from,
        }
    }

    pub fn insert_rebalance(&mut self, security: SecurityDefinition, amount: f32, timestamp: u64) {
        if amount < 0.0 {
            self.insert_withdraw(security, -amount, timestamp);
        } else {
            self.insert_deposit(security, amount, timestamp);
        }
    }

    pub fn insert_deposit(&mut self, security: SecurityDefinition, amount: f32, timestamp: u64) {
        let amount = float2fixed(amount);

        if let Some(vec) = self.transfers.get_mut(&security) {
            vec.push(Transfer {
                from: None,
                to: Some(security.to_public_key()),
                amount,
                timestamp,
            })
        }
    }

    pub fn insert_withdraw(&mut self, security: SecurityDefinition, amount: f32, timestamp: u64) {
        let amount = float2fixed(amount);

        if let Some(vec) = self.transfers.get_mut(&security) {
            vec.push(Transfer {
                from: Some(self.from),
                to: None,
                amount,
                timestamp,
            })
        }
    }

    pub fn insert_fee(&mut self, amount: f32, timestamp: u64) {
        let amount = float2fixed(amount);

        if let Some(vec) = self.transfers.get_mut(&SecurityDefinition::Cash) {
            vec.push(Transfer {
                from: Some(self.from),
                // TODO: Add the broker here
                to: Default::default(),
                amount,
                timestamp,
            })
        }
    }

    pub fn transfers(self) -> TransfersMap {
        self.transfers
    }
}

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
        // the account name and all events are the first key value pairs
        if let Some((account_name, events)) = obj.into_iter().next() {
            let events: Events = serde_json::from_value(events)?;
            let from = public_key(account_name);
            // Transfers holds all our transfers
            let mut transfers: Transfers = Transfers::new(from);

            events.events.into_iter().for_each(|event| {
                let timestamp = event.occurrence;

                event.changes.into_iter().for_each(|change| {
                    let amount = change.size;

                    let security = change.security_definition;

                    match event.cause {
                        Cause::Rebalance => transfers.insert_rebalance(security, amount, timestamp),
                        Cause::Deposit => transfers.insert_deposit(security, amount, timestamp),
                        Cause::Withdraw => transfers.insert_withdraw(security, amount, timestamp),
                        Cause::Fee => transfers.insert_fee(amount, timestamp),
                    }
                })
            });

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

fn float2fixed(x: f32) -> u64 {
    (x * 4_294_967_295.0) as u64
}
