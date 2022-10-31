use std::io;
use std::mem::size_of;

use canonical::{Canon, CanonError, Sink, Source};
use csv::StringRecord;
use dusk_bytes::Serializable;
use dusk_pki::PublicKey;

#[derive(Debug, Clone)]
pub struct WhitelistCall {
    pub caller: PublicKey,
    pub signature: PublicKey,
    pub count: usize,
    pub whitelist: Vec<u8>,
}

#[derive(Debug)]
pub enum Whitelist {
    Add { address: PublicKey },
    Remove { address: PublicKey },
    None,
}

impl WhitelistCall {
    const OPERATION_IDENTIFIER: u8 = 0x01;
}

impl<'a> From<&'a Whitelist> for [u8; size_of::<Whitelist>()] {
    fn from(whitelist: &'a Whitelist) -> Self {
        let mut buffer = [0u8; size_of::<Whitelist>()];

        match whitelist {
            Whitelist::Add { address } => {
                buffer[0..1].copy_from_slice(&0_i32.to_le_bytes());
                buffer[1..].copy_from_slice(&address.to_bytes());
            }
            Whitelist::Remove { address } => {
                buffer[0..1].copy_from_slice(&1_i32.to_le_bytes());
                buffer[1..].copy_from_slice(&address.to_bytes());
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

        let mut buffer = [0; 32];

        match (variant, value) {
            (Some(b"add"), Some(x)) => {
                bs58::decode(x).into(&mut buffer).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("base58 convert error for {:?}", x),
                    )
                })?;

                Ok(Whitelist::Add {
                    address: PublicKey::from_bytes(&buffer).map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Cannot convert bytes to public key {:?}", buffer),
                        )
                    })?,
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
                    address: PublicKey::from_bytes(&buffer).map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Cannot convert bytes to public key {:?}", buffer),
                        )
                    })?,
                })
            }
            _ => Ok(Whitelist::None),
        }
    }
}

/// TODO: Fuzz this
impl Canon for WhitelistCall {
    fn encode(&self, sink: &mut Sink<'_>) {
        let mut bytes = Vec::new();

        bytes[0..32].copy_from_slice(&self.caller.to_bytes());
        bytes[32..64].copy_from_slice(&self.signature.to_bytes());
        bytes[64..65].copy_from_slice(&Self::OPERATION_IDENTIFIER.to_le_bytes());
        bytes[65..73].copy_from_slice(&self.count.to_le_bytes());
        bytes[73..].copy_from_slice(&self.whitelist);

        sink.copy_bytes(&bytes)
    }

    fn decode(source: &mut Source<'_>) -> Result<Self, CanonError> {
        let all_bytes: [u8; size_of::<Self>()] = source
            .read_bytes(size_of::<Self>())
            .try_into()
            .map_err(|_| CanonError::InvalidEncoding)?;

        let mut caller = [0; 32];
        let mut signature = [0; 32];
        let mut count = [0; 8];
        let mut whitelist = Vec::new();

        caller.copy_from_slice(&all_bytes[0..32]);
        signature.copy_from_slice(&all_bytes[32..64]);
        count.copy_from_slice(&all_bytes[65..73]);
        whitelist.copy_from_slice(&all_bytes[73..]);

        Ok(Self {
            caller: PublicKey::from_bytes(&caller).map_err(|_| CanonError::InvalidEncoding)?,
            signature: PublicKey::from_bytes(&signature)
                .map_err(|_| CanonError::InvalidEncoding)?,
            count: usize::from_le_bytes(count),
            whitelist,
        })
    }

    fn encoded_len(&self) -> usize {
        size_of::<Self>()
    }
}
