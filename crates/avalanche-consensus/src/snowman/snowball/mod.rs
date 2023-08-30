//! Snowball consensus.
pub mod binary;
pub mod tree;
pub mod unary;

use avalanche_types::ids::{bag::Bag, Id};

/// Represents a node interface for binary trie.
/// ref. "avalanchego/snow/consensus/snowball/tree.go#node"
#[derive(Clone, Debug)]
pub enum Node {
    Unary(unary::node::Node),
    Binary(binary::node::Node),
}

impl Node {
    /// Returns the preferred choice of this sub-tree.
    pub fn preference(&self) -> Id {
        match self {
            Node::Unary(n) => n.preference(),
            Node::Binary(n) => n.preference(),
        }
    }

    /// Returns the number of assumed decided bits of this node.
    /// Patricia trie places into each node the bit index to be tested
    /// in order to decide which path to take out of that node.
    /// Thus, it can skip directly to the bit where a significant
    /// decision is to be made by tracking "decided_prefix".
    pub fn decided_prefix(&self) -> i64 {
        match self {
            Node::Unary(n) => n.decided_prefix(),
            Node::Binary(n) => n.decided_prefix(),
        }
    }

    /// Returns "true" when the consensus has been reached on this node.
    pub fn finalized(&self) -> bool {
        match self {
            Node::Unary(n) => n.finalized(),
            Node::Binary(n) => n.finalized(),
        }
    }

    pub fn add(&mut self, new_choice: &Id) -> Node {
        match self {
            Node::Unary(n) => n.add(new_choice),
            Node::Binary(n) => n.add(new_choice),
        }
    }

    pub fn record_poll(&mut self, votes: Bag, reset: bool) -> (Node, bool) {
        match self {
            Node::Unary(n) => n.record_poll(votes, reset),
            Node::Binary(n) => n.record_poll(votes, reset),
        }
    }
}
