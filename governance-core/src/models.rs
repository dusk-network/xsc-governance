pub mod events;
pub mod transfer;

pub use self::events::*;
pub use self::transfer::*;

use std::collections::HashMap;
/// Type of the hashmap we use to store our Transfers
/// The first element of the tuple is the deposit transfers and the second is the fee transfers
pub type TxHashMap = HashMap<SecurityDefinition, (Vec<Transfer>, Vec<Transfer>)>;
/// List of transfers we send to the blockchain
#[derive(Debug)]
pub struct TransferMap {
    transfers: TxHashMap,
    pub security: SecurityDefinition,
}

impl TransferMap {
    pub fn new() -> Self {
        Self {
            transfers: HashMap::new(),
            // None by default, we need to set it manually
            security: SecurityDefinition::None,
        }
    }

    pub fn insert_tx(&mut self, tx: Transfer) {
        if let Some((vec, _)) = self.transfers.get_mut(&self.security) {
            vec.push(tx)
        }
    }

    pub fn insert_fee(&mut self, tx: Transfer) {
        if let Some((_, vec)) = self.transfers.get_mut(&self.security) {
            vec.push(tx)
        }
    }

    pub fn transfers(self) -> TxHashMap {
        self.transfers
    }
}
