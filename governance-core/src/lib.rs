pub mod config;
pub mod models;

pub mod prelude {
    pub use crate::config::{Config, SecureWallet};
    pub use crate::models::*;
    pub use crate::Governance;
}

use crate::prelude::*;

use std::fmt::Debug;
use std::iter;

use blake2::{digest::consts::U32, Digest};
use canonical::EncodeToVec;
use dusk_abi::ContractId;
use dusk_bls12_381::BlsScalar;
use dusk_bytes::Serializable;
use dusk_wallet::{TransportTCP,gas::Gas, Wallet};
use toml_base_config::BaseConfig;
use tracing::info;

pub const TX_TRANSFER: u8 = 0x06;

type Blake2b = blake2::Blake2b<U32>;

/// Data we send to the blockchain
#[derive(Debug)]
pub struct Governance {
    pub data: Transfers,
}

impl Governance {
    pub fn seed(data: &Vec<Transfer>) -> BlsScalar {
        let msg = data.encode_to_vec();
        let mut digest: [u8; BlsScalar::SIZE] = Blake2b::digest(msg).into();

        // Truncate the contract id to fit bls
        digest[31] &= 0x3f;

        BlsScalar::from_bytes(&digest).unwrap_or_default()
    }

    pub fn scalars(data: &[Transfer], seed: BlsScalar) -> Vec<BlsScalar> {
        let chained = data.iter().flat_map(|t| t.as_scalars());

        iter::once([seed, BlsScalar::from(TX_TRANSFER as u64)])
            .flatten()
            .chain(chained)
            .collect()
    }

    pub fn new(data: Transfers) -> Self {
        Self { data }
    }

    pub async fn send_data(self, wallet: SecureWallet) -> Result<(), dusk_wallet::Error> {
        let Config {
            rusk_address,
            prover_address,
            gas_limit,
            gas_price,
        } = Config::load()?;

        let mut wallet = Wallet::from_file(wallet)?;
        let (pub_key, sec_key) = wallet.provisioner_keys(wallet.default_address())?;
        let transport_tcp = TransportTCP::new(rusk_address, prover_address);

        wallet
            .connect_with_status(transport_tcp, |s| info!("Status: {}", s))
            .await?;

        for (contract, transfer) in self.data {
            let mut gas = Gas::new(gas_limit);

            if let Some(gas_price) = gas_price {
                gas.set_price(gas_price);
            }

            let contract_id = ContractId::reserved(contract as u8);
            let seed = Self::seed(&transfer);
            let scalars = Self::scalars(&transfer, seed);

            let scalar_bytes: Vec<u8> = scalars.iter().flat_map(|e| e.to_bits()).collect();
            let signature = sec_key.sign(&pub_key, &scalar_bytes);

            let mut data = Vec::new();

            data.push(TX_TRANSFER);
            data.extend(seed.to_bytes());
            data.extend(signature.to_bytes());
            data.extend(transfer.encode_to_vec());

            if wallet.is_online() {
                    send(
                        data,
                        &wallet,
                        contract_id,
                        gas
                        
                    ).await?;
            }
        }

        Err(dusk_wallet::Error::WalletFileMissing)
    }
}

pub async fn send(
    data: Vec<u8>,
    wallet: &Wallet<SecureWallet>,
    contract_id: ContractId,
    gas: Gas,
) -> Result<(), dusk_wallet::Error>
{
    // TODO: Make sure this is correct
    let sender = wallet.default_address();

    // finish sending data to blockchain
    wallet.execute(
        sender,
        contract_id,
        data,
        gas
    ).await?;

    Ok(())
}
