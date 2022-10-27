pub mod activity;
pub mod whitelist;

pub use activity::{Activity, ActivityCall};
pub use whitelist::{Whitelist, WhitelistCall};

use core::mem::size_of;

use serde::Deserializer;

#[derive(Debug, Clone, Copy)]
pub struct Address(pub [u8; 64]);

impl Address {
    pub fn buffer() -> [u8; size_of::<Address>()] {
        [0; size_of::<Address>()]
    }
}

impl Default for Address {
    fn default() -> Self {
        Self(Self::buffer())
    }
}
