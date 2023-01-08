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
use dusk_pki::PublicSpendKey;
use dusk_wallet::{TransportTCP, Wallet};
use dusk_wallet_core::{ProverClient, StateClient, Store};
use rand::rngs::ThreadRng;
use toml_base_config::BaseConfig;
use tracing::info;

pub const TX_TRANSFER: u8 = 0x06;

type Blake2b = blake2::Blake2b<U32>;

/// Data we send to the blockchain
#[derive(Debug)]
pub struct Governance {
    pub scalars: Vec<BlsScalar>,
    pub data: Vec<Transfer>,
    pub seed: BlsScalar,
}

impl Governance {
    pub fn new(data: Vec<Transfer>) -> Self {
        // seed generation
        let msg = data.encode_to_vec();

        let mut digest: [u8; BlsScalar::SIZE] = Blake2b::digest(msg).into();

        // Truncate the contract id to fit bls
        digest[31] &= 0x3f;

        let seed = BlsScalar::from_bytes(&digest).unwrap_or_default();

        let chained = data.iter().flat_map(|t| t.as_scalars());

        let scalars: Vec<BlsScalar> = iter::once([seed, BlsScalar::from(TX_TRANSFER as u64)])
            .flatten()
            .chain(chained)
            .collect();

        Self {
            data,
            scalars,
            seed,
        }
    }

    pub async fn send_data(&self, wallet: SecureWallet) -> Result<(), dusk_wallet::Error> {
        let Config {
            // we load contract_id from config for now
            contract_id,
            rusk_address,
            prover_address,
            sender_index,
            refund,
            gas_limit,
            gas_price,
        } = Config::load()?;

        let mut wallet = Wallet::from_file(wallet)?;
        let (pub_key, sec_key) = wallet.provisioner_keys(wallet.default_address())?;

        let scalar_bytes: Vec<u8> = self.scalars.iter().flat_map(|e| e.to_bits()).collect();

        let signature = sec_key.sign(&pub_key, &scalar_bytes);
        let mut data = Vec::new();

        data.push(TX_TRANSFER);
        data.extend(self.seed.to_bytes());
        data.extend(signature.to_bytes());
        data.extend(self.data.encode_to_vec());

        let transport_tcp = TransportTCP::new(rusk_address, prover_address);

        wallet
            .connect_with_status(transport_tcp, |s| info!("Status: {}", s))
            .await?;

        if let Some(core_wallet) = wallet.get_wallet() {
            send(
                data,
                core_wallet,
                contract_id,
                sender_index,
                refund,
                gas_limit,
                gas_price,
            )
            .await?;
        }

        Err(dusk_wallet::Error::WalletFileMissing)
    }
}

pub async fn send<S, SC, PC>(
    data: Vec<u8>,
    core_wallet: &dusk_wallet_core::Wallet<S, SC, PC>,
    contract_id: ContractId,
    sender_index: u64,
    refund: Address,
    gas_limit: u64,
    gas_price: u64,
) -> Result<(), dusk_wallet::Error>
where
    S: Store,
    SC: StateClient,
    PC: ProverClient,
    dusk_wallet::Error: From<dusk_wallet_core::Error<S, SC, PC>>,
{
    let mut thread_rng = ThreadRng::default();
    let refund = PublicSpendKey::from_bytes(&refund.0)?;

    // finish sending data to blockchain
    core_wallet.execute(
        &mut thread_rng,
        contract_id,
        data,
        sender_index,
        &refund,
        gas_limit,
        gas_price,
    )?;

    Ok(())
}
