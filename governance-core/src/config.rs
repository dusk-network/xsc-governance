use crate::models::Address;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use toml_base_config::BaseConfig;

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    pub mnemonic: String,
    pub contract_id: [u8; 32],
    pub rusk_address: String,
    pub prover_address: String,
    pub sender_index: u64,
    #[serde(deserialize_with = "to_key", serialize_with = "from_key")]
    pub refund: Address,
    pub gas_limit: u64,
    pub gas_price: u64,
}

fn to_key<'de, D>(deserializer: D) -> Result<Address, D::Error>
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

fn from_key<S>(x: &Address, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encode = bs58::encode(x.0).into_string();

    s.serialize_str(&encode)
}

impl BaseConfig for Config {
    const PACKAGE: &'static str = env!("CARGO_PKG_NAME");
}
