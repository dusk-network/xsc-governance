// read json data and make sense of it
pub mod json;
// config of the backend
pub mod config;
// types/models for the backend
pub mod models;

pub mod prelude {
    pub use crate::config::{Config, SecureWallet};
    pub use crate::json::*;
    pub use crate::models::*;
    pub use crate::Governance;
}

use crate::prelude::*;

use blake2::{digest::consts::U32, Digest};
use canonical::{Canon, EncodeToVec, Sink};
use dusk_abi::ContractId;
use dusk_bls12_381::BlsScalar;
use dusk_bls12_381_sign::{PublicKey as BlsPublicKey, SecretKey as BlsSecretKey, Signature};
use dusk_bytes::Serializable;
use dusk_wallet::{gas::Gas, TransportTCP, Wallet};
use toml_base_config::BaseConfig;
use tracing::info;

pub const TX_TRANSFER: u8 = 0x04;
pub const TX_FEE: u8 = 0x05;

pub struct Governance {
    config: Config,
    wallet: SecureWallet,
}

impl Governance {
    pub fn new(wallet: SecureWallet) -> Result<Self, dusk_wallet::Error> {
        Ok(Self {
            config: Config::load()?,
            wallet,
        })
    }

    // set a custom config
    pub fn with_config(&mut self, config: Config) {
        self.config = config;
    }

    /// Data we send to the blockchain
    pub async fn send_data(self, data: TransferMap) -> Result<(), dusk_wallet::Error> {
        let Config {
            rusk_address,
            prover_address,
            gas_limit,
            gas_price,
        } = self.config;

        let mut wallet = Wallet::from_file(self.wallet)?;
        let (_, sec_key) = wallet.provisioner_keys(wallet.default_address())?;
        let transport_tcp = TransportTCP::new(rusk_address, prover_address);

        wallet
            .connect_with_status(transport_tcp, |s| info!("Status: {}", s))
            .await?;

        assert!(!wallet.is_online(), "Wallet is not online");

        for (contract, (transfers, fees)) in data.transfers() {
            // get contract_id from security
            let contract_id = ContractId::reserved(contract as u8);

            if !transfers.is_empty() {
                let data = signed_payload(&sec_key, (seed(&transfers), TX_TRANSFER, transfers));

                send(data, &wallet, contract_id, gas_limit, gas_price).await?;
            };

            if !fees.is_empty() {
                let data = signed_payload(&sec_key, (seed(&fees), TX_FEE, fees));

                send(data, &wallet, contract_id, gas_limit, gas_price).await?;
            }
        }

        Err(dusk_wallet::Error::WalletFileMissing)
    }
}

// send to the blockchain
async fn send<C>(
    data: C,
    wallet: &Wallet<SecureWallet>,
    contract_id: ContractId,
    gas_limit: u64,
    gas_price: Option<u64>,
) -> Result<(), dusk_wallet::Error>
where
    C: Canon,
{
    // TODO: Make sure this is correct
    let sender = wallet.default_address();

    let mut gas = Gas::new(gas_limit);
    gas.set_price(gas_price);

    // finish sending data to blockchain
    wallet.execute(sender, contract_id, data, gas).await?;

    Ok(())
}

// sign the payload before sending to the blockchain
fn signed_payload<C>(sk: &BlsSecretKey, payload: C) -> (Signature, u32, C)
where
    C: Canon,
{
    let payload_len = payload.encoded_len();
    let capacity = payload_len + (payload_len as u32).encoded_len();
    let len_u32 = capacity as u32;
    let mut buffer = vec![0; capacity];

    let mut sink = Sink::new(&mut buffer);
    len_u32.encode(&mut sink);
    payload.encode(&mut sink);

    let pk = BlsPublicKey::from(sk);
    let signature = sk.sign(&pk, &buffer);

    (signature, len_u32, payload)
}

// generate seed for Transfer
fn seed(data: &Vec<Transfer>) -> BlsScalar {
    let msg = data.encode_to_vec();
    let mut digest: [u8; BlsScalar::SIZE] = blake2::Blake2b::<U32>::digest(msg).into();

    // Truncate the contract id to fit bls
    digest[31] &= 0x3f;

    BlsScalar::from_bytes(&digest).unwrap_or_default()
}
