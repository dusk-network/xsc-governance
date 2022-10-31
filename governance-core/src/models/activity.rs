use crate::models::*;

use std::mem::size_of;

use canonical::{Canon, CanonError, Sink, Source};
use dusk_bytes::Serializable;
use dusk_pki::PublicKey;
use serde::Deserialize;
use tai64::Tai64;

#[derive(Deserialize, Debug, Clone)]
pub struct Activity {
    #[serde(deserialize_with = "to_base58")]
    pub sender: PublicKey,
    #[serde(deserialize_with = "to_base58")]
    pub buyer: PublicKey,
    pub amount: u64,
    #[serde(deserialize_with = "to_tai64")]
    pub timestamp: Tai64,
}

#[derive(Debug, Clone)]
pub struct ActivityCall {
    pub caller: PublicKey,
    pub signature: PublicKey,
    pub count: usize,
    pub activities: Vec<u8>,
}

impl ActivityCall {
    const OPERATION_IDENTIFIER: u8 = 0x00;
}

// Converts a Vec<Activity> to [u8; x]
impl<'a> From<&'a Activity> for [u8; size_of::<Activity>()] {
    fn from(activity: &'a Activity) -> Self {
        let Activity {
            sender,
            buyer,
            amount,
            timestamp,
        } = activity;

        let mut buffer = [0u8; size_of::<Activity>()];

        buffer[0..32].copy_from_slice(&sender.to_bytes());
        buffer[32..64].copy_from_slice(&buyer.to_bytes());
        buffer[64..72].copy_from_slice(&amount.to_le_bytes());
        buffer[72..].copy_from_slice(&timestamp.0.to_le_bytes());

        buffer
    }
}

impl Canon for ActivityCall {
    fn encode(&self, sink: &mut Sink<'_>) {
        let mut bytes = [0; size_of::<ActivityCall>()];

        bytes[0..32].copy_from_slice(&self.caller.to_bytes());
        bytes[32..64].copy_from_slice(&self.signature.to_bytes());
        bytes[64..65].copy_from_slice(&Self::OPERATION_IDENTIFIER.to_le_bytes());
        bytes[65..73].copy_from_slice(&self.count.to_le_bytes());
        bytes[73..].copy_from_slice(&self.activities);

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
        let mut activities = Vec::new();

        caller.copy_from_slice(&all_bytes[0..32]);
        signature.copy_from_slice(&all_bytes[32..64]);
        count.copy_from_slice(&all_bytes[65..73]);
        activities.copy_from_slice(&all_bytes[73..]);

        Ok(Self {
            caller: PublicKey::from_bytes(&caller).map_err(|_| CanonError::InvalidEncoding)?,
            signature: PublicKey::from_bytes(&signature)
                .map_err(|_| CanonError::InvalidEncoding)?,
            count: usize::from_le_bytes(count),
            activities,
        })
    }

    fn encoded_len(&self) -> usize {
        size_of::<ActivityCall>()
    }
}

fn to_base58<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let s: &str = Deserialize::deserialize(deserializer)?;
    let mut buffer = [0; 32];

    bs58::decode(s)
        .into(&mut buffer)
        .map_err(D::Error::custom)?;

    PublicKey::from_bytes(&buffer).map_err(|e| D::Error::custom(format!("{:?}", e)))
}

fn to_tai64<'de, D>(deserializer: D) -> Result<Tai64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: u64 = Deserialize::deserialize(deserializer)?;

    Ok(Tai64(s))
}
