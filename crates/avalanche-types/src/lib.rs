//! # avalanche-types
//!
//! avalanche-types contains the foundational types used in the Avalanche ecosystem in Rust.
//! This include types used by the Avalanche JSON-RPC API and the EVM. Modules
//! for serialization/deserialization, hashing, and codecs are all provided.  
//!
//! The APIs can be used to build a custom, high-performance Rust VM that can run on
//! Avalanche. See the `subnet` subdirectory for an SDK that makes it easy to build a
//! custom VM in Rust.
//!
//! avalanche-types can also be used to build Rust clients and tooling within the Avalanche
//! ecosystem.
//!
#![cfg_attr(docsrs, feature(doc_cfg))]
pub mod avm;
pub mod choices;
pub mod codec;
pub mod constants;
pub mod errors;
pub mod formatting;
pub mod hash;
pub mod ids;
pub mod jsonrpc;
pub mod key;
pub mod node;
pub mod packer;
pub mod platformvm;
pub mod txs;
pub mod units;
pub mod utils;
pub mod verify;

#[cfg(feature = "avalanchego")]
#[cfg_attr(docsrs, doc(cfg(feature = "avalanchego")))]
pub mod avalanchego;

#[cfg(feature = "coreth")]
#[cfg_attr(docsrs, doc(cfg(feature = "coreth")))]
pub mod coreth;

#[cfg(feature = "subnet_evm")]
#[cfg_attr(docsrs, doc(cfg(feature = "subnet_evm")))]
pub mod subnet_evm;

#[cfg(feature = "xsvm")]
#[cfg_attr(docsrs, doc(cfg(feature = "xsvm")))]
pub mod xsvm;

#[cfg(feature = "evm")]
#[cfg_attr(docsrs, doc(cfg(feature = "evm")))]
pub mod evm;

#[cfg(feature = "message")]
#[cfg_attr(docsrs, doc(cfg(feature = "message")))]
pub mod message;

#[cfg(feature = "wallet")]
#[cfg_attr(docsrs, doc(cfg(feature = "wallet")))]
pub mod wallet;

#[cfg(feature = "proto")]
#[cfg_attr(docsrs, doc(cfg(feature = "proto")))]
pub mod proto;

#[cfg(feature = "subnet")]
#[cfg_attr(docsrs, doc(cfg(feature = "subnet")))]
pub mod subnet;
