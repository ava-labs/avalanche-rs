//! The consensus execution context.
use avalanche_types::ids::{node::Id as NodeId, Id};

/// Represents the current execution.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow#Context>
pub struct Context {
    network_id: u32,

    subnet_id: Id,
    chain_id: Id,
    node_id: NodeId,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            network_id: 1,
            subnet_id: Id::empty(),
            chain_id: Id::empty(),
            node_id: NodeId::empty(),
        }
    }

    pub fn network_id(&self) -> u32 {
        self.network_id
    }

    pub fn subnet_id(&self) -> Id {
        self.subnet_id
    }

    pub fn chain_id(&self) -> Id {
        self.chain_id
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}
