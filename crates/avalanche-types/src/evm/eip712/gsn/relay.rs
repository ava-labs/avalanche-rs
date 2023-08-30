#![allow(deprecated)]

use std::{
    convert::TryFrom,
    io::{self, Error, ErrorKind},
    str::FromStr,
    sync::Arc,
};

use ethers::prelude::Eip1559TransactionRequest;
use ethers_core::types::{
    transaction::{
        eip2718,
        eip712::{Eip712, TypedData},
    },
    Bytes as EthBytes, RecoveryMessage, Signature, H160, H256, U256,
};
use ethers_providers::{Http, Middleware, Provider, RetryClient};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tokio::time::{sleep, Duration, Instant};
use zerocopy::AsBytes;

impl super::Tx {
    pub async fn sign(
        &self,
        eth_signer: impl ethers_signers::Signer + Clone,
    ) -> io::Result<Vec<u8>> {
        Request::sign(self, eth_signer).await
    }

    /// Builds and signs the typed data with the signer and returns the
    /// "RelayTransactionRequest" with the signature attached in the relay metadata.
    /// Use "serde_json::to_vec" to encode to "ethers_core::types::Bytes"
    /// and send the request via "eth_sendRawTransaction".
    pub async fn sign_to_request(
        &self,
        eth_signer: impl ethers_signers::Signer + Clone,
    ) -> io::Result<Request> {
        Request::sign_to_request(self, eth_signer).await
    }

    /// "sign_to_request" but with estimated gas via RPC endpoints.
    pub async fn sign_to_request_with_estimated_gas(
        &mut self,
        eth_signer: impl ethers_signers::Signer + Clone,
        chain_rpc_provider: Arc<Provider<RetryClient<Http>>>,
    ) -> io::Result<Request> {
        // as if a user sends EIP-1559 to the recipient contract
        let eip1559_tx = Eip1559TransactionRequest::new()
            .chain_id(self.domain_chain_id.as_u64())
            .from(self.from)
            .to(self.to)
            .gas(self.gas)
            .data(self.data.clone());
        let typed_tx: eip2718::TypedTransaction = eip1559_tx.into();
        log::info!(
            "estimating gas for typed tx {}",
            serde_json::to_string(&typed_tx).unwrap()
        );

        // this can fail with 'gas required exceeds allowance'
        // when the specified gas cap is too low
        let estimated_gas = chain_rpc_provider
            .estimate_gas(&typed_tx, None)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed estimate_gas '{}'", e)))?;
        log::info!("estimated gas {estimated_gas} -- now signing again with updated gas");

        self.gas = estimated_gas;
        Request::sign_to_request(&self, eth_signer).await
    }

    /// "sign_to_request" but with estimated gas via RPC endpoints.
    pub async fn sign_to_request_with_estimated_gas_with_retries(
        &mut self,
        eth_signer: impl ethers_signers::Signer + Clone,
        chain_rpc_provider: Arc<Provider<RetryClient<Http>>>,
        retry_timeout: Duration,
        retry_interval: Duration,
        retry_increment_gas: U256,
    ) -> io::Result<Request> {
        log::info!(
            "sign with retries estimated gas, retry timeout {:?}, retry interval {:?}, retry increment gas {retry_increment_gas}",
            retry_timeout,
            retry_interval,
        );

        let start = Instant::now();
        if self.gas.is_zero() {
            self.gas = U256::from(21000);
        }
        let mut retries = 0;
        loop {
            let elapsed = start.elapsed();
            if elapsed.gt(&retry_timeout) {
                break;
            }
            retries = retries + 1;

            match Self::sign_to_request_with_estimated_gas(
                self,
                eth_signer.clone(),
                chain_rpc_provider.clone(),
            )
            .await
            {
                Ok(req) => return Ok(req),
                Err(e) => {
                    log::warn!(
                        "[retry {:02}] failed to estimate gas {} with gas {} (incrementing, elapsed {:?})",
                        retries,
                        e,
                        self.gas,
                        elapsed
                    );
                    if let Some(added_gas) = self.gas.checked_add(retry_increment_gas) {
                        self.gas = added_gas;
                    } else {
                        return Err(Error::new(ErrorKind::Other, "gas overflow U256"));
                    }

                    sleep(retry_interval).await;
                    continue;
                }
            }
        }
        return Err(Error::new(ErrorKind::Other, "failed estimate_gas in time"));
    }
}

/// Used for gas relayer server, compatible with the OpenGSN request.
/// ref. <https://github.com/opengsn/gsn/blob/master/packages/common/src/types/RelayTransactionRequest.ts>
/// ref. <https://github.com/opengsn/gsn/blob/master/packages/common/src/EIP712/RelayRequest.ts>
/// ref. <https://github.com/opengsn/gsn/blob/master/packages/common/src/EIP712/ForwardRequest.ts>
/// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/IForwarder.sol>
/// ref. <https://github.com/opengsn/gsn/blob/master/packages/common/src/EIP712/RelayData.ts>
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub forward_request: TypedData,
    pub metadata: Metadata,
}

/// ref. <https://github.com/opengsn/gsn/blob/master/packages/common/src/types/RelayTransactionRequest.ts>
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    #[serde_as(as = "serde_with::hex::Hex")]
    pub signature: Vec<u8>,
}

impl Request {
    /// Parses the eth_sendRawTransaction request and decodes the EIP-712 encoded typed
    /// data and signature in the relay metadata,
    /// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_sendrawtransaction>
    pub fn from_send_raw_transaction(value: serde_json::Value) -> io::Result<Self> {
        let params = value.get("params").ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "missing/invalid params field in the request body",
            )
        })?;

        let hex_encoded_tx = params
            .get(0)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidData,
                    "missing/invalid params[0] value in the request body",
                )
            })?;

        let hex_decoded = EthBytes::from_str(hex_encoded_tx).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("failed to decode raw transaction {}", e),
            )
        })?;

        Self::decode_signed(&hex_decoded)
    }

    /// Decodes the EIP-712 encoded typed data and signature in the relay metadata.
    /// ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_sendrawtransaction>
    pub fn decode_signed(b: impl AsRef<[u8]>) -> io::Result<Self> {
        serde_json::from_slice(b.as_ref()).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed serde_json::from_slice '{}'", e),
            )
        })
    }

    /// Signs the typed data with the signer and returns the signature.
    pub async fn sign(
        tx: &super::Tx,
        signer: impl ethers_signers::Signer + Clone,
    ) -> io::Result<Vec<u8>> {
        let sig = signer
            .sign_typed_data(tx)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed sign_typed_data '{}'", e)))?;

        Ok(sig.to_vec())
    }

    /// Signs the typed data with the signer and returns the "RelayTransactionRequest"
    /// with the signature attached in the relay metadata.
    /// Use "serde_json::to_vec" to encode to "ethers_core::types::Bytes"
    /// and send the request via "eth_sendRawTransaction".
    pub async fn sign_to_request(
        tx: &super::Tx,
        signer: impl ethers_signers::Signer + Clone,
    ) -> io::Result<Self> {
        let sig = signer
            .sign_typed_data(tx)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed sign_typed_data '{}'", e)))?;

        Ok(Self {
            forward_request: tx.typed_data(),
            metadata: Metadata {
                signature: sig.to_vec(),
            },
        })
    }

    /// Recovers the GSN transaction object based on the raw typed data and given type name and suffix data.
    pub fn recover_tx(&self, type_name: &str, type_suffix_data: &str) -> io::Result<super::Tx> {
        let domain_name = if let Some(name) = &self.forward_request.domain.name {
            name.clone()
        } else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "forward_request.domain missing 'name' field",
            ));
        };

        let domain_version = if let Some(version) = &self.forward_request.domain.version {
            version.clone()
        } else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "forward_request.domain missing 'version' field",
            ));
        };

        let domain_chain_id = if let Some(chain_id) = &self.forward_request.domain.chain_id {
            chain_id
        } else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "forward_request.domain missing 'chain_id' field",
            ));
        };

        let domain_verifying_contract =
            if let Some(verifying_contract) = &self.forward_request.domain.verifying_contract {
                verifying_contract.clone()
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "forward_request.domain missing 'verifying_contract' field",
                ));
            };

        let from = if let Some(from) = self.forward_request.message.get("from") {
            if let Some(v) = from.as_str() {
                H160::from_str(v).map_err(|e| {
                    Error::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "forward_request.message.from[{v}] H160::from_str parse failed {:?}",
                            e
                        ),
                    )
                })?
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "forward_request.message expected type 'from'",
                ));
            }
        } else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "forward_request.message missing 'from' field",
            ));
        };

        let to = if let Some(to) = self.forward_request.message.get("to") {
            if let Some(v) = to.as_str() {
                H160::from_str(v).map_err(|e| {
                    Error::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "forward_request.message.to[{v}] H160::from_str parse failed {:?}",
                            e
                        ),
                    )
                })?
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "forward_request.message expected type 'to'",
                ));
            }
        } else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "forward_request.message missing 'to' field",
            ));
        };

        let value = if let Some(value) = self.forward_request.message.get("value") {
            if let Some(v) = value.as_str() {
                if v.starts_with("0x") {
                    U256::from_str_radix(v, 16).map_err(|e| {
                        Error::new(
                            ErrorKind::InvalidInput,
                            format!("forward_request.message.value[{v}] U256::from_str_radix parse failed {:?}", e),
                        )
                    })?
                } else {
                    U256::from_str(v).map_err(|e| {
                        Error::new(
                            ErrorKind::InvalidInput,
                            format!("forward_request.message.value[{v}] U256::from_str parse failed {:?}", e),
                        )
                    })?
                }
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "forward_request.message expected type 'value'",
                ));
            }
        } else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "forward_request.message missing 'value' field",
            ));
        };

        let gas = if let Some(gas) = self.forward_request.message.get("gas") {
            if let Some(v) = gas.as_str() {
                if v.starts_with("0x") {
                    U256::from_str_radix(v, 16).map_err(|e| {
                        Error::new(
                            ErrorKind::InvalidInput,
                            format!("forward_request.message.gas[{v}] U256::from_str_radix parse failed {:?}", e),
                        )
                    })?
                } else {
                    U256::from_str(v).map_err(|e| {
                        Error::new(
                            ErrorKind::InvalidInput,
                            format!(
                                "forward_request.message.gas[{v}] U256::from_str parse failed {:?}",
                                e
                            ),
                        )
                    })?
                }
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "forward_request.message expected type 'gas'",
                ));
            }
        } else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "forward_request.message missing 'gas' field",
            ));
        };

        let nonce = if let Some(nonce) = self.forward_request.message.get("nonce") {
            if let Some(v) = nonce.as_str() {
                if v.starts_with("0x") {
                    U256::from_str_radix(v, 16).map_err(|e| {
                        Error::new(
                            ErrorKind::InvalidInput,
                            format!("forward_request.message.nonce[{v}] U256::from_str_radix parse failed {:?}", e),
                        )
                    })?
                } else {
                    U256::from_str(v).map_err(|e| {
                        Error::new(
                            ErrorKind::InvalidInput,
                            format!("forward_request.message.nonce[{v}] U256::from_str parse failed {:?}", e),
                        )
                    })?
                }
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "forward_request.message expected type 'nonce'",
                ));
            }
        } else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "forward_request.message missing 'nonce' field",
            ));
        };

        let data = if let Some(data) = self.forward_request.message.get("data") {
            if let Some(v) = data.as_str() {
                hex::decode(v.trim_start_matches("0x")).map_err(|e| {
                    Error::new(
                        ErrorKind::InvalidInput,
                        format!("failed hex::decode on 'data' field '{}'", e),
                    )
                })?
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "forward_request.message expected type 'data'",
                ));
            }
        } else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "forward_request.message missing 'data' field",
            ));
        };

        let valid_until_time = if let Some(valid_until_time) =
            self.forward_request.message.get("validUntilTime")
        {
            if let Some(v) = valid_until_time.as_str() {
                if v.starts_with("0x") {
                    U256::from_str_radix(v, 16).map_err(|e| {
                            Error::new(
                                ErrorKind::InvalidInput,
                                format!(
                            "forward_request.message.validUntilTime[{v}] U256::from_str_radix parse failed {:?}",
                            e
                        ),
                            )
                        })?
                } else {
                    U256::from_str(v).map_err(|e| {
                        Error::new(
                            ErrorKind::InvalidInput,
                            format!(
                                "forward_request.message.validUntilTime[{v}] U256::from_str parse failed {:?}",
                                e
                            ),
                        )
                    })?
                }
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "forward_request.message expected type 'validUntilTime'",
                ));
            }
        } else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "forward_request.message missing 'validUntilTime' field",
            ));
        };

        Ok(super::Tx::new()
            .domain_name(domain_name)
            .domain_version(domain_version)
            .domain_chain_id(domain_chain_id)
            .domain_verifying_contract(domain_verifying_contract)
            .from(from)
            .to(to)
            .value(value)
            .gas(gas)
            .nonce(nonce)
            .data(data)
            .valid_until_time(valid_until_time)
            .type_name(type_name)
            .type_suffix_data(type_suffix_data))
    }

    /// Recovers the signature and signer address from its relay metadata signature field.
    pub fn recover_signature(
        &self,
        type_name: &str,
        type_suffix_data: &str,
    ) -> io::Result<(Signature, H160)> {
        let sig = Signature::try_from(self.metadata.signature.as_bytes()).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed Signature::try_from '{}'", e),
            )
        })?;

        let tx = self.recover_tx(type_name, type_suffix_data)?;
        let fwd_req_hash = tx
            .encode_eip712()
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed encode_eip712 '{}'", e)))?;
        let fwd_req_hash = H256::from_slice(&fwd_req_hash.to_vec());

        let signer_addr = sig.recover(RecoveryMessage::Hash(fwd_req_hash)).map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!(
                        "failed to recover signer address from signature and forward request hash '{}'",
                        e
                    ),
                )
            })?;
        Ok((sig, signer_addr))
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib --features="evm" -- evm::eip712::gsn::relay::test_build_relay_transaction_request --exact --show-output
#[test]
fn test_build_relay_transaction_request() {
    use ethers_core::{
        abi::{Function, Param, ParamType, StateMutability, Token},
        types::U256,
    };
    use ethers_signers::{LocalWallet, Signer};

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
    let arg_tokens = vec![Token::String(random_manager::secure_string(10))];
    let calldata = crate::evm::abi::encode_calldata(func, &arg_tokens).unwrap();
    log::info!("calldata: 0x{}", hex::encode(calldata.clone()));

    macro_rules! ab {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    let domain_name = random_manager::secure_string(20);
    let domain_version = format!("{}", random_manager::u16());

    let my_type = random_manager::secure_string(20);
    let my_suffix_data = random_manager::secure_string(20);

    let tx = super::Tx::new()
        .domain_name(domain_name)
        .domain_version(domain_version)
        .domain_chain_id(U256::from(random_manager::u64()))
        .domain_verifying_contract(H160::random())
        .from(H160::random())
        .to(H160::random())
        .value(U256::zero())
        .gas(U256::from(random_manager::u64()))
        .nonce(U256::from(random_manager::u64()))
        .data(calldata)
        .valid_until_time(U256::from(random_manager::u64()))
        .type_name(&my_type)
        .type_suffix_data(&my_suffix_data);

    let k = crate::key::secp256k1::private_key::Key::generate().unwrap();
    let signer: LocalWallet = k.to_ethers_core_signing_key().into();

    let rr: Request = ab!(tx.sign_to_request(signer.clone())).unwrap();
    log::info!("request: {}", serde_json::to_string_pretty(&rr).unwrap());

    let signed_bytes: EthBytes = serde_json::to_vec(&rr).unwrap().into();
    log::info!("signed bytes of relay request: {}", signed_bytes);

    // ref. <https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_sendrawtransaction>
    let raw_tx_req = serde_json::json!({
        "id": 1,
        "method": "eth_sendRawTransaction",
        "jsonrpc": "2.0",
        "params": [signed_bytes],
    });
    let decoded_from_raw_tx = Request::from_send_raw_transaction(raw_tx_req).unwrap();
    log::info!(
        "decoded_from_raw_tx: {}",
        serde_json::to_string_pretty(&decoded_from_raw_tx).unwrap()
    );
    assert_eq!(rr, decoded_from_raw_tx);

    let (sig1, signer_addr) = rr.recover_signature(&my_type, &my_suffix_data).unwrap();
    assert_eq!(k.to_public_key().to_h160(), signer_addr);

    // default TypeData has different "struct_hash"
    let sig2 = ab!(signer.sign_typed_data(&rr.forward_request)).unwrap();
    assert_ne!(sig1, sig2);

    // Tx implements its own "struct_hash", must match with the recovered signature
    let sig3 =
        ab!(signer.sign_typed_data(&rr.recover_tx(&my_type, &my_suffix_data).unwrap())).unwrap();
    assert_eq!(sig1, sig3);

    let d = tx.encode_execute_call(sig1.to_vec()).unwrap();
    log::info!("encode_execute_call: {}", hex::encode(d));
}
