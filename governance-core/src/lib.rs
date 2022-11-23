pub mod config;
pub mod models;

pub use models::*;

use crate::config::Config;

use std::fmt::Debug;

use canonical::Canon;
use dusk_abi::ContractId;
use dusk_bytes::Serializable;
use dusk_pki::{PublicKey, PublicSpendKey};
use dusk_wallet::{SecureWalletFile, TransportTCP, Wallet};
use dusk_wallet_core::{ProverClient, StateClient, Store};
use rand::rngs::ThreadRng;
use toml_base_config::BaseConfig;
use tracing::info;

/// Sock implementation for SecureWallet because we cannot construct a
/// dusk_wallet::Wallet instance without an `T: SecureWalletFile`
#[derive(Debug)]
struct SecureWallet {}

pub async fn send_call<C, F>(call: F) -> Result<(), dusk_wallet::Error>
where
    C: Canon,
    F: Fn(PublicKey, PublicKey) -> C,
{
    let Config {
        mnemonic,
        contract_id,
        rusk_address,
        prover_address,
        sender_index,
        refund,
        gas_limit,
        gas_price,
    } = Config::load()?;

    let mut wallet = Wallet::<SecureWallet>::new(mnemonic)?;
    let transport_tcp = TransportTCP::new(rusk_address, prover_address);

    wallet
        .connect_with_status(transport_tcp, |s| info!("Status: {}", s))
        .await?;

    if let Some(core_wallet) = wallet.get_wallet() {
        // TODO: Decide caller and signature
        let caller = PublicKey::from_bytes(&Default::default())?;
        let signature = PublicKey::from_bytes(&Default::default())?;

        send(
            call(caller, signature),
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

// Both methods are unreachable because Wallet requires a F: SecureWalletFile to
// be passed when creating an instance even without a wallet file. These methods
// will not be called otherwise.
impl SecureWalletFile for SecureWallet {
    fn path(&self) -> &dusk_wallet::WalletPath {
        unreachable!()
    }

    fn pwd(&self) -> blake3::Hash {
        unreachable!()
    }
}
