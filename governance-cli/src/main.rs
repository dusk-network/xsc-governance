mod args;

use crate::args::Args;

use std::error::Error;
use std::path::PathBuf;

use clap::Parser;
use dusk_wallet::WalletPath;
use governance_core::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();

    let data = json_file("../assets/data.json")?;

    let wallet = SecureWallet {
        pwd: cli.password,
        path: WalletPath::from(PathBuf::from(concat!(
            env!("RUSK_PROFILE_PATH"),
            "/wallet.dat"
        ))),
    };

    let contract = Governance::new(wallet)?;

    contract.send_data(data).await?;

    Ok(())
}
