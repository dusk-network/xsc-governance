// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

pub mod events;
pub mod transfer;

pub use self::events::*;
pub use self::transfer::*;

use std::collections::HashMap;
/// Type of the hashmap we use to store our Transfers
/// The first element of the tuple is the deposit transfers and the second is the fee transfers
pub type TxHashMap = HashMap<SecurityDefinition, (Vec<Transfer>, Vec<Transfer>)>;
/// List of transfers we send to the blockchain
#[derive(Debug, Default)]
pub struct TransferMap {
    transfers: TxHashMap,
}

impl TransferMap {
    pub fn insert_tx(&mut self, security: SecurityDefinition, tx: Transfer) {
        if let Some((vec, _)) = self.transfers.get_mut(&security) {
            vec.push(tx)
        } else {
            self.transfers.insert(security, (vec![tx], vec![]));
        }
    }

    pub fn insert_fee(&mut self, security: SecurityDefinition, tx: Transfer) {
        if let Some((_, vec)) = self.transfers.get_mut(&security) {
            vec.push(tx)
        } else {
            self.transfers.insert(security, (vec![], vec![tx]));
        }
    }

    pub fn into_transfers(self) -> TxHashMap {
        self.transfers
    }
}
