pub mod config;
pub mod models;

use governance_contract::TX_TRANSFER;
pub use models::*;

use crate::config::Config;

use std::fmt::Debug;

use canonical::{Canon, EncodeToVec};
use dusk_abi::ContractId;
use dusk_bls12_381::BlsScalar;
use dusk_bytes::Serializable;
use dusk_pki::PublicSpendKey;
use dusk_wallet::WalletPath;
use dusk_wallet::{SecureWalletFile, TransportTCP, Wallet};
use dusk_wallet_core::{ProverClient, StateClient, Store};
use rand::rngs::ThreadRng;
use toml_base_config::BaseConfig;
use tracing::info;

/// Data we send to the blockchain
#[derive(Debug)]
pub struct Governance<C> {
    pub scalars: Vec<BlsScalar>,
    pub data: C,
    pub seed: BlsScalar,
}

#[derive(Debug)]
pub struct SecureWallet {
    pub path: WalletPath,
    pub pwd: String,
}

pub async fn send_data<C>(
    wallet: SecureWallet,
    contract: Governance<C>,
) -> Result<(), dusk_wallet::Error>
where
    C: Canon + Debug,
{
    let Config {
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

    let scalar_bytes: Vec<u8> = contract.scalars.iter().flat_map(|e| e.to_bits()).collect();

    let signature = sec_key.sign(&pub_key, &scalar_bytes);
    let mut data = Vec::new();

    data.push(TX_TRANSFER);
    data.extend(contract.seed.to_bytes());
    data.extend(signature.to_bytes());
    data.extend(contract.data.encode_to_vec());

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

pub async fn send<C, S, SC, PC>(
    data: C,
    core_wallet: &dusk_wallet_core::Wallet<S, SC, PC>,
    contract_id: ContractId,
    sender_index: u64,
    refund: Address,
    gas_limit: u64,
    gas_price: u64,
) -> Result<(), dusk_wallet::Error>
where
    C: Canon,
    S: Store,
    SC: StateClient,
    PC: ProverClient,
    dusk_wallet::Error: From<dusk_wallet_core::Error<S, SC, PC>>,
{
    let mut thread_rng = ThreadRng::default();
    let refund = PublicSpendKey::from_bytes(&refund.0)?;

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

impl SecureWalletFile for SecureWallet {
    fn path(&self) -> &WalletPath {
        &self.path
    }

    fn pwd(&self) -> blake3::Hash {
        blake3::hash(self.pwd.as_bytes())
    }
}
