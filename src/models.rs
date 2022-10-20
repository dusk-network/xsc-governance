use std::io;

use csv::StringRecord;
use serde::{Deserialize, Deserializer};
use tai64::Tai64;

type Address = [u8; 128];

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

impl TryFrom<StringRecord> for Whitelist {
    type Error = io::Error;

    fn try_from(record: StringRecord) -> Result<Self, Self::Error> {
        let record = record.into_byte_record();
        let variant = record.get(0);
        let value = record.get(1);

        let mut buffer: Address = [0; 128];

        match (variant, value) {
            (Some(b"add"), Some(x)) => {
                bs58::decode(x).into(&mut buffer).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("base58 convert error for {:?}", x),
                    )
                })?;

                Ok(Whitelist::Add { address: buffer })
            }
            (Some(b"remove"), Some(x)) => {
                bs58::decode(x).into(&mut buffer).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("base58 convert error for {:?}", x),
                    )
                })?;

                Ok(Whitelist::Remove { address: buffer })
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
    let mut buffer: Address = [0; 128];

    bs58::decode(s)
        .into(&mut buffer)
        .map_err(D::Error::custom)?;

    Ok(buffer)
}

fn to_tai64<'de, D>(deserializer: D) -> Result<Tai64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: u64 = Deserialize::deserialize(deserializer)?;

    Ok(Tai64(s))
}
