use core::mem::size_of;

use std::io;

use csv::StringRecord;
use serde::{Deserialize, Deserializer};
use tai64::Tai64;

#[derive(Debug)]
pub struct Address(pub [u8; 64]);

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
