#![allow(deprecated)]

pub mod relay;

use std::{collections::BTreeMap, io};

use crate::evm::abi as evm_abi;
use ethers_core::{
    abi::{Function, Param, ParamType, StateMutability, Token},
    types::{
        transaction::eip712::{
            EIP712Domain, Eip712, Eip712DomainType, Eip712Error, TypedData, Types,
        },
        H160, H256, U256,
    },
    utils::keccak256,
};

/// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "GENERIC_PARAMS"
pub const GENERIC_PARAMS: &str = "address from,address to,uint256 value,uint256 gas,uint256 nonce,bytes data,uint256 validUntilTime";

/// Implements the "Eip712" trait for GSN.
/// ref. <https://eips.ethereum.org/EIPS/eip-712>
/// ref. <https://eips.ethereum.org/EIPS/eip-2770>
/// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol>
/// ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/types/transaction/eip712.rs>
pub struct Tx {
    /// EIP-712 domain name.
    /// Used for domain separator hash.
    /// ref. "ethers_core::types::transaction::eip712::Eip712::domain_separator"
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol> "registerDomainSeparator"
    pub domain_name: String,
    /// EIP-712 domain version.
    /// Used for domain separator hash.
    /// ref. "ethers_core::types::transaction::eip712::Eip712::domain_separator"
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol> "registerDomainSeparator"
    pub domain_version: String,
    /// EIP-712 domain chain id.
    /// Used for domain separator hash.
    /// ref. "ethers_core::types::transaction::eip712::Eip712::domain_separator"
    pub domain_chain_id: U256,
    /// EIP-712 domain verifying contract name.
    /// Used for domain separator hash.
    /// Address of the contract that will verify the signature (e.g., trusted forwarder).
    /// ref. "ethers_core::types::transaction::eip712::Eip712::domain_separator"
    pub domain_verifying_contract: H160,

    /// Forward request "from" field.
    /// An externally-owned account making the request.
    /// ref. <https://eips.ethereum.org/EIPS/eip-2770>
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol> "ForwardRequest"
    pub from: H160,
    /// Forward request "to" field.
    /// A destination address, normally a smart-contract.
    /// ref. <https://eips.ethereum.org/EIPS/eip-2770>
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol> "ForwardRequest"
    pub to: H160,
    /// Forward request "value" field.
    /// An amount of Ether to transfer to the destination.
    /// ref. <https://eips.ethereum.org/EIPS/eip-2770>
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol> "ForwardRequest"
    pub value: U256,
    /// Forward request "gas" field.
    /// An amount of gas limit to set for the execution.
    /// When an externally owned account (EOA) signs the transaction, it must estimate the required gas
    /// to provide enough for its execution.
    /// ref. <https://eips.ethereum.org/EIPS/eip-2770>
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol> "ForwardRequest"
    pub gas: U256,
    /// Forward request "nonce" field.
    /// An on-chain tracked nonce of a transaction.
    /// ref. <https://eips.ethereum.org/EIPS/eip-2770>
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol> "ForwardRequest"
    pub nonce: U256,
    /// Forward request "data" field.
    /// The data to be sent to the destination (recipient contract).
    /// ref. <https://eips.ethereum.org/EIPS/eip-2770>
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol> "ForwardRequest"
    pub data: Vec<u8>,
    /// Forward request "validUntil" field.
    /// The highest block number the request can be forwarded in, or 0 if request validity is not time-limited.
    /// ref. <https://eips.ethereum.org/EIPS/eip-2770>
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol> "ForwardRequest"
    pub valid_until_time: U256,

    /// The name of the request type.
    /// Must match with the one used in "registerRequestType".
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "registerRequestType"
    pub type_name: String,
    /// The ABI-encoded extension data for the current `RequestType` used when signing this request.
    /// Must match with the one used in "registerRequestType".
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "registerRequestType"
    pub type_suffix_data: String,
}

impl Tx {
    pub fn new() -> Self {
        Self {
            domain_name: String::new(),
            domain_version: String::new(),
            domain_chain_id: U256::zero(),
            domain_verifying_contract: H160::zero(),

            from: H160::zero(),
            to: H160::zero(),
            value: U256::zero(),
            gas: U256::zero(),
            nonce: U256::zero(),
            data: Vec::new(),
            valid_until_time: U256::zero(),

            type_name: String::new(),
            type_suffix_data: String::new(),
        }
    }

    #[must_use]
    pub fn domain_name(mut self, domain_name: impl Into<String>) -> Self {
        self.domain_name = domain_name.into();
        self
    }

    #[must_use]
    pub fn domain_version(mut self, domain_version: impl Into<String>) -> Self {
        self.domain_version = domain_version.into();
        self
    }

    #[must_use]
    pub fn domain_chain_id(mut self, domain_chain_id: impl Into<U256>) -> Self {
        self.domain_chain_id = domain_chain_id.into();
        self
    }

    #[must_use]
    pub fn domain_verifying_contract(mut self, domain_verifying_contract: impl Into<H160>) -> Self {
        self.domain_verifying_contract = domain_verifying_contract.into();
        self
    }

    #[must_use]
    pub fn from(mut self, from: impl Into<H160>) -> Self {
        self.from = from.into();
        self
    }

    #[must_use]
    pub fn to(mut self, to: impl Into<H160>) -> Self {
        self.to = to.into();
        self
    }

    #[must_use]
    pub fn value(mut self, value: impl Into<U256>) -> Self {
        self.value = value.into();
        self
    }

    /// Fails if zero (e.g., "out of gas").
    #[must_use]
    pub fn gas(mut self, gas: impl Into<U256>) -> Self {
        self.gas = gas.into();
        self
    }

    #[must_use]
    pub fn nonce(mut self, nonce: impl Into<U256>) -> Self {
        self.nonce = nonce.into();
        self
    }

    #[must_use]
    pub fn data(mut self, data: impl Into<Vec<u8>>) -> Self {
        self.data = data.into();
        self
    }

    #[must_use]
    pub fn valid_until_time(mut self, valid_until_time: impl Into<U256>) -> Self {
        self.valid_until_time = valid_until_time.into();
        self
    }

    #[must_use]
    pub fn type_name(mut self, type_name: impl Into<String>) -> Self {
        self.type_name = type_name.into();
        self
    }

    #[must_use]
    pub fn type_suffix_data(mut self, type_suffix_data: impl Into<String>) -> Self {
        self.type_suffix_data = type_suffix_data.into();
        self
    }

    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "registerDomainSeparator"
    fn eip712_domain(&self) -> EIP712Domain {
        EIP712Domain {
            name: Some(self.domain_name.clone()),
            version: Some(self.domain_version.clone()),
            chain_id: Some(self.domain_chain_id),
            verifying_contract: Some(self.domain_verifying_contract),
            salt: None,
        }
    }

    /// Computes the domain separator hash.
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "registerDomainSeparator"
    pub fn compute_domain_separator(&self) -> H256 {
        H256(self.eip712_domain().separator())
    }

    /// Hash of the struct, according to EIP-712 definition of `hashStruct`.
    /// Implements "_getEncoded" and "_verifySig" in GSN Forwarder.sol.
    /// This method is used for "encode_eip712".
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "_getEncoded"
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "registerDomainSeparator"
    /// ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/types/transaction/eip712.rs> "EIP712Domain.separator"
    pub fn compute_struct_hash(&self) -> H256 {
        // "requestTypeHash"
        // Implements "type_hash".
        // ref. "ethers_core::types::transaction::eip712::EIP712_DOMAIN_TYPE_HASH"
        // ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "registerRequestType"
        // ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "registerRequestTypeInternal"
        let request_type_hash = compute_request_type_hash(&self.type_name, &self.type_suffix_data)
            .to_fixed_bytes()
            .to_vec();

        // "uint256(uint160(req.from))"
        // ref. <https://docs.soliditylang.org/en/v0.8.17/abi-spec.html>
        let from_bytes = self.from.to_fixed_bytes().to_vec();
        let from_bytes = U256::from(&from_bytes[..]);
        let mut from = [0u8; 32];
        from_bytes.to_big_endian(&mut from);

        // "uint256(uint160(req.to))"
        // ref. <https://docs.soliditylang.org/en/v0.8.17/abi-spec.html>
        let to_bytes = self.to.to_fixed_bytes().to_vec();
        let to_bytes = U256::from(&to_bytes[..]);
        let mut to = [0u8; 32];
        to_bytes.to_big_endian(&mut to);

        // "req.value"
        let mut value = [0u8; 32];
        self.value.to_big_endian(&mut value);

        // "req.gas"
        let mut gas = [0u8; 32];
        self.gas.to_big_endian(&mut gas);

        // "req.nonce"
        let mut nonce = [0u8; 32];
        self.nonce.to_big_endian(&mut nonce);

        // "keccak256(req.data)"
        let data = keccak256(self.data.clone());

        // "req.validUntilTime"
        let mut valid_until_time = [0u8; 32];
        self.valid_until_time.to_big_endian(&mut valid_until_time);

        // GSN "_getEncoded" appends suffixData as-is
        let type_suffix_data = self.type_suffix_data.as_bytes().to_vec();

        // solidity "abi.encodePacked" equals to string format/concatenation
        // solidity "abi.encode" equals to "ethabi::encode(&tokens)"
        // solidity "keccak256" equals to "ethers_core::utils::keccak256(&bytes)"
        // ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "_getEncoded"
        let digest_input = [
            &request_type_hash[..],
            &from[..],
            &to[..],
            &value[..],
            &gas[..],
            &nonce[..],
            &data[..],
            &valid_until_time[..],
            &type_suffix_data[..],
        ]
        .concat();

        // ref. "keccak256(_getEncoded(req, requestTypeHash, suffixData))"
        keccak256(digest_input).into()
    }

    /// Returns the calldata based on the arguments to the forwarder "execute" function.
    /// ref. "HumanReadableParser::parse_function"
    /// ref. "execute((address,address,uint256,uint256,uint256,bytes,uint256) req,bytes32 domainSeparator,bytes32 requestTypeHash,bytes suffixData,bytes sig) (bool success, bytes memory ret)"
    /// ref. ["(0x52C84043CD9c865236f11d9Fc9F56aa003c1f922,0x52C84043CD9c865236f11d9Fc9F56aa003c1f922,0,0,0,0x11,0)", "0x11", "0x11", "0x11", "0x11", "0x11"]
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol>
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol>
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/RelayHub.sol> "innerRelayCall"
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/utils/GsnEip712Library.sol> "execute"
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/utils/GsnTypes.sol> "GsnTypes"
    /// ref. <https://eips.ethereum.org/EIPS/eip-712>
    pub fn encode_execute_call(&self, sig: Vec<u8>) -> io::Result<Vec<u8>> {
        // Parsed function of "execute((address,address,uint256,uint256,uint256,bytes,uint256) req,bytes32 domainSeparator,bytes32 requestTypeHash,bytes suffixData,bytes sig) (bool success, bytes memory ret)".
        // ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol> "execute"
        // ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/abi/human_readable/mod.rs> "HumanReadableParser::parse_function"
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
            state_mutability: StateMutability::Payable,
        };

        // do not use "encode_args" from str
        // "LenientTokenizer::tokenize" cannot handle hex encode
        // "Uint parse error: InvalidCharacter"
        // ref. <https://github.com/foundry-rs/foundry/blob/master/common/src/abi.rs> "encode_args"
        let arg_tokens = vec![
            Token::Tuple(vec![
                Token::Address(self.from),
                Token::Address(self.to),
                Token::Uint(self.value),
                Token::Uint(self.gas),
                Token::Uint(self.nonce),
                Token::Bytes(self.data.clone()),
                Token::Uint(self.valid_until_time),
            ]),
            Token::FixedBytes(self.compute_domain_separator().as_bytes().to_vec()),
            Token::FixedBytes(
                compute_request_type_hash(&self.type_name, &self.type_suffix_data)
                    .as_bytes()
                    .to_vec(),
            ),
            Token::Bytes(self.type_suffix_data.as_bytes().to_vec()),
            Token::Bytes(sig),
        ];

        evm_abi::encode_calldata(func, &arg_tokens)
    }

    /// Returns the default "TypedData" with its default "struct_hash" implementation.
    /// "TypedData" implements "Eip712" trait.
    /// THIS WOULD NOT work with GSN contracts that include "type_suffix_data" on its hash and signature.
    fn typed_data(&self) -> TypedData {
        let mut message = BTreeMap::new();
        message.insert(
            String::from("from"),
            serde_json::to_value(self.from).unwrap(),
        );
        message.insert(String::from("to"), serde_json::to_value(self.to).unwrap());
        message.insert(
            String::from("value"),
            serde_json::to_value(self.value).unwrap(),
        );
        message.insert(String::from("gas"), serde_json::to_value(self.gas).unwrap());
        message.insert(
            String::from("nonce"),
            serde_json::to_value(self.nonce).unwrap(),
        );
        message.insert(
            String::from("data"),
            serde_json::to_value(hex::encode(&self.data)).unwrap(),
        );
        message.insert(
            String::from("validUntilTime"),
            serde_json::to_value(self.valid_until_time).unwrap(),
        );

        TypedData {
            domain: self.eip712_domain(),
            types: foward_request_types(),
            primary_type: "Message".to_string(),
            message,
        }
    }
}

/// Implements "type_hash".
/// ref. "ethers_core::types::transaction::eip712::EIP712_DOMAIN_TYPE_HASH"
/// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "registerRequestType"
/// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "registerRequestTypeInternal"
pub fn compute_request_type_hash(type_name: &str, type_suffix_data: &str) -> H256 {
    // solidity "abi.encodePacked" equals to string format/concatenation
    // solidity "abi.encode" equals to "ethabi::encode(&tokens)"
    // solidity "keccak256" equals to "ethers_core::utils::keccak256(&bytes)"
    //
    // e.g.,
    // keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)")
    // ref. ethers_core::types::transaction::eip712::EIP712_DOMAIN_TYPE_HASH
    let request_type = keccak256(format!(
        "{}({GENERIC_PARAMS},{}",
        type_name, type_suffix_data
    ));
    H256::from_slice(&request_type)
}

/// ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/types/transaction/eip712.rs> "TypedData"
impl Eip712 for Tx {
    type Error = Eip712Error;

    /// Default implementation of the domain separator;
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "registerDomainSeparator"
    fn domain_separator(&self) -> Result<[u8; 32], Self::Error> {
        // solidity "abi.encode" equals to "ethabi::encode(&tokens)"
        // solidity "keccak256" equals to "ethers_core::utils::keccak256"
        // ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/types/transaction/eip712.rs> "EIP712Domain.separator"
        Ok(self.eip712_domain().separator())
    }

    /// Returns the current domain. The domain depends on the contract and unique domain
    /// for which the user is targeting. In the derive macro, these attributes
    /// are passed in as arguments to the macro. When manually deriving, the user
    /// will need to know the name of the domain, version of the contract, chain ID of
    /// where the contract lives and the address of the verifying contract.
    fn domain(&self) -> Result<EIP712Domain, Self::Error> {
        Ok(self.eip712_domain())
    }

    /// This method is used for calculating the hash of the type signature of the
    /// struct. The field types of the struct must map to primitive
    /// ethereum types or custom types defined in the contract.
    fn type_hash() -> Result<[u8; 32], Self::Error> {
        Err(Eip712Error::Message("dynamic type".to_string()))
    }

    /// Hash of the struct, according to EIP-712 definition of `hashStruct`.
    fn struct_hash(&self) -> Result<[u8; 32], Self::Error> {
        let struct_hash = self.compute_struct_hash();
        Ok(struct_hash.to_fixed_bytes())
    }

    /// When using the derive macro, this is the primary method used for computing the final
    /// EIP-712 encoded payload. This method relies on the aforementioned methods for computing
    /// the final encoded payload.
    /// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "_getEncoded"
    /// ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/types/transaction/eip712.rs> "TypedData" "struct_hash"
    fn encode_eip712(&self) -> Result<[u8; 32], Self::Error> {
        let domain_separator = self.eip712_domain().separator();
        let struct_hash = self.compute_struct_hash();

        // "self.primary_type" here is "Message" for GSN
        // if self.primary_type != "EIP712Domain"
        // ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol> "_verifySig" "abi.encodePacked"
        // ref. <https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/types/transaction/eip712.rs> "TypedData"
        let digest_input = [&[0x19, 0x01], &domain_separator[..], &struct_hash[..]].concat();

        Ok(keccak256(digest_input))
    }
}

/// ref. <https://eips.ethereum.org/EIPS/eip-2770>
/// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol>
fn foward_request_types() -> Types {
    let mut types = BTreeMap::new();
    types.insert(
        "EIP712Domain".to_string(),
        vec![
            Eip712DomainType {
                name: String::from("name"),
                r#type: String::from("string"),
            },
            Eip712DomainType {
                name: String::from("version"),
                r#type: String::from("string"),
            },
            Eip712DomainType {
                name: String::from("chainId"),
                r#type: String::from("uint256"),
            },
            Eip712DomainType {
                name: String::from("verifyingContract"),
                r#type: String::from("address"),
            },
        ],
    );
    types.insert(
        "Message".to_string(),
        vec![
            Eip712DomainType {
                name: String::from("from"),
                r#type: String::from("address"),
            },
            Eip712DomainType {
                name: String::from("to"),
                r#type: String::from("address"),
            },
            Eip712DomainType {
                name: String::from("value"),
                r#type: String::from("uint256"),
            },
            Eip712DomainType {
                name: String::from("gas"),
                r#type: String::from("uint256"),
            },
            Eip712DomainType {
                name: String::from("nonce"),
                r#type: String::from("uint256"),
            },
            Eip712DomainType {
                name: String::from("data"),
                r#type: String::from("bytes"),
            },
            Eip712DomainType {
                name: String::from("validUntilTime"),
                r#type: String::from("uint256"),
            },
        ],
    );
    return types;
}
