mod args;

use crate::args::Args;

use std::error::Error;

use clap::Parser;
use dusk_wallet::WalletPath;
use governance_core::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();

    let data = json_file("../assets/data.json")?;
    WalletPath::set_cache_dir(&cli.profile)?;
    let wallet_path = WalletPath::from(cli.profile.as_path().join("wallet.dat"));
    let config_path = cli.profile.as_path().join("gov_config.toml");

    let wallet = SecureWallet {
        pwd: cli.password,
        path: wallet_path,
    };

    let contract = Governance::new(wallet, config_path)?;

    contract.send_data(data).await?;

    Ok(())
}
