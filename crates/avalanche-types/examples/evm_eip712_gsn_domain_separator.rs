use std::{env::args, io, str::FromStr};

use avalanche_types::evm::eip712::gsn::Tx;
use ethers_core::types::{H160, U256};

/// "registerDomainSeparator(string name, string version)" "my name" "1"
/// cargo run --example evm_eip712_gsn_domain_separator --features="evm" -- "my domain name" "1" 1234567 0x17aB05351fC94a1a67Bf3f56DdbB941aE6c63E25
/// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "registerDomainSeparator"
fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let domain_name = args().nth(1).expect("no domain_name given");
    log::info!("domain_name: {domain_name}");

    let domain_version = args().nth(2).expect("no domain_version given");
    log::info!("domain_version: {domain_version}");

    let domain_chain_id = args().nth(3).expect("no domain_chain_id given");
    let domain_chain_id = U256::from_str(&domain_chain_id).unwrap();
    log::info!("domain_chain_id: {domain_chain_id}");

    let domain_verifying_contract = args().nth(4).expect("no domain_verifying_contract given");
    let domain_verifying_contract =
        H160::from_str(&domain_verifying_contract.trim_start_matches("0x")).unwrap();
    log::info!("domain_verifying_contract: {domain_verifying_contract}");

    let domain_separator = Tx::new()
        .domain_name(domain_name)
        .domain_version(domain_version)
        .domain_chain_id(domain_chain_id)
        .domain_verifying_contract(domain_verifying_contract)
        .compute_domain_separator();
    log::info!("domain separator: 0x{:x}", domain_separator);

    Ok(())
}
