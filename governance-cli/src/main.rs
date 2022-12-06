use std::error::Error;
use std::iter;
use std::path::PathBuf;

use blake2::{digest::consts::U32, Digest};
use canonical::EncodeToVec;
use clap::Parser;
use dusk_bls12_381::BlsScalar;
use dusk_bytes::Serializable;
use dusk_wallet::WalletPath;
use governance_contract::TX_TRANSFER;
use governance_core::{models::*, send_data, Governance, SecureWallet};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    /// Directory to store user data [default: `$HOME/.dusk/rusk-wallet`]
    #[clap(short, long)]
    profile: PathBuf,
    /// Set the password for wallet's creation
    #[clap(long, env = "RUSK_WALLET_PWD")]
    pub password: String,
}

type Blake2b = blake2::Blake2b<U32>;

fn h(msg: &[u8]) -> BlsScalar {
    let mut digest: [u8; BlsScalar::SIZE] = Blake2b::digest(msg).into();

    // Truncate the contract id to fit bls
    digest[31] &= 0x3f;

    BlsScalar::from_bytes(&digest).unwrap_or_default()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();

    let data = json_file("../assets/data.json")?;
    let chained = data.iter().flat_map(|t| t.as_scalars());
    let bytes = data.encode_to_vec();

    let seed = h(&bytes);

    let scalars: Vec<BlsScalar> = iter::once([seed, BlsScalar::from(TX_TRANSFER as u64)])
        .flatten()
        .chain(chained)
        .collect();

    let contract = Governance {
        scalars,
        data,
        seed,
    };

    let wallet = SecureWallet {
        pwd: cli.password,
        path: WalletPath::from(PathBuf::from(concat!(
            env!("RUSK_PROFILE_PATH"),
            "/wallet.dat"
        ))),
    };

    let bytes = send_data(wallet, contract).await?;

    Ok(())
}
