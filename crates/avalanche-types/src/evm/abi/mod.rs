//! The EVM ABI.
#![allow(deprecated)]
use std::io::{self, Error, ErrorKind};

use ethers_core::abi::{Function, Token};

/// ref. <https://github.com/foundry-rs/foundry/blob/master/common/src/abi.rs> "encode_args"
pub fn encode_calldata(func: Function, arg_tokens: &[Token]) -> io::Result<Vec<u8>> {
    // ref. "abi.encodeWithSelector"
    func.encode_input(arg_tokens)
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed to encode_input {}", e)))
}

/// TODO: implement this with "foundry 4-byte decode"
/// ref. <https://github.com/foundry-rs/foundry/blob/master/common/src/selectors.rs> "decode_calldata"
/// ref. <sig.eth.samczsun.com>
/// ref. <https://tools.deth.net/calldata-decoder>
pub fn decode_calldata(calldata: &str) -> io::Result<(Function, Vec<Token>)> {
    let calldata = calldata.strip_prefix("0x").unwrap_or(calldata);
    if calldata.len() < 8 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "calldata too short: expected at least 8 characters (excluding 0x prefix), got {}.",
                calldata.len()
            ),
        ));
    }

    // TODO
    unimplemented!("not yet")
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib --features="evm" -- evm::abi::test_encode_calldata_register_name --exact --show-output
#[test]
fn test_encode_calldata_register_name() {
    use ethers_core::abi::{Function, Param, ParamType, StateMutability, Token};

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
    let calldata = encode_calldata(func, &arg_tokens).unwrap();
    log::info!("calldata: 0x{}", hex::encode(calldata));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib --features="evm" -- evm::abi::test_encode_calldata_register_mint --exact --show-output
#[test]
fn test_encode_calldata_register_mint() {
    use std::str::FromStr;

    use ethers_core::{
        abi::{Function, Param, ParamType, StateMutability, Token},
        types::{H160, U256},
    };

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    // parsed function of "mint(address receiver, uint amount)"
    let func = Function {
        name: "mint".to_string(),
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
            H160::from_str("0x8db97c7cece249c2b98bdc0226cc4c2a57bf52fc".trim_start_matches("0x"))
                .unwrap(),
        ),
        Token::Uint(U256::from_str_radix("0x12345", 16).unwrap()),
    ];
    let calldata = encode_calldata(func, &arg_tokens).unwrap();
    log::info!("calldata: 0x{}", hex::encode(calldata));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib --features="evm" -- evm::abi::test_encode_calldata_send --exact --show-output
#[test]
fn test_encode_calldata_send() {
    use std::str::FromStr;

    use ethers_core::{
        abi::{Function, Param, ParamType, StateMutability, Token},
        types::{H160, U256},
    };

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

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
    let calldata = encode_calldata(func, &arg_tokens).unwrap();
    log::info!("calldata: 0x{}", hex::encode(calldata));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib --features="evm" -- evm::abi::test_encode_calldata_forward_request --exact --show-output
#[test]
fn test_encode_calldata_forward_request() {
    use ethers_core::{
        abi::{Function, Param, ParamType, StateMutability, Token},
        types::{H160, H256, U256},
    };

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    // parsed function of "execute((address,address,uint256,uint256,uint256,bytes,uint256) req,bytes32 domainSeparator,bytes32 requestTypeHash,bytes suffixData,bytes sig) (bool success, bytes memory ret)"
    let func = Function {
        name: "execute".to_string(),
        inputs: vec![
            Param {
                name: "req".to_string(),
                kind: ParamType::Tuple(vec![
                    ParamType::Address,   // "from"
                    ParamType::Address,   // "to"
                    ParamType::Uint(256), // "value"
                    ParamType::Uint(256), // "gas"
                    ParamType::Uint(256), // "nonce"
                    ParamType::Bytes,     // "data"
                    ParamType::Uint(256), // "validUntilTime"
                ]),
                internal_type: None,
            },
            Param {
                name: "domainSeparator".to_string(),
                kind: ParamType::FixedBytes(32),
                internal_type: None,
            },
            Param {
                name: "requestTypeHash".to_string(),
                kind: ParamType::FixedBytes(32),
                internal_type: None,
            },
            Param {
                name: "suffixData".to_string(),
                kind: ParamType::Bytes,
                internal_type: None,
            },
            Param {
                name: "sig".to_string(),
                kind: ParamType::Bytes,
                internal_type: None,
            },
        ],
        outputs: vec![
            Param {
                name: "success".to_string(),
                kind: ParamType::Bool,
                internal_type: None,
            },
            Param {
                name: "ret".to_string(),
                kind: ParamType::Bytes,
                internal_type: None,
            },
        ],
        constant: None,
        state_mutability: StateMutability::NonPayable,
    };
    let arg_tokens = vec![
        Token::Tuple(vec![
            Token::Address(H160::random()),
            Token::Address(H160::random()),
            Token::Uint(U256::from(123)),
            Token::Uint(U256::from(123)),
            Token::Uint(U256::from(123)),
            Token::Bytes(vec![1, 2, 3]),
            Token::Uint(U256::MAX),
        ]),
        Token::FixedBytes(H256::random().as_fixed_bytes().to_vec()),
        Token::FixedBytes(H256::random().as_fixed_bytes().to_vec()),
        Token::Bytes(vec![1, 2, 3]),
        Token::Bytes(vec![1, 2, 3]),
    ];
    let calldata = encode_calldata(func, &arg_tokens).unwrap();
    log::info!("calldata: 0x{}", hex::encode(calldata));
}
