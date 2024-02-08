use std::{io::Result, time::Duration};

use bytes::Bytes;

use crate::{
    ids::Id,
    subnet::rpc::{consensus::snowman, snow::engine::common::vm::CommonVm},
};

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/snowman/block#ChainVm>
#[tonic::async_trait]
pub trait ChainVm: CommonVm + BatchedChainVm + Getter + Parser {
    type Block: snowman::Block;

    /// Attempt to create a new block from ChainVm data
    /// Returns either a block or an error
    async fn build_block(&self) -> Result<<Self as ChainVm>::Block>;

    /// Issues a transaction to the chain
    async fn issue_tx(&self) -> Result<<Self as ChainVm>::Block>;

    /// Notify the Vm of the currently preferred block.
    async fn set_preference(&self, id: Id) -> Result<()>;

    /// Returns the ID of the last accepted block.
    /// If no blocks have been accepted, this should return the genesis block
    async fn last_accepted(&self) -> Result<Id>;

    /// Returns empty if the height index is available.
    /// Returns ErrIndexIncomplete if the height index is not currently available.
    /// TODO: Remove after v1.11.x activates.
    async fn verify_height_index(&self) -> Result<()>;

    /// Returns the ID of the block that was accepted with `height`.
    /// Returns ErrNotFound if the `height` index is unknown.
    async fn get_block_id_at_height(&self, height: u64) -> Result<Id>;

    /// Returns whether state sync is enabled.
    async fn state_sync_enabled(&self) -> Result<bool>;
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/snowman/block#Getter>
#[tonic::async_trait]
pub trait Getter {
    type Block: snowman::Block;

    async fn get_block(&self, id: Id) -> Result<<Self as Getter>::Block>;
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/snowman/block#Parser>
#[tonic::async_trait]
pub trait Parser {
    type Block: snowman::Block;

    async fn parse_block(&self, bytes: &[u8]) -> Result<<Self as Parser>::Block>;
}

/// Defines the block context that will be optionally provided by the proposervm
/// to an underlying vm.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/snowman/block#Context>
pub struct Context {
    /// Height that this block will use to verify it's state.  In the
    /// proposervm, blocks verify the proposer based on the P-chain height
    /// recorded in the parent block. The P-chain height provided here is also
    /// the parent's P-chain height, not this block's P-chain height.
    ///
    /// Because PreForkBlocks and PostForkOptions do not verify their execution
    /// against the P-chain's state, this context is undefined for those blocks.
    pub p_chain_height: u64,
}

/// Defines the trait a [`ChainVm`] can optionally implement to consider the
/// P-Chain height when building blocks.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/snowman/block#BuildBlockWithContextChainVM>
#[tonic::async_trait]
pub trait BuildBlockWithContextChainVM {
    type Block: snowman::Block;

    /// Attempt to build a new block given that the P-Chain height is
    /// [block_ctx.p_chain_height].
    ///
    /// This method will be called if and only if the proposervm is activated.
    /// Otherwise \[build_block\] will be called.
    async fn build_block_with_context(
        &self,
        blk_context: &Context,
    ) -> Result<<Self as BuildBlockWithContextChainVM>::Block>;
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/snowman/block#WithVerifyContext>
#[tonic::async_trait]
pub trait WithVerifyContext {
    /// Returns true if \[`verify_with_context`\] should be called.
    /// Returns false if \[verify\] should be called.
    ///
    /// This method will be called if and only if the proposervm is activated.
    /// Otherwise \[verify\] will be called.
    async fn should_verify_with_context(&self) -> Result<bool>;

    /// Verify that the state transition this block would make if accepted is
    /// valid. If the state transition is invalid, a non-nil error should be
    /// returned.
    ///
    /// It is guaranteed that the Parent has been successfully verified.
    ///
    /// This method may be called again with a different context.
    async fn verify_with_context(&self, blk_context: &Context) -> Result<()>;
}

/// Extends the minimal functionalities exposed by [`ChainVm`] for VMs
/// communicating over network (gRPC in our case). This allows more efficient
/// operations since calls over network can be batched.
///
/// ref <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/snowman/block#BatchedChainVM>
#[tonic::async_trait]
pub trait BatchedChainVm {
    type Block: snowman::Block;

    /// Attempts to obtain the ancestors of a block.
    async fn get_ancestors(
        &self,
        block_id: Id,
        max_block_num: i32,
        max_block_size: i32,
        max_block_retrival_time: Duration,
    ) -> Result<Vec<Bytes>>;

    /// Attempts to batch parse_block requests.
    async fn batched_parse_block(&self, blocks: &[Vec<u8>]) -> Result<Vec<Self::Block>>;
}
