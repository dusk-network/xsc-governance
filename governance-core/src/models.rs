use core::mem::size_of;

use std::io;

use canonical::{Canon, CanonError, Sink, Source};
use csv::StringRecord;
use serde::{Deserialize, Deserializer};
use tai64::Tai64;

#[derive(Debug, Clone, Copy)]
pub struct Address(pub [u8; 64]);

#[derive(Debug, Clone)]
pub struct ActivityCall {
    pub caller: Address,
    pub signature: Address,
    pub count: usize,
    pub activities: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct WhitelistCall {
    pub caller: Address,
    pub signature: Address,
    pub count: usize,
    pub whitelist: Vec<u8>,
}

#[derive(Deserialize, Debug)]
pub struct Activity {
    #[serde(deserialize_with = "to_base58")]
    pub sender: Address,
    #[serde(deserialize_with = "to_base58")]
    pub buyer: Address,
    pub amount: u64,
    #[serde(deserialize_with = "to_tai64")]
    pub timestamp: Tai64,
}

#[derive(Debug)]
pub enum Whitelist {
    Add { address: Address },
    Remove { address: Address },
    None,
}

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

impl ActivityCall {
    const OPERATION_IDENTIFIER: u8 = 0x00;
}

impl WhitelistCall {
    const OPERATION_IDENTIFIER: u8 = 0x01;
}

impl From<Activity> for [u8; size_of::<Activity>()] {
    fn from(activity: Activity) -> Self {
        let Activity {
            sender,
            buyer,
            amount,
            timestamp,
        } = activity;

        let mut buffer = [0u8; size_of::<Activity>()];

        buffer[0..64].copy_from_slice(&sender.0);
        buffer[64..128].copy_from_slice(&buyer.0);
        buffer[128..136].copy_from_slice(&amount.to_le_bytes());
        buffer[136..].copy_from_slice(&timestamp.0.to_le_bytes());

        buffer
    }
}

impl From<Whitelist> for [u8; size_of::<Activity>()] {
    fn from(whitelist: Whitelist) -> Self {
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

impl Canon for ActivityCall {
    fn encode(&self, sink: &mut Sink<'_>) {
        let mut bytes = [0; size_of::<ActivityCall>()];

        bytes[0..64].copy_from_slice(&self.caller.0);
        bytes[64..128].copy_from_slice(&self.signature.0);
        bytes[128..129].copy_from_slice(&Self::OPERATION_IDENTIFIER.to_le_bytes());
        bytes[129..137].copy_from_slice(&self.count.to_le_bytes());
        bytes[137..].copy_from_slice(&self.activities);

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
        let mut activities = Vec::new();

        caller.copy_from_slice(&all_bytes[0..64]);
        signature.copy_from_slice(&all_bytes[64..128]);
        count.copy_from_slice(&all_bytes[129..137]);
        activities.copy_from_slice(&all_bytes[137..]);

        Ok(Self {
            caller: Address(caller),
            signature: Address(signature),
            count: usize::from_le_bytes(count),
            activities,
        })
    }

    fn encoded_len(&self) -> usize {
        137 + self.activities.len()
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

fn to_base58<'de, D>(deserializer: D) -> Result<Address, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let s: &str = Deserialize::deserialize(deserializer)?;
    let mut buffer = Address::buffer();

    bs58::decode(s)
        .into(&mut buffer)
        .map_err(D::Error::custom)?;

    Ok(Address(buffer))
}

fn to_tai64<'de, D>(deserializer: D) -> Result<Tai64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: u64 = Deserialize::deserialize(deserializer)?;

    Ok(Tai64(s))
}
