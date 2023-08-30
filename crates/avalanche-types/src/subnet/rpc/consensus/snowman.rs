use std::io::Result;

use crate::{choices::status::Status, ids::Id};

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Block>
#[tonic::async_trait]
pub trait Block: Decidable + Send + Sync {
    /// Returns the binary representation of this block.
    ///
    /// This is used for sending blocks to peers. The bytes should be able to be
    /// parsed into the same block on another node.
    async fn bytes(&self) -> &[u8];

    /// Returns the height of the block in the chain.
    async fn height(&self) -> u64;

    /// Time this block was proposed at. This value should be consistent across
    /// all nodes. If this block hasn't been successfully verified, any value can
    /// be returned. If this block is the last accepted block, the timestamp must
    /// be returned correctly. Otherwise, accepted blocks can return any value.
    async fn timestamp(&self) -> u64;

    /// Returns the ID of this block's parent.
    async fn parent(&self) -> Id;

    /// Returns error if the block can not be verified.
    async fn verify(&mut self) -> Result<()>;
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/choices#Decidable>
#[tonic::async_trait]
pub trait Decidable {
    /// Returns a unique ID for this element.
    ///
    /// Typically, this is implemented by using a cryptographic hash of a
    /// binary representation of this element. An element should return the same
    /// IDs upon repeated calls.
    async fn id(&self) -> Id;

    /// Status returns this element's current status.
    ///
    /// If Accept has been called on an element with this ID, Accepted should be
    /// returned. Similarly, if Reject has been called on an element with this
    /// ID, Rejected should be returned. If the contents of this element are
    /// unknown, then Unknown should be returned. Otherwise, Processing should be
    /// returned.
    async fn status(&self) -> Status;

    /// Accept this element.
    ///
    /// This element will be accepted by every correct node in the network.
    async fn accept(&mut self) -> Result<()>;

    /// Rejects this element.
    ///
    /// This element will not be accepted by any correct node in the network.
    async fn reject(&mut self) -> Result<()>;
}
