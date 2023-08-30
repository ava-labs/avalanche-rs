//! Foundry forge functions.
use std::io::{self, Error, ErrorKind};

use crate::codec::serde::{
    hex_0x_primitive_types_h160::Hex0xH160, hex_0x_primitive_types_h256::Hex0xH256,
};
use primitive_types::{H160, H256};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tokio::process::Command;

/// Runs "forge create" with a hotkey.
pub async fn create_with_hotkey(
    private_key: &str,
    rpc_url: &str,
    gas_price: u64,
    priority_gas_price: u64,
    contract_arg: &str,
    constructor_args: Option<Vec<String>>,
) -> io::Result<TxOutput> {
    log::info!("running forge create for the contract {contract_arg} via {rpc_url} (gas price {gas_price}, priority gas price {priority_gas_price})");

    let mut args = vec![
        "create".to_string(),
        "--json".to_string(),
        format!("--private-key={private_key}"),
        format!("--rpc-url={rpc_url}"),
        format!("--gas-price={gas_price}"),
        format!("--priority-gas-price={priority_gas_price}"),
        contract_arg.to_string(),
    ];
    if let Some(cargs) = constructor_args {
        args.push("--constructor-args".to_string());
        args.extend(cargs.to_vec());
    }

    let output = Command::new("forge").args(args).output().await?;

    if !output.status.success() {
        return Err(Error::new(ErrorKind::Other, "forge create status failed"));
    }

    serde_json::from_slice(&output.stdout).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed serde_json::from_slice '{}'", e),
        )
    })
}

/// Represents "forge create" or "cast send" output.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TxOutput {
    #[serde_as(as = "Hex0xH160")]
    pub deployer: H160,
    #[serde_as(as = "Hex0xH160")]
    pub deployed_to: H160,
    #[serde_as(as = "Hex0xH256")]
    pub transaction_hash: H256,
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- evm::foundry::forge::test_tx_output --exact --show-output
#[test]
fn test_tx_output() {
    use crate::key::secp256k1::address::h160_to_eth_address;

    let _ = env_logger::builder().is_test(true).try_init();

    let json_decoded: TxOutput = serde_json::from_str(
        "

{
    \"deployer\":\"0x8db97C7cEcE249c2b98bDC0226Cc4C2A57BF52FC\",
    \"deployedTo\":\"0x52C84043CD9c865236f11d9Fc9F56aa003c1f922\",
    \"transactionHash\":\"0x50c415005599eb1f53256d6f2f5edc275b4dc3b046c2320ffa7cd436585e2da2\"
}

",
    )
    .unwrap();

    assert_eq!(
        h160_to_eth_address(&json_decoded.deployer, None),
        "0x8db97C7cEcE249c2b98bDC0226Cc4C2A57BF52FC"
    );
    assert_eq!(
        h160_to_eth_address(&json_decoded.deployed_to, None),
        "0x52C84043CD9c865236f11d9Fc9F56aa003c1f922"
    );
    assert_eq!(
        format!("0x{:x}", json_decoded.transaction_hash),
        "0x50c415005599eb1f53256d6f2f5edc275b4dc3b046c2320ffa7cd436585e2da2"
    );

    log::info!("tx output decoded: {:?}", json_decoded);
}
