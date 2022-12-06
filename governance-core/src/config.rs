use crate::models::Address;

use dusk_abi::ContractId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use toml_base_config::BaseConfig;

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(deserialize_with = "to_contract", serialize_with = "from_contract")]
    pub contract_id: ContractId,
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

fn from_contract<S>(x: &ContractId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encode = x.as_bytes();

    s.serialize_bytes(encode)
}

fn to_contract<'de, D>(deserializer: D) -> Result<ContractId, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &[u8] = Deserialize::deserialize(deserializer)?;

    let mut buffer = [0; 32];

    buffer.copy_from_slice(&s[0..32]);

    Ok(ContractId::from_raw(buffer))
}

impl BaseConfig for Config {
    const PACKAGE: &'static str = env!("CARGO_PKG_NAME");
}
