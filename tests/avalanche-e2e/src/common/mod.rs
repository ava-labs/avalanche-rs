use std::{ops::Div, str::FromStr};

use avalanche_types::{
    errors::Result,
    jsonrpc::client::{evm as avalanche_sdk_evm, p as avalanche_sdk_p, x as avalanche_sdk_x},
    key, units,
};
use rand::{seq::SliceRandom, thread_rng};

pub struct KeyInfosWithBalance {
    pub key_infos: Vec<key::secp256k1::Info>,

    pub x_addrs: Vec<String>,
    pub x_balances: Vec<u64>,

    pub p_addrs: Vec<String>,
    pub p_balances: Vec<u64>,

    pub c_addrs: Vec<String>,
    pub c_balances: Vec<primitive_types::U256>,
}

/// Load the signing hot keys and fetch their balances.
/// TODO: parallelize fetch
pub async fn load_keys_with_balance(
    key_infos: Vec<key::secp256k1::Info>,
    permute_keys: bool,
    network_id: u32,
    rpc_ep: &str,
) -> Result<KeyInfosWithBalance> {
    let mut cloned_key_infos = key_infos.clone();
    if permute_keys {
        cloned_key_infos.shuffle(&mut thread_rng());
    };

    let mut x_addrs: Vec<String> = Vec::new();
    let mut x_balances: Vec<u64> = Vec::new();

    let mut p_addrs: Vec<String> = Vec::new();
    let mut p_balances: Vec<u64> = Vec::new();

    let mut c_addrs: Vec<String> = Vec::new();
    let mut c_balances: Vec<primitive_types::U256> = Vec::new();

    for k in cloned_key_infos.iter() {
        let x_addr = k.addresses.get(&network_id).unwrap().x.clone();
        let resp = avalanche_sdk_x::get_balance(rpc_ep, &x_addr).await?;
        let x_bal = resp.result.unwrap().balance;

        x_addrs.push(x_addr.clone());
        x_balances.push(x_bal);

        let p_addr = k.addresses.get(&network_id).unwrap().p.clone();
        let resp = avalanche_sdk_p::get_balance(rpc_ep, &p_addr).await?;
        let p_bal = resp.result.unwrap().balance;

        p_addrs.push(p_addr.clone());
        p_balances.push(p_bal);

        let c_addr = k.eth_address.clone();
        let c_bal = avalanche_sdk_evm::get_balance(
            format!("{rpc_ep}/ext/bc/C/rpc").as_str(),
            primitive_types::H160::from_str(c_addr.trim_start_matches("0x")).unwrap(),
        )
        .await?;

        c_addrs.push(c_addr.clone());
        c_balances.push(c_bal);

        // On the X-Chain, one AVAX is 10^9  units.
        // On the P-Chain, one AVAX is 10^9  units.
        // On the C-Chain, one AVAX is 10^18 units.
        // ref. https://snowtrace.io/unitconverter
        log::debug!("'{}' X-chain balance: {} AVAX", x_addr, x_bal / units::AVAX);
        log::debug!("'{}' P-chain balance: {} AVAX", p_addr, p_bal / units::AVAX);
        log::debug!(
            "'{}' C-chain balance: {:?} AVAX",
            c_addr,
            c_bal.div(units::AVAX_EVM_CHAIN)
        );
    }

    Ok(KeyInfosWithBalance {
        key_infos: cloned_key_infos,

        x_addrs,
        x_balances,

        p_addrs,
        p_balances,

        c_addrs,
        c_balances,
    })
}
