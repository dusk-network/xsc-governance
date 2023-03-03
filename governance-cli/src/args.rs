use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    /// Directory to store user data [default: `$HOME/.dusk/rusk-wallet`]
    #[clap(short, long)]
    pub profile: PathBuf,
    /// Set the password for wallet's creation
    #[clap(long, env = "RUSK_WALLET_PWD")]
    pub password: String,
}
