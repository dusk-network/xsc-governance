// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

// read json data and make sense of it
pub mod json;
// config of the backend
pub mod config;
// types/models for the backend
pub mod models;

// transaction propagation confirmation
mod gql;

pub mod prelude {
    pub use crate::config::{Config, SecureWallet};
    pub use crate::gql::GraphQL;
    pub use crate::json::*;
    pub use crate::models::*;
    pub use crate::Governance;
}

use std::path::PathBuf;

use crate::prelude::*;

use blake2::{digest::consts::U32, Digest};
use canonical::{Canon, EncodeToVec, Sink};
use dusk_abi::ContractId;
use dusk_bls12_381::BlsScalar;
use dusk_bls12_381_sign::{
    PublicKey as BlsPublicKey, SecretKey as BlsSecretKey, Signature,
};
use dusk_bytes::Serializable;
use dusk_wallet::{gas::Gas, TransportTCP, Wallet};
use toml_base_config::BaseConfig;
use tracing::info;

pub const TX_TRANSFER: u8 = 0x04;
pub const TX_FEE: u8 = 0x05;

pub struct Governance {
    config: Config,
    wallet: SecureWallet,
}

impl Governance {
    // Create a new Governance instance, loading the config from the file
    pub fn new(
        wallet: SecureWallet,
        config: PathBuf,
    ) -> Result<Self, dusk_wallet::Error> {
        Ok(Self {
            config: Config::load_path(config)?,
            wallet,
        })
    }

    // Set a custom config, by default it loads from the config.toml file
    pub fn with_config(&mut self, config: Config) {
        self.config = config;
    }

    /// Data we send to the blockchain
    pub async fn send_data(self, data: TransferMap) -> anyhow::Result<()> {
        let Self {
            wallet,
            config:
                Config {
                    rusk_address,
                    prover_address,
                    graphql_address,
                    gas_limit,
                    gas_price,
                },
        } = self;

        let mut wallet = Wallet::from_file(wallet)?;
        let (_, sec_key) = wallet.provisioner_keys(wallet.default_address())?;
        let transport_tcp = TransportTCP::new(rusk_address, prover_address);

        wallet
            .connect_with_status(transport_tcp, |s| info!("Status: {}", s))
            .await?;

        assert!(wallet.is_online(), "Wallet is not online");
        let transfers = data.into_transfers();
        let gql = GraphQL::new(graphql_address, |s| info!("Status: {}", s));

        for (security, (transfers, fees)) in transfers {
            // get contract_id from security
            let contract_id = security.to_id();

            if !transfers.is_empty() {
                let payload = (seed(&transfers), TX_TRANSFER, transfers);
                let data = signed_payload(&sec_key, payload);

                let tx_hash =
                    send(data, &wallet, contract_id, gas_limit, gas_price)
                        .await?;
                let tx_id = format!("{:x}", tx_hash);
                gql.wait_for(&tx_id).await?;
            };

            if !fees.is_empty() {
                let payload = (seed(&fees), TX_FEE, fees);
                let data = signed_payload(&sec_key, payload);

                let tx_hash =
                    send(data, &wallet, contract_id, gas_limit, gas_price)
                        .await?;
                let tx_id = format!("{:x}", tx_hash);
                gql.wait_for(&tx_id).await?;
            }
        }
        Ok(())
    }
}

// send to the blockchain
async fn send<C>(
    data: C,
    wallet: &Wallet<SecureWallet>,
    contract_id: ContractId,
    gas_limit: u64,
    gas_price: Option<u64>,
) -> Result<BlsScalar, dusk_wallet::Error>
where
    C: Canon,
{
    // TODO: Make sure this is correct
    let sender = wallet.default_address();

    let mut gas = Gas::new(gas_limit);
    gas.set_price(gas_price);

    // finish sending data to blockchain
    let tx = wallet.execute(sender, contract_id, data, gas).await?;

    Ok(tx.hash())
}

// sign the payload before sending to the blockchain
fn signed_payload<C>(sk: &BlsSecretKey, payload: C) -> (Signature, u32, C)
where
    C: Canon,
{
    let payload_len = payload.encoded_len();
    let capacity = payload_len + (payload_len as u32).encoded_len();
    let len_u32 = capacity as u32;
    let mut buffer = vec![0; capacity];

    let mut sink = Sink::new(&mut buffer);
    len_u32.encode(&mut sink);
    payload.encode(&mut sink);

    let pk = BlsPublicKey::from(sk);
    let signature = sk.sign(&pk, &buffer);

    (signature, len_u32, payload)
}

// generate seed for Transfer
fn seed(data: &Vec<Transfer>) -> BlsScalar {
    let msg = data.encode_to_vec();
    let mut digest: [u8; BlsScalar::SIZE] =
        blake2::Blake2b::<U32>::digest(msg).into();

    // Truncate the contract id to fit bls
    digest[31] &= 0x3f;

    BlsScalar::from_bytes(&digest).unwrap_or_default()
}
