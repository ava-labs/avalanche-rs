//! A unit of consensus known as a block.
pub mod test_block;

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::snowman::snowball::tree::Tree;
use avalanche_types::{
    choices::{decidable::Decidable, status::Status},
    ids::Id,
    verify::Verifiable,
};
use bytes::Bytes;

/// Represents a block as a unit of consensus, a possible decision in the chain.
/// "verify" is only called when its parent block has been verified.
/// A block can only be accepted when its parent has already been accepted.
/// A block can be rejected when its parent has already been accepted or rejected.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Block>
///
/// For instance, "avalanchego/vms/rpcchainvm/vm_client.go#blockClient" implements
/// this "Decidable" and "Block" interface to use snowman consensus.
///
/// Invariants are:
/// - accept or reject on a block will be called at most once
/// - once accept or reject is called, neither is ever called again on the same block
/// - if block.verify returns no error, the block will eventually accepted or rejected
/// - accept or reject returning an error must be fatal
/// - accept can only be called when its parent has already been accepted
///
pub trait Block: Clone + Verifiable + Decidable {
    /// Returns the bytes of this block.
    fn bytes(&self) -> Bytes;

    /// Returns the height of the block in the chain.
    fn height(&self) -> u64;
    /// Returns the block proposal timestamp in unix seconds.
    /// Use "UNIX_EPOCH + Duration::from_secs(epoch_seconds)"
    /// to convert into SystemTime.
    fn timestamp(&self) -> u64;

    /// Returns the ID of this block's parent.
    fn parent(&self) -> Id;
}

/// Maps from a block Id to the block with reference counting.
type Blocks<B> = Rc<RefCell<HashMap<Id, Rc<RefCell<B>>>>>;

/// Tracks the state of a snowman block.
/// ref. "avalanchego/snow/consensus/snowman#snowmanBlock"
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Block>
#[derive(Debug, Clone)]
pub struct SnowmanBlock<B: Block> {
    /// Parameters to be inherited by the following trees.
    parameters: crate::Parameters,

    /// A block that this node contains.
    /// This field must be set None for the genesis block.
    /// snowman code does not use this field for recursive updates.
    /// Thus just define it with "Option" as an immutable field.
    pub blk: Option<Rc<RefCell<B>>>,

    /// Snowball instance to run votes and build consensus for conflicting child blocks.
    /// Used to decide which child is the canonical child of this block.
    /// If this node has not had a child issued under it, this field will be None.
    pub sb: Option<Box<Tree>>,

    /// Set of blocks that have been issued with this block as their parent.
    /// If this node has not had a child issued under it, this field will be None.
    pub children: Option<Blocks<B>>,

    /// Set to "true" if this node and all its descendants received
    /// less than the alpha votes.
    pub should_falter: Cell<bool>,
}

impl<B> SnowmanBlock<B>
where
    B: Block,
{
    pub fn new(parameters: crate::Parameters) -> Self {
        Self {
            parameters,
            blk: None,
            sb: None,
            children: None,
            should_falter: Cell::new(false),
        }
    }

    pub fn new_with_block(parameters: crate::Parameters, blk: Rc<RefCell<B>>) -> Self {
        Self {
            parameters,
            blk: Some(blk),
            sb: None,
            children: None,
            should_falter: Cell::new(false),
        }
    }

    /// Returns the Id of the inner block.
    /// If the inner block is None (genesis), then it returns None.
    pub fn id(&self) -> Option<Id> {
        if self.blk.is_none() {
            None
        } else {
            Some(self.blk.as_ref().unwrap().borrow().id())
        }
    }

    /// Returns the height of the inner block.
    /// If the inner block is None (genesis), then it returns None.
    pub fn height(&self) -> Option<u64> {
        if self.blk.is_none() {
            None
        } else {
            Some(self.blk.as_ref().unwrap().borrow().height())
        }
    }

    /// Returns the status of the inner block.
    /// If the inner block is None (genesis), then it returns None.
    pub fn status(&self) -> Option<Status> {
        if self.blk.is_none() {
            None
        } else {
            Some(self.blk.as_ref().unwrap().borrow().status())
        }
    }

    pub fn accepted(&self) -> bool {
        // genesis block, defined as accepted
        if self.blk.is_none() {
            return true;
        }
        self.blk.as_ref().unwrap().borrow().status() == Status::Accepted
    }

    pub fn add_child(&mut self, child: Rc<RefCell<B>>) {
        let child_id = child.borrow().id();

        // if the snowball instance is nil, this is the first child
        // thus the instance should be initialized
        if let Some(sb) = self.sb.as_mut() {
            sb.add(&child_id);
        } else {
            let sb = Tree::new(self.parameters.clone(), child_id);
            self.sb = Some(Box::new(sb));
            self.children = Some(Rc::new(RefCell::new(HashMap::new())));
        }

        if let Some(children) = self.children.as_mut() {
            children.borrow_mut().insert(child_id, child);
        } else {
            panic!("unexpected None children")
        }
    }
}
