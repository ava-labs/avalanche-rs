//! Wallets for Avalanche.
pub mod p;
pub mod x;

#[cfg(feature = "wallet_evm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wallet_evm")))]
pub mod evm;

use std::{
    fmt,
    sync::{Arc, Mutex},
};

use crate::{
    errors::Result,
    ids::{self, short},
    jsonrpc::client::{info as api_info, x as api_x},
    key, utils,
};

#[derive(Debug, Clone)]
pub struct Wallet<T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone> {
    pub key_type: key::secp256k1::KeyType,
    pub keychain: key::secp256k1::keychain::Keychain<T>,

    /// Base HTTP URLs without RPC endpoint path.
    pub base_http_urls: Vec<String>,
    pub base_http_url_cursor: Arc<Mutex<usize>>, // to roundrobin

    pub network_id: u32,
    pub network_name: String,

    pub x_address: String,
    pub p_address: String,
    pub short_address: short::Id,
    pub eth_address: String,
    pub h160_address: primitive_types::H160,

    pub blockchain_id_x: ids::Id,
    pub blockchain_id_p: ids::Id,

    pub avax_asset_id: ids::Id,

    /// Fee that is burned by every non-state creating transaction.
    pub tx_fee: u64,
    /// Transaction fee for adding a primary network validator.
    pub add_primary_network_validator_fee: u64,
    /// Transaction fee to create a new subnet.
    pub create_subnet_tx_fee: u64,
    /// Transaction fee to create a new blockchain.
    pub create_blockchain_tx_fee: u64,
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl<T> fmt::Display for Wallet<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "key_type: {}", self.key_type.as_str())?;
        writeln!(f, "http_rpcs: {:?}", self.base_http_urls)?;
        writeln!(f, "network_id: {}", self.network_id)?;
        writeln!(f, "network_name: {}", self.network_name)?;

        writeln!(f, "x_address: {}", self.x_address)?;
        writeln!(f, "p_address: {}", self.p_address)?;
        writeln!(f, "short_address: {}", self.short_address)?;
        writeln!(f, "eth_address: {}", self.eth_address)?;
        writeln!(f, "h160_address: {}", self.h160_address)?;

        writeln!(f, "blockchain_id_x: {}", self.blockchain_id_x)?;
        writeln!(f, "blockchain_id_p: {}", self.blockchain_id_p)?;

        writeln!(f, "avax_asset_id: {}", self.avax_asset_id)?;

        writeln!(f, "tx_fee: {}", self.tx_fee)?;
        writeln!(
            f,
            "add_primary_network_validator_fee: {}",
            self.add_primary_network_validator_fee
        )?;
        writeln!(f, "create_subnet_tx_fee: {}", self.create_subnet_tx_fee)?;
        writeln!(
            f,
            "create_blockchain_tx_fee: {}",
            self.create_blockchain_tx_fee
        )
    }
}

impl<T> Wallet<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
{
    /// Picks one endpoint in roundrobin, and updates the cursor for next calls.
    /// Returns the pair of an index and its corresponding endpoint.
    pub fn pick_base_http_url(&self) -> (usize, String) {
        let mut idx = self.base_http_url_cursor.lock().unwrap();

        let picked = *idx;
        let http_rpc = self.base_http_urls[picked].clone();
        *idx = (picked + 1) % self.base_http_urls.len();

        log::debug!("picked base http URL {http_rpc} at index {picked}");
        (picked, http_rpc)
    }
}

#[derive(Debug, Clone)]
pub struct Builder<T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone> {
    pub key: T,
    pub base_http_urls: Vec<String>,
    pub only_evm: bool,
}

impl<T> Builder<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
{
    pub fn new(key: &T) -> Self {
        Self {
            key: key.clone(),
            base_http_urls: Vec::new(),
            only_evm: false,
        }
    }

    /// Adds an HTTP rpc endpoint to the `http_rpcs` field in the Builder.
    /// If URL path is specified, it strips the URL path.
    #[must_use]
    pub fn base_http_url(mut self, u: String) -> Self {
        let (scheme, host, port, _, _) =
            utils::urls::extract_scheme_host_port_path_chain_alias(&u).unwrap();
        let scheme = if let Some(s) = scheme {
            format!("{s}://")
        } else {
            String::new()
        };
        let rpc_ep = format!("{scheme}{host}");
        let rpc_url = if let Some(port) = port {
            format!("{rpc_ep}:{port}")
        } else {
            rpc_ep.clone() // e.g., DNS
        };

        if self.base_http_urls.is_empty() {
            self.base_http_urls = vec![rpc_url];
        } else {
            self.base_http_urls.push(rpc_url);
        }
        self
    }

    #[must_use]
    pub fn only_evm(mut self) -> Self {
        self.only_evm = true;
        self
    }

    /// Overwrites the HTTP rpc endpoints to the `urls` field in the Builder.
    /// If URL path is specified, it strips the URL path.
    #[must_use]
    pub fn base_http_urls(mut self, urls: Vec<String>) -> Self {
        let mut base_http_urls = Vec::new();
        for http_rpc in urls.iter() {
            let (scheme, host, port, _, _) =
                utils::urls::extract_scheme_host_port_path_chain_alias(http_rpc).unwrap();
            let scheme = if let Some(s) = scheme {
                format!("{s}://")
            } else {
                String::new()
            };
            let rpc_ep = format!("{scheme}{host}");
            let rpc_url = if let Some(port) = port {
                format!("{rpc_ep}:{port}")
            } else {
                rpc_ep.clone() // e.g., DNS
            };

            base_http_urls.push(rpc_url);
        }
        self.base_http_urls = base_http_urls;
        self
    }

    pub async fn build(&self) -> Result<Wallet<T>> {
        log::info!(
            "building wallet with {} endpoints",
            self.base_http_urls.len()
        );

        let keychain = key::secp256k1::keychain::Keychain::new(vec![self.key.clone()]);
        let h160_address = keychain.keys[0].h160_address();

        let (
            network_id,
            network_name,
            blockchain_id_x,
            blockchain_id_p,
            avax_asset_id,
            tx_fee,
            create_subnet_tx_fee,
            create_blockchain_tx_fee,
        ) = if self.only_evm {
            log::warn!("wallet is only used for EVM thus skipping querying info API");
            (
                0,
                String::new(),
                ids::Id::empty(),
                ids::Id::empty(),
                ids::Id::empty(),
                0,
                0,
                0,
            )
        } else {
            let resp = api_info::get_network_id(&self.base_http_urls[0]).await?;
            let network_id = resp.result.unwrap().network_id;
            let resp = api_info::get_network_name(&self.base_http_urls[0]).await?;
            let network_name = resp.result.unwrap().network_name;

            let resp = api_info::get_blockchain_id(&self.base_http_urls[0], "X").await?;
            let blockchain_id_x = resp.result.unwrap().blockchain_id;

            let resp = api_info::get_blockchain_id(&self.base_http_urls[0], "P").await?;
            let blockchain_id_p = resp.result.unwrap().blockchain_id;

            let resp = api_x::get_asset_description(&self.base_http_urls[0], "AVAX").await?;
            let resp = resp
                .result
                .expect("unexpected None GetAssetDescriptionResult");
            let avax_asset_id = resp.asset_id;

            let resp = api_info::get_tx_fee(&self.base_http_urls[0]).await?;
            let get_tx_fee_result = resp.result.unwrap();
            let tx_fee = get_tx_fee_result.tx_fee;
            let create_subnet_tx_fee = get_tx_fee_result.create_subnet_tx_fee;
            let create_blockchain_tx_fee = get_tx_fee_result.create_blockchain_tx_fee;

            (
                network_id,
                network_name,
                blockchain_id_x,
                blockchain_id_p,
                avax_asset_id,
                tx_fee,
                create_subnet_tx_fee,
                create_blockchain_tx_fee,
            )
        };

        let w = Wallet {
            key_type: self.key.key_type(),
            keychain,

            base_http_urls: self.base_http_urls.clone(),
            base_http_url_cursor: Arc::new(Mutex::new(0)),

            network_id,
            network_name,

            x_address: self.key.hrp_address(network_id, "X").unwrap(),
            p_address: self.key.hrp_address(network_id, "P").unwrap(),
            short_address: self.key.short_address().unwrap(),
            eth_address: self.key.eth_address(),
            h160_address,

            blockchain_id_x,
            blockchain_id_p,

            avax_asset_id,

            tx_fee,
            add_primary_network_validator_fee: ADD_PRIMARY_NETWORK_VALIDATOR_FEE,
            create_subnet_tx_fee,
            create_blockchain_tx_fee,
        };
        log::info!("initiated the wallet:\n{}", w);

        Ok(w)
    }
}

/// ref. <https://docs.avax.network/learn/platform-overview/transaction-fees/#fee-schedule>
pub const ADD_PRIMARY_NETWORK_VALIDATOR_FEE: u64 = 0;
