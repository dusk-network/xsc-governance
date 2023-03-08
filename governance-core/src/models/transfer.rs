// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical_derive::Canon;
use dusk_pki::PublicKey;

// TODO: The same struct exists in the governance contract, do we just import that?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Canon)]
pub struct Transfer {
    pub to: Option<PublicKey>,
    pub from: Option<PublicKey>,
    pub amount: u64,
    pub timestamp: u64,
}

impl Transfer {
    pub fn new(amount: f32, timestamp: u64) -> Self {
        let amount = float2fixed(amount);

        Self {
            to: None,
            from: None,
            amount,
            timestamp,
        }
    }

    pub fn amount(&mut self, amount: f32) -> Self {
        self.amount = float2fixed(amount);

        *self
    }

    pub fn withdraw(&mut self, from: PublicKey) -> Self {
        self.from = Some(from);
        self.to = None;

        *self
    }

    pub fn deposit(&mut self, to: PublicKey) -> Self {
        self.to = Some(to);
        self.from = None;

        *self
    }
}

fn float2fixed(x: f32) -> u64 {
    // 2^32 - 1 = 4_294_967_295
    (x * 4_294_967_295.0) as u64
}
