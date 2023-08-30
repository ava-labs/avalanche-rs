#![allow(deprecated)]

use std::{io, str::FromStr};

use avalanche_types::evm::abi;
use ethers_core::{
    abi::{Function, Param, ParamType, StateMutability, Token},
    types::{H160, U256},
};

/// cargo run --example evm_contract_abi_calldata --features="evm"
fn main() -> io::Result<()> {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    // parsed function of "register(string name)"
    let func = Function {
        name: "register".to_string(),
        inputs: vec![Param {
            name: "name".to_string(),
            kind: ParamType::String,
            internal_type: None,
        }],
        outputs: Vec::new(),
        constant: None,
        state_mutability: StateMutability::NonPayable,
    };
    let arg_tokens = vec![Token::String("abc".to_string())];
    let calldata = abi::encode_calldata(func, &arg_tokens).unwrap();
    log::info!("calldata: 0x{}", hex::encode(calldata));

    // parsed function of "send(address receiver, uint amount)"
    let func = Function {
        name: "send".to_string(),
        inputs: vec![
            Param {
                name: "receiver".to_string(),
                kind: ParamType::Address,
                internal_type: None,
            },
            Param {
                name: "amount".to_string(),
                kind: ParamType::Uint(256),
                internal_type: None,
            },
        ],
        outputs: Vec::new(),
        constant: None,
        state_mutability: StateMutability::NonPayable,
    };
    let arg_tokens = vec![
        Token::Address(
            H160::from_str("0x53C62F5d19f94556c4e9E9Ee97CeE274AB053399".trim_start_matches("0x"))
                .unwrap(),
        ),
        Token::Uint(U256::from(1)),
    ];
    let calldata = abi::encode_calldata(func, &arg_tokens).unwrap();
    log::info!("calldata: 0x{}", hex::encode(calldata));

    Ok(())
}
