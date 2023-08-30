pub mod client;

use std::{collections::BTreeMap, fmt::Debug, io};

use crate::{ids, key::bls::public_key::Key};

/// Contains the base values representing a validator
/// of the Avalanche Network.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/validators#Validator>
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Validator {
    pub node_id: ids::node::Id,
    pub public_key: Key,
    pub tx_id: ids::Id,
    pub weight: u64,

    /// Used to efficiently remove validators from the validator set. It
    /// represents the index of this validator in the vdrSlice and weights
    /// arrays.
    index: u32,
}

/// Contains the publicly relevant values of a validator of the Avalanche
/// Network for the output of GetValidator.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/validators#GetValidatorOutput>
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GetValidatorOutput {
    pub node_id: ids::node::Id,
    pub public_key: Option<Key>,
    pub weight: u64,
}

/// Allows the lookup of validator sets on specified subnets at the
/// requested P-chain height.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/validators#State>
#[tonic::async_trait]
pub trait State: Debug {
    /// Returns the minimum height of the block still in the
    /// proposal window.
    async fn get_minimum_height(&self) -> io::Result<u64>;

    /// Returns the current height of the P-chain.
    async fn get_current_height(&self) -> io::Result<u64>;

    /// Returns the subnet ID of the provided chain.
    async fn get_subnet_id(&self, chain_id: ids::Id) -> io::Result<ids::Id>;

    /// Returns the validators of the provided subnet at the
    /// requested P-chain height.
    /// The returned map should not be modified.
    async fn get_validator_set(
        &self,
        height: u64,
        subnet_id: ids::Id,
    ) -> io::Result<BTreeMap<ids::node::Id, GetValidatorOutput>>;
}
