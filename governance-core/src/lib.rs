pub mod config;
pub mod csv_parser;
pub mod models;

pub use csv_parser::CsvParser;
pub use models::*;

use crate::config::Config;

use std::fmt::Debug;

use canonical::Canon;
use dusk_abi::ContractId;
use dusk_bytes::Serializable;
use dusk_pki::PublicSpendKey;
use dusk_wallet::{SecureWalletFile, TransportTCP, Wallet};
use rand::rngs::ThreadRng;
use toml_base_config::BaseConfig;
use tracing::info;

pub async fn send<F, T>(data: T) -> Result<(), dusk_wallet::Error>
where
    T: Canon,
    F: Debug + SecureWalletFile,
{
    let config = Config::load()?;
    let mut wallet = Wallet::<F>::new(config.mnemonic)?;
    let transport_tcp = TransportTCP::new(config.rusk_address, config.prover_address);

    wallet
        .connect_with_status(transport_tcp, |s| info!("Status: {}", s))
        .await?;

    if let Some(core_wallet) = wallet.get_wallet() {
        let mut thread_rng = ThreadRng::default();
        let contract_id = ContractId::from_raw(config.contract_id);
        let refund = PublicSpendKey::from_bytes(&config.refund.0)?;

        core_wallet.execute(
            &mut thread_rng,
            contract_id,
            data,
            config.sender_index,
            &refund,
            config.gas_limit,
            config.gas_price,
        )?;
    }

    Ok(())
}
