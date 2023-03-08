// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

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
    let wallet_path =
        WalletPath::from(cli.profile.as_path().join("wallet.dat"));
    let config_path = cli.profile.as_path().join("gov_config.toml");

    let wallet = SecureWallet {
        pwd: cli.password,
        path: wallet_path,
    };

    let contract = Governance::new(wallet, config_path)?;

    contract.send_data(data).await?;

    Ok(())
}
