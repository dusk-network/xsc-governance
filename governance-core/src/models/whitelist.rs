use crate::activity::Activity;
use crate::Address;

use std::io;
use std::mem::size_of;

use canonical::{Canon, CanonError, Sink, Source};
use csv::StringRecord;

#[derive(Debug, Clone)]
pub struct WhitelistCall {
    pub caller: Address,
    pub signature: Address,
    pub count: usize,
    pub whitelist: Vec<u8>,
}

#[derive(Debug)]
pub enum Whitelist {
    Add { address: Address },
    Remove { address: Address },
    None,
}

impl WhitelistCall {
    const OPERATION_IDENTIFIER: u8 = 0x01;
}

impl<'a> From<&'a Whitelist> for [u8; size_of::<Activity>()] {
    fn from(whitelist: &'a Whitelist) -> Self {
        let mut buffer = [0u8; size_of::<Activity>()];

        match whitelist {
            Whitelist::Add { address } => {
                buffer[0..1].copy_from_slice(&0_i32.to_le_bytes());
                buffer[1..].copy_from_slice(&address.0);
            }
            Whitelist::Remove { address } => {
                buffer[0..1].copy_from_slice(&1_i32.to_le_bytes());
                buffer[1..].copy_from_slice(&address.0);
            }
            _ => (),
        }

        buffer
    }
}

impl TryFrom<StringRecord> for Whitelist {
    type Error = io::Error;

    fn try_from(record: StringRecord) -> Result<Self, Self::Error> {
        let record = record.into_byte_record();
        let variant = record.get(0);
        let value = record.get(1);

        let mut buffer = [0; size_of::<Address>()];

        match (variant, value) {
            (Some(b"add"), Some(x)) => {
                bs58::decode(x).into(&mut buffer).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("base58 convert error for {:?}", x),
                    )
                })?;

                Ok(Whitelist::Add {
                    address: Address(buffer),
                })
            }
            (Some(b"remove"), Some(x)) => {
                bs58::decode(x).into(&mut buffer).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("base58 convert error for {:?}", x),
                    )
                })?;

                Ok(Whitelist::Remove {
                    address: Address(buffer),
                })
            }
            _ => Ok(Whitelist::None),
        }
    }
}

impl Canon for WhitelistCall {
    fn encode(&self, sink: &mut Sink<'_>) {
        let mut bytes = [0; size_of::<WhitelistCall>()];

        bytes[0..64].copy_from_slice(&self.caller.0);
        bytes[64..128].copy_from_slice(&self.signature.0);
        bytes[128..129].copy_from_slice(&Self::OPERATION_IDENTIFIER.to_le_bytes());
        bytes[129..137].copy_from_slice(&self.count.to_le_bytes());
        bytes[137..].copy_from_slice(&self.whitelist);

        sink.copy_bytes(&bytes)
    }

    fn decode(source: &mut Source<'_>) -> Result<Self, CanonError> {
        let all_bytes: [u8; size_of::<Self>()] = source
            .read_bytes(size_of::<Self>())
            .try_into()
            .map_err(|_| CanonError::InvalidEncoding)?;

        let mut caller = [0; 64];
        let mut signature = [0; 64];
        let mut count = [0; 8];
        let mut whitelist = Vec::new();

        caller.copy_from_slice(&all_bytes[0..64]);
        signature.copy_from_slice(&all_bytes[64..128]);
        count.copy_from_slice(&all_bytes[129..137]);
        whitelist.copy_from_slice(&all_bytes[137..]);

        Ok(Self {
            caller: Address(caller),
            signature: Address(signature),
            count: usize::from_le_bytes(count),
            whitelist,
        })
    }

    fn encoded_len(&self) -> usize {
        137 + self.whitelist.len()
    }
}
