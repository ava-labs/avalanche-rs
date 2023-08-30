use std::{env::args, io};

use avalanche_types::evm::eip712::gsn;

/// "registerRequestType(string typeName, string typeSuffix)" "my name" "my suffix"
/// cargo run --example evm_eip712_gsn_request_type_hash --features="evm" -- "my name" "my suffix"
/// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol>
fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let type_name = args().nth(1).expect("no type_name given");
    log::info!("type_name: {type_name}");

    let type_suffix_data = args().nth(2).expect("no type_suffix_data given");
    log::info!("type_suffix_data: {type_suffix_data}");

    let request_type_hash = gsn::compute_request_type_hash(&type_name, &type_suffix_data);
    log::info!("request type hash: {:x}", request_type_hash);

    Ok(())
}
