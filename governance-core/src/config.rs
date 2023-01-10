use dusk_wallet::{SecureWalletFile, WalletPath};
use serde::{Deserialize, Serialize};
use toml_base_config::BaseConfig;

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    pub rusk_address: String,
    pub prover_address: String,
    pub gas_limit: u64,
    pub gas_price: Option<u64>,
}

#[derive(Debug)]
pub struct SecureWallet {
    pub path: WalletPath,
    pub pwd: String,
}

impl SecureWalletFile for SecureWallet {
    fn path(&self) -> &WalletPath {
        &self.path
    }

    fn pwd(&self) -> blake3::Hash {
        blake3::hash(self.pwd.as_bytes())
    }
}

impl BaseConfig for Config {
    const PACKAGE: &'static str = env!("CARGO_PKG_NAME");
}
