//! Implementation of the Snowman interface.
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use crate::{
    context::Context,
    snowman::block::{Block, SnowmanBlock},
};
use avalanche_types::{
    choices::status::Status,
    errors::{Error, Result},
    ids::{bag::Bag, Id},
};

/// Implements the Snowman interface by using a tree tracking
/// the strongly preferred branch. This tree structure amortizes
/// network polls to vote on more than just the next block.
///
/// "Tree" is to "snowball/snowman", "Directed" is to "snowstorm/avalanche".
/// "Tree" implements "snowball.Consensus".
/// "Directed" implements "snowstorm.Consensus".
/// "snowman.Topological" implements "snowman.Consensus".
/// "avalanche.Topological" implements "avalanche.Consensus".
///
/// ref. <https://github.com/ava-labs/avalanchego/blob/master/snow/consensus/snowman/topological.go>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowball#Tree>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowball#Consensus>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowstorm#Directed>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Topological>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Consensus>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/avalanche#Topological>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/avalanche#Consensus>
///
/// Invariants are:
/// - accept or reject on a block will be called at most once
/// - once accept or reject is called, neither is ever called again on the same block
/// - if block.verify returns no error, the block will eventually accepted or rejected
/// - accept or reject returning an error must be fatal
/// - accept can only be called when its parent has already been accepted
///
/// TODO: add metrics
pub struct Topological<B: Block> {
    /// Context that this snowman instance is executing in.
    #[allow(dead_code)]
    ctx: Context,

    /// Parameters to initialize snowball instances.
    parameters: crate::Parameters,

    /// Number of times "record_polls" has been called.
    poll_number: Cell<u64>,

    /// Height of the last accepted block.
    height: Cell<u64>,

    /// Tracks the set of Ids that are currently preferred.
    preferred_ids: Rc<RefCell<HashSet<Id>>>,
    /// Last preferred block from votes to track newly preferred blocks.
    /// Must reset for each votes apply -- only used for applying votes.
    last_preferred: Cell<Id>,

    /// Preference of the last accepted block.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Topological.head>
    head: Cell<Id>,
    /// Currently preferred block with "no" child.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Topological.tail>
    tail: Cell<Id>,

    /// Maps each block ID to the snowman block.
    /// This includes the "last" accepted block and all the pending blocks.
    /// A block is added to "blocks" when "Topological" is created with genesis
    /// and a new block is added to the "Topological".
    /// A block is removed from "blocks" when a block is rejected,
    /// or its child has just been accepted.
    blocks: SnowmanBlocks<B>,

    /// If we are on the preferred branch and the next_id is the preference of
    /// the current snowball instance, then we are following the preferred branch.
    /// Must reset for each votes apply -- only used for applying votes.
    currently_on_preferred_branch: Cell<bool>,

    /// Must reset for each "calculate_in_degree".
    leaves: Rc<RefCell<HashSet<Id>>>,

    /// Must reset for each "calculate_in_degree".
    kahn_nodes: Rc<RefCell<HashMap<Id, KahnNode>>>,
}

/// Maps from a block Id to the snowman block with reference counting.
type SnowmanBlocks<B> = Rc<RefCell<HashMap<Id, Rc<RefCell<SnowmanBlock<B>>>>>>;

/// Track the Kahn topological sort status.
/// ref. "avalanchego/snow/consensus/snowman#kahnNode"
struct KahnNode {
    /// Number of children that haven't been processed yet.
    /// If "in_degree" is 0, then this node is a leaf.
    in_degree: Cell<i64>,
    /// Votes for all the children of this node, so far.
    votes: Bag,
}

impl KahnNode {
    fn new() -> Self {
        Self {
            in_degree: Cell::new(0),
            votes: Bag::new(),
        }
    }
}

impl Clone for KahnNode {
    fn clone(&self) -> Self {
        Self {
            in_degree: Cell::new(self.in_degree.get()),
            votes: self.votes.deep_copy(),
        }
    }
}

/// Tracks which children should receive votes.
/// ref. "avalanchego/snow/consensus/snowman#votes"
struct Votes {
    /// Parent of all the votes provided in the votes bag.
    parent_id: Id,
    /// Votes for all the children of the parent.
    votes: Bag,
}

impl Votes {
    fn new(parent_id: Id, votes: Bag) -> Self {
        Self { parent_id, votes }
    }
}

impl<B> Topological<B>
where
    B: Block,
{
    /// Creates a new Topological instance, and returns the reference to the genesis block.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Topological.Initialize>
    pub fn new(
        ctx: Context,
        parameters: crate::Parameters,
        genesis_blk_id: Id,
        genesis_blk_height: u64,
    ) -> Result<(Self, Rc<RefCell<SnowmanBlock<B>>>)> {
        parameters.verify()?;

        let genesis_blk = SnowmanBlock::new(parameters.clone());
        let genesis_blk_rc = Rc::new(RefCell::new(genesis_blk));

        let mut blocks: HashMap<Id, Rc<RefCell<SnowmanBlock<B>>>> = HashMap::new();
        blocks.insert(genesis_blk_id, Rc::clone(&genesis_blk_rc));

        log::debug!(
            "creating a new Topological with genesis block {}",
            genesis_blk_id
        );
        Ok((
            Self {
                ctx,
                parameters,
                poll_number: Cell::new(0),
                height: Cell::new(genesis_blk_height),

                preferred_ids: Rc::new(RefCell::new(HashSet::new())),
                last_preferred: Cell::new(Id::empty()),

                head: Cell::new(genesis_blk_id),
                tail: Cell::new(genesis_blk_id),

                blocks: Rc::new(RefCell::new(blocks)),

                currently_on_preferred_branch: Cell::new(false),
                leaves: Rc::new(RefCell::new(HashSet::new())),
                kahn_nodes: Rc::new(RefCell::new(HashMap::new())),
            },
            Rc::clone(&genesis_blk_rc),
        ))
    }

    /// Returns the copied parameters.
    pub fn parameters(&self) -> crate::Parameters {
        self.parameters.clone()
    }

    /// Returns the Id of the tail of the strongly preferred
    /// sequence of decisions.
    pub fn preference(&self) -> Id {
        self.tail.get()
    }

    /// Returns the current height.
    pub fn height(&self) -> u64 {
        self.height.get()
    }

    /// Returns true if all decisions that have been added have been finalized.
    /// Note that it is possible that after returning finalized true,
    /// a new decision may be added such that this instance is no longer finalized.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Topological.Finalized>
    pub fn finalized(&self) -> bool {
        // "blocks" includes the last accepted block and all the pending blocks.
        self.blocks.borrow().len() == 1
    }

    /// Returns the number of block processing.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Topological.NumProcessing>
    pub fn num_processing(&self) -> i64 {
        // "blocks" includes the last accepted block and all the pending blocks.
        (self.blocks.borrow().len() - 1) as i64
    }

    /// Returns true if the block is currently processing.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Topological.Processing>
    pub fn block_processing(&self, blk_id: Id) -> bool {
        // the last accepted block is in the blocks map
        // so we first must ensure the requested block isn't the last accepted block
        if blk_id == self.head.get() {
            return false;
        }
        // not the head, and pending block
        // then the block is currently processing
        self.blocks.borrow().contains_key(&blk_id)
    }

    /// Returns true if the block is currently on the preferred chain.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Topological.IsPreferred>
    pub fn block_preferred(&self, blk: Rc<RefCell<SnowmanBlock<B>>>) -> bool {
        // if the block is accepted, then it must be transitively preferred
        if blk.borrow().accepted() {
            return true;
        }

        self.preferred_ids
            .borrow()
            .contains(&blk.borrow().id().unwrap())
    }

    /// Returns true if the block has been decided.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Topological.Decided>
    pub fn block_decided(&self, blk: Rc<RefCell<SnowmanBlock<B>>>) -> bool {
        // genesis block
        if blk.borrow().status().is_none() {
            return true;
        }

        // if the block is decided, it must have been previously issued
        if blk.borrow().status().unwrap().decided() {
            return true;
        }

        // if the block is marked as fetched,
        // check if it has been transitively rejected
        blk.borrow().status().unwrap() == Status::Processing
            && blk.borrow().height().unwrap() <= self.height()
    }

    /// Adds a block decision to the consensus.
    /// Assumes the dependency has already been added.
    /// It clones the being-passed block to create a new
    /// snowman block and returns the reference to the one
    /// that's being tracked in Topological.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Topological.Add>
    pub fn add_block(&self, blk: B) -> Result<Rc<RefCell<SnowmanBlock<B>>>> {
        // This makes sure a block is not inserted twice -- invariant blocks are always
        // added in topological order. Essentially, a block that is being added should
        // never have a child that was already added. Additionally, this prevents any
        // edge cases that may occur due to adding different blocks with the same ID.
        let blk_id = blk.id();
        if self.block_processing(blk_id) {
            return Err(Error::Other {
                message: "duplicate block add".to_string(),
                retryable: false,
            });
        }

        // insert into the "blocks" itself, thus requires mutable borrower
        let mut borrowed_mut_blks = self.blocks.borrow_mut();

        // add this block as a child of its parent
        let parent_blk_id = blk.parent();

        // MUST create its own data for interior mutability
        // with its own reference counter for following calls!
        let blk_rc = Rc::new(RefCell::new(blk));
        let snowman_blk_rc = Rc::new(RefCell::new(SnowmanBlock::new_with_block(
            self.parameters(),
            Rc::clone(&blk_rc),
        )));

        // This makes sure a block is not inserted twice -- invariant blocks are always
        // added in topological order. Essentially, a block that is being added should
        // never have a child that was already added. Additionally, this prevents any
        // edge cases that may occur due to adding different blocks with the same ID.
        if self.block_decided(Rc::clone(&snowman_blk_rc)) {
            return Err(Error::Other {
                message: "duplicate block add".to_string(),
                retryable: false,
            });
        }

        let res = borrowed_mut_blks.get(&parent_blk_id);
        if res.is_none() {
            // its ancestor is missing, must've been pruned/rejected
            // thus its dependent should be transitively rejected
            log::debug!(
                "rejecting {} due to missing parent block {}",
                blk_id,
                parent_blk_id
            );
            blk_rc.borrow_mut().reject()?;

            return Ok(Rc::clone(&snowman_blk_rc));
        }

        log::debug!("adding {} as a child to {}", blk_id, parent_blk_id);
        let parent_blk = res.unwrap();
        parent_blk.borrow_mut().add_child(Rc::clone(&blk_rc));

        // "blocks" should include the last accepted block and all the pending blocks
        borrowed_mut_blks.insert(blk_id, Rc::clone(&snowman_blk_rc));

        // if we are extending the tail, the newly added block becomes the new tail
        if self.tail.get() == parent_blk_id {
            // currently preferred block with no child
            self.tail.set(blk_id);
            self.preferred_ids.borrow_mut().insert(blk_id);
        }

        Ok(Rc::clone(&snowman_blk_rc))
    }

    /// Collects the results of network poll.
    /// Assumes all decisions have been previously added.
    ///
    /// The votes bag contains at most K votes for blocks in the tree.
    /// If there is a vote for a block that isn't in the tree, the vote is dropped.
    ///
    /// Votes are propagated transitively towards the genesis. All blocks in the tree
    /// that result in at least alpha votes will record the poll on their children.
    /// Every other block will have an unsuccessful poll registered.
    ///
    /// After collecting which blocks should be voted on, the polls are registered
    /// and blocks are accepted/rejected as needed. The tail is then updated to equal
    /// the leaf on the preferred branch.
    ///
    /// To optimize the theoretical complexity of the vote propagation, a topological
    /// sort is done over the blocks that are reachable from the provided votes.
    /// During the sort, votes are pushed towards the genesis. To prevent interacting
    /// over all blocks that had unsuccessful polls, we set a flag on the block to
    /// know that any future traversal through that block should register an
    /// unsuccessful poll on that block and every descendant block.
    ///
    /// The complexity of this function is:
    /// - runtime = 3 * |live set| + |votes|
    /// - space = 2 * |live set| + |votes|
    ///
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Topological.RecordPoll>
    pub fn record_poll(&self, votes_bag: Bag) -> Result<()> {
        // register a new poll call
        self.poll_number.set(self.poll_number.get() + 1);

        let mut votes_stack = {
            if votes_bag.len() >= self.parameters.alpha as u32 {
                // received at least alpha votes, thus possibly reached
                // an alpha majority on the processing block
                // must perform the traversals to calculate all blocks
                // that reached an alpha majority

                // updates [self.leaves] and [self.kahn_nodes]
                // runtime = |live set| + |votes|
                // space = |live set| + |votes|
                self.calculate_in_degree(votes_bag);

                // runtime = |live set|
                // space = |live set|
                self.push_votes()
            } else {
                // if there is no way for an alpha majority to occur,
                // there is no need to perform any traversals
                Vec::new()
            }
        };
        self.apply_votes_stack(&mut votes_stack)?;

        // If the set of preferred IDs already contains the preference, then the
        // tail is guaranteed to already be set correctly. This is because the value
        // returned from vote reports the next preferred block after the last
        // preferred block that was voted for. If this block was previously
        // preferred, then we know that following the preferences down the chain
        // will return the current tail.
        let last_preferred = self.last_preferred.get();
        if self.preferred_ids.borrow().contains(&last_preferred) {
            return Ok(());
        }

        // runtime = |live set|
        // space = constant
        self.preferred_ids.borrow_mut().clear();
        self.tail.set(last_preferred);

        // not mutating (e.g., insert/remove) blocks itself, thus read-only borrower
        let borrowed_blks = self.blocks.borrow();

        // runtime = |live set|
        // space = constant
        // traverse from the preferred Id to the last accepted ancestor
        let mut cursor = borrowed_blks.get(&self.tail.get()).expect("unexpected missing block for tail (possibly hash collsion of previously accepted block)").borrow();
        while !cursor.accepted() {
            let blk_id = cursor.blk.as_ref().unwrap().borrow().id();
            self.preferred_ids.borrow_mut().insert(blk_id);

            let parent_id = cursor.blk.as_ref().unwrap().borrow().parent();
            cursor = borrowed_blks.get(&parent_id).unwrap().borrow();
        }

        // traverse from the preferred Id to the preferred child
        // until there are no child
        let mut cursor = borrowed_blks.get(&self.tail.get()).expect("unexpected missing block for tail (possibly hash collsion of previously accepted block)").borrow();
        while cursor.sb.is_some() {
            let cur_preference = cursor.sb.as_ref().unwrap().preference();
            self.tail.set(cur_preference);

            self.preferred_ids.borrow_mut().insert(cur_preference);

            cursor = borrowed_blks.get(&cur_preference).unwrap().borrow();
        }

        Ok(())
    }

    /// Takes the list of votes and sets up the topological ordering.
    /// Finds the reachable section of the graph annotated with the number
    /// of inbound edges and the non-transitively applied votes.
    /// Also, updates the list of leaf blocks.
    /// ref. "avalanchego/snow/consensus/snowman#Topological.calculateInDegree"
    fn calculate_in_degree(&self, votes_bag: Bag) {
        // not mutating (e.g., insert/remove) blocks itself, thus read-only borrower
        let borrowed_blks = self.blocks.borrow();

        self.leaves.borrow_mut().clear();
        self.kahn_nodes.borrow_mut().clear();

        for vote in votes_bag.list() {
            let res = borrowed_blks.get(&vote);
            if res.is_none() {
                // vote for a block that is not in the tree (current pending set)
                // thus drop the vote
                continue;
            }
            let voted_block = res.unwrap();
            if voted_block.borrow().accepted() {
                // if the vote is for the last accepted block, the vote is dropped
                continue;
            }

            // parent contains the snowball instance of its children
            let parent_blk_id = voted_block.borrow().blk.as_ref().unwrap().borrow().parent();

            // add votes for this block to the parent's set of responses
            let num_votes = votes_bag.count(&vote);

            let (parent_kahn, previously_seen) = {
                let kahn = self.kahn_nodes.borrow_mut().remove(&parent_blk_id);
                if kahn.is_some() {
                    let kahn = kahn.unwrap();
                    kahn.votes.add_count(&vote, num_votes);

                    // if the parent block already had registered votes,
                    // then there is no need to iterate into the parents
                    (kahn, true)
                } else {
                    let kahn = KahnNode::new();
                    kahn.votes.add_count(&vote, num_votes);

                    (kahn, false)
                }
            };

            self.kahn_nodes
                .borrow_mut()
                .insert(parent_blk_id, parent_kahn);

            if previously_seen {
                continue;
            }

            // never seen this parent block before, thus currently a leaf
            self.leaves.borrow_mut().insert(parent_blk_id);

            // iterate through all the block's ancestors
            // set up the in_degree of the blocks
            let mut cursor = parent_blk_id;
            while let Some(blk) = borrowed_blks.get(&cursor) {
                if blk.borrow().accepted() {
                    break;
                }

                cursor = blk.borrow().blk.as_ref().unwrap().borrow().parent();

                let in_degree = {
                    // let Some(k) = ....borrow() then the else-case "borrow_mut" will
                    // "already borrowed: BorrowMutError"
                    let mut borrowed_mut_kahn_nodes = self.kahn_nodes.borrow_mut();

                    if let Some(kahn) = borrowed_mut_kahn_nodes.get(&cursor) {
                        kahn.in_degree.set(kahn.in_degree.get() + 1);
                        kahn.in_degree.get()
                    } else {
                        let kahn_node = KahnNode::new();
                        kahn_node.in_degree.set(kahn_node.in_degree.get() + 1);
                        let in_degree = kahn_node.in_degree.get();
                        borrowed_mut_kahn_nodes.insert(cursor, kahn_node);
                        in_degree
                    }
                };

                // already seen this block before (>= 1 before incremented)
                // then we shouldn't increase the in_degree of the ancestors
                // through this block again
                if in_degree != 1 {
                    break;
                }

                //  transitively seeing this block for the first time,
                // either the block was previously unknown or previously a leaf
                // regardless, shouldn't be tracked as a leaf
                self.leaves.borrow_mut().remove(&cursor);
            }
        }
    }

    /// Convert the tree into a branch of snowball instances with at least
    /// alpha votes.
    /// ref. "avalanchego/snow/consensus/snowman#Topological.pushVotes"
    fn push_votes(&self) -> Vec<Votes> {
        // not mutating (e.g., insert/remove) blocks itself, thus read-only borrower
        let borrowed_blks = self.blocks.borrow();

        let mut leaves: VecDeque<Id> = self.leaves.borrow_mut().drain().collect();
        let mut votes_stack: Vec<Votes> = Vec::with_capacity(self.kahn_nodes.borrow().len());

        while !leaves.is_empty() {
            let leaf_blk_id = leaves.pop_front().unwrap();

            // remove an inbound edge from the parent kahn node and push the votes
            let mut borrowed_mut_kahn_nodes = self.kahn_nodes.borrow_mut();

            // get the block and sort information about the block
            let kahn = borrowed_mut_kahn_nodes.get(&leaf_blk_id).unwrap();
            let kahn_votes = kahn.votes.deep_copy();
            let kahn_votes_len = kahn_votes.len();

            // at least alpha votes, then this block needs to record
            // the poll on the snowball instance
            if kahn_votes.len() >= self.parameters.alpha as u32 {
                votes_stack.push(Votes::new(leaf_blk_id, kahn_votes));
            }

            let blk = borrowed_blks.get(&leaf_blk_id).unwrap();
            if blk.borrow().accepted() {
                // if the block is accepted, no need to push votes to parent block
                continue;
            }

            let parent_blk_id = blk.borrow().blk.as_ref().unwrap().borrow().parent();

            let parent_kahn = borrowed_mut_kahn_nodes.get_mut(&parent_blk_id).unwrap();
            parent_kahn.in_degree.set(parent_kahn.in_degree.get() - 1);
            parent_kahn.votes.add_count(&leaf_blk_id, kahn_votes_len);

            // if in_degree is zero, then the parent node is now a leaf
            if parent_kahn.in_degree.get() == 0 {
                leaves.push_back(parent_blk_id);
            }
        }

        if !leaves.is_empty() {
            let mut borrowed_mut_leaves = self.leaves.borrow_mut();
            for id in leaves {
                borrowed_mut_leaves.insert(id);
            }
        }

        votes_stack
    }

    /// Apply votes to the branch that received an alpha threshold
    /// and returns the next preferred block Id after the last preferred
    /// block that received an alpha threshold.
    ///
    /// runtime = |live set|
    /// space = constant
    ///
    /// ref. "avalanchego/snow/consensus/snowman#Topological.vote"
    fn apply_votes_stack(&self, votes_stack: &mut Vec<Votes>) -> Result<()> {
        if votes_stack.is_empty() {
            // not mutating (e.g., insert/remove) blocks itself, thus read-only borrower
            let borrowed_blks = self.blocks.borrow();

            let head_blk = borrowed_blks
                .get(&self.head.get())
                .expect("head block not found in 'blocks'");
            head_blk.borrow_mut().should_falter.set(true);

            // don't use "self.num_processing" when "blocks" is already being borrowed
            let num_processing = self.num_processing();
            if num_processing > 0 {
                log::debug!(
                    "no progress was made after a vote with {} pending blocks",
                    num_processing
                );
            }

            // avalanchego#vote just returns "self.tail" with recursion
            // thus just track it as "last_preferred" field
            self.last_preferred.set(self.tail.get());
            return Ok(());
        }

        // keep track of new preferred block
        self.last_preferred.set(self.head.get());
        self.currently_on_preferred_branch.set(true);

        let mut poll_successful = false;
        while let Some(votes) = votes_stack.pop() {
            // check the block that we are going to vote on
            if !self.block_exists(&votes.parent_id) {
                // block block we are going to vote on was already rejected,
                // thus stop applying the votes
                break;
            }

            let (prev_should_falter, votes_finalized, poll_success) = self.record_votes(&votes);
            poll_successful = poll_successful || poll_success;

            // buffer deletes from "blocks" map as updating preference
            // still requires block information
            let mut pending_removals: HashSet<Id> = HashSet::new();

            // only accept when you are finalized and the head
            if votes_finalized && self.head.get() == votes.parent_id {
                let mut rejected = self.accept_preferred_child(&votes.parent_id)?;
                while let Some(reject_id) = rejected.pop() {
                    pending_removals.insert(reject_id);

                    let pending_rejects = self.reject_block_transitively(&reject_id)?;
                    rejected.extend_from_slice(&pending_rejects);
                }

                // by accepting the child of parent_block,
                // the last accepted block is no longer votes.parent_id, but its child
                // thus, votes.parent_id can be removed from the tree
                pending_removals.insert(votes.parent_id);
            }

            let next_id = {
                // Id of the child that is having a record_poll called
                // all other children will need to have their confidence reset
                // if there isn't a child having record_poll called,
                // then the next_id defaults to empty
                if votes_stack.is_empty() {
                    Id::empty()
                } else {
                    votes_stack.last().unwrap().parent_id
                }
            };
            self.update_preferences(prev_should_falter, &votes.parent_id, next_id)?;

            self.remove_blocks(&pending_removals);
        }

        // TODO: increment metric
        if poll_successful {
            log::debug!("poll was successful");
        } else {
            log::debug!("poll was not successful");
        }

        Ok(())
    }

    /// ref. "avalanchego/snow/consensus/snowman#Topological.vote"
    fn block_exists(&self, blk_id: &Id) -> bool {
        // not mutating (e.g., insert/remove) blocks itself, thus read-only borrower
        self.blocks.borrow().get(blk_id).is_some()
    }

    fn remove_blocks(&self, blk_ids: &HashSet<Id>) {
        if blk_ids.is_empty() {
            return;
        }

        // remove from the "blocks" itself, thus requires mutable borrower
        let mut borrowed_mut_blks = self.blocks.borrow_mut();
        for blk_id in blk_ids.iter() {
            borrowed_mut_blks.remove(blk_id);
        }
    }

    /// Returns the previous boolean value of the votes parent block.
    /// Returns true if the votes are finalized.
    /// Returns true if the poll was successful.
    /// ref. "avalanchego/snow/consensus/snowman#Topological.vote"
    fn record_votes(&self, votes: &Votes) -> (bool, bool, bool) {
        // not mutating (e.g., insert/remove) blocks itself, thus read-only borrower
        let borrowed_blks = self.blocks.borrow();
        let parent_blk = borrowed_blks
            .get(&votes.parent_id)
            .expect("votes.parent_id not found in 'blocks', must be rejected");

        // keep track of transitive falters to propagate to this block's children
        let prev_should_falter = parent_blk.borrow().should_falter.get();
        if prev_should_falter {
            // update falter to propagate to this block's children
            log::debug!("resetting confidence below parent Id {}", votes.parent_id);

            let borrowed_parent_blk = parent_blk.borrow();
            let sb = borrowed_parent_blk.sb.as_ref().unwrap();

            sb.record_unsuccessful_poll();
            borrowed_parent_blk.should_falter.set(false);
        }

        let mut borrowed_mut_parent_blk = parent_blk.borrow_mut();
        let sb = borrowed_mut_parent_blk.sb.as_mut().unwrap();
        let poll_successful = sb.record_poll(&votes.votes);

        (prev_should_falter, sb.finalized(), poll_successful)
    }

    /// Accepts the preferred child of the provided snowman block.
    /// By accepted the preferred child, all other children will be rejected.
    /// When these children are rejected, all their descendants will be rejected.
    ///
    /// When we are accepting a block, we are accepting it because the parent
    /// snowball instance has finalized. And the preference of the snowball
    /// instance after finalization is that block's ID.
    ///
    /// ref. "avalanchego/snow/consensus/snowman#Topological.accept"
    fn accept_preferred_child(&self, parent_blk_id: &Id) -> Result<Vec<Id>> {
        // not mutating (e.g., insert/remove) blocks itself, thus read-only borrower
        let borrowed_blks = self.blocks.borrow();
        let parent_blk = borrowed_blks
            .get(parent_blk_id)
            .expect("block Id not found in 'blocks', must be rejected");

        let borrowed_parent_blk = parent_blk.borrow();
        let preference = borrowed_parent_blk
            .sb
            .as_ref()
            .expect("unexpected None snowball for SnowmanBlock")
            .preference();

        // we are finalizing the block's child
        let children = borrowed_parent_blk
            .children
            .as_ref()
            .expect("unexpected None children for SnowmanBlock");
        let mut borrowed_mut_children = children.borrow_mut();
        let child_blk = borrowed_mut_children.get_mut(&preference).expect(
            format!(
                "missing child in SnowmanBlock for preference {}",
                preference
            )
            .as_str(),
        );

        log::info!("accepting the block {}", preference);
        child_blk.borrow_mut().accept()?;

        // because this is the newest accepted block,
        // this is the new head
        self.head.set(preference);
        self.height.set(child_blk.borrow().height());

        // remove the decided block from the set of processing Ids,
        // as its status now implies its preferredness
        let mut preferred_ids = self.preferred_ids.take();
        preferred_ids.remove(&preference);

        if borrowed_mut_children.is_empty() {
            return Ok(Vec::new());
        }

        // because self.blocks contains the last accepted block,
        // we don't delete the block from the blocks map here
        let mut rejected: Vec<Id> = Vec::with_capacity(borrowed_mut_children.len() - 1);
        for (child_id, child) in borrowed_mut_children.iter_mut() {
            if *child_id == preference {
                // don't reject the block we just accepted
                continue;
            }

            log::debug!(
                "rejecting {} due to conflict with the accepted block {}",
                child_id,
                preference
            );
            child.borrow_mut().reject()?;

            rejected.push(*child_id);
        }

        Ok(rejected)
    }

    /// Reject all descendants of the block we just rejected.
    /// Returns the children of rejected blocks.
    /// ref. "avalanchego/snow/consensus/snowman#Topological.rejectTransitively"
    fn reject_block_transitively(&self, blk_id: &Id) -> Result<Vec<Id>> {
        let borrowed_blks = self.blocks.borrow();
        let blk = borrowed_blks.get(blk_id).unwrap();

        let mut pending_rejects: Vec<Id> = Vec::new();
        let mut borrowed_mut_blk = blk.borrow_mut();
        if let Some(children) = borrowed_mut_blk.children.as_mut() {
            for (child_id, child) in children.borrow_mut().iter_mut() {
                log::debug!(
                    "rejecting the child {} for transitivity from its parent {}",
                    child_id,
                    blk_id
                );
                child.borrow_mut().reject()?;
                pending_rejects.push(*child_id);
            }
        }
        Ok(pending_rejects)
    }

    /// ref. "avalanchego/snow/consensus/snowman#Topological.vote"
    fn update_preferences(
        &self,
        prev_should_falter: bool,
        parent_id: &Id,
        next_id: Id,
    ) -> Result<()> {
        // not mutating (e.g., insert/remove) blocks itself, thus read-only borrower
        let borrowed_blks = self.blocks.borrow();
        let parent_blk = borrowed_blks.get(parent_id).unwrap();

        let borrowed_parent_blk = parent_blk.borrow();

        // if we are on the preferred branch, then the parent's preference is
        // the next block on the preferred branch
        let sb = borrowed_parent_blk.sb.as_ref().unwrap();
        let parent_preference = sb.preference();
        if self.currently_on_preferred_branch.get() {
            self.last_preferred.set(parent_preference);
        }

        // if we are on the preferred branch and the next_id is the preference of
        // the snowball instance, then we are following the preferred branch
        self.currently_on_preferred_branch
            .set(self.currently_on_preferred_branch.get() && next_id == parent_preference);

        // if there wasn't an alpha threshold on the branch
        // (either on this vote or a past transitive vote),
        // this should falter now
        let children = borrowed_parent_blk.children.as_ref().unwrap();
        for (child_id, _) in children.borrow().iter() {
            if !prev_should_falter && *child_id == next_id {
                // if we don't need to transitively falter and the child is going to
                // have RecordPoll called on it, then there is no reason to reset
                // the block's confidence
                continue;
            }

            // If we finalized a child of the current block, then all other
            // children will have been rejected and removed from the tree.
            // Therefore, we need to make sure the child is still in the tree.
            let res = borrowed_blks.get(child_id);
            if res.is_none() {
                continue;
            }
            let child_blk = res.unwrap();

            log::debug!(
                "deferring confidence reset of {}, voting for {}",
                child_id,
                next_id
            );

            // if the child is ever voted for positively,
            // the confidence must be reset first
            child_blk.borrow().should_falter.set(true);
        }

        Ok(())
    }
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_initialize --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#InitializeTest"
#[test]
fn test_topological_initialize() {
    use crate::snowman::block::test_block::TestBlock;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let genesis_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 3,
        beta_rogue: 5,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, genesis_blk) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_id, genesis_height)
            .expect("failed to create Topological");

    assert!(genesis_blk.borrow().accepted());
    assert_eq!(tp.parameters().k, 1);
    assert_eq!(tp.parameters().alpha, 1);
    assert_eq!(tp.parameters().beta_virtuous, 3);
    assert_eq!(tp.parameters().beta_rogue, 5);
    assert_eq!(tp.parameters().concurrent_repolls, 1);
    assert_eq!(tp.parameters().optimal_processing, 1);
    assert_eq!(tp.parameters().max_outstanding_items, 1);
    assert_eq!(tp.parameters().max_item_processing_time, 1);

    assert_eq!(tp.preference(), genesis_id);
    assert_eq!(tp.height(), genesis_height);

    assert!(tp.finalized());
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_num_processing --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#NumProcessingTest"
#[test]
fn test_topological_num_processing() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 1,
        beta_rogue: 1,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_id, genesis_height)
            .expect("failed to create Topological");

    assert_eq!(tp.num_processing(), 0);

    let blk_id = Id::empty().prefix(&[1]).unwrap();
    let blk = TestBlock::new(
        TestDecidable::new(blk_id, Status::Processing),
        genesis_id.clone(),
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk.id(), blk_id);
    assert_eq!(blk.parent(), genesis_id);

    // adding to the previous preference will update the preference
    assert!(tp.add_block(blk.clone()).is_ok());

    assert_eq!(tp.num_processing(), 1);

    let votes_bag = Bag::new();
    votes_bag.add_count(&blk.id(), 1);
    assert!(tp.record_poll(votes_bag).is_ok());

    assert_eq!(tp.num_processing(), 0);
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_add_to_tail --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#AddToTailTest"
#[test]
fn test_topological_add_to_tail() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 3,
        beta_rogue: 5,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) = Topological::<TestBlock>::new(
        Context::default(),
        params.clone(),
        genesis_blk_id,
        genesis_height,
    )
    .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk1 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk1.id(), Id::empty().prefix(&[1]).unwrap());
    let blk1_rc = Rc::new(RefCell::new(SnowmanBlock::new_with_block(
        params,
        Rc::new(RefCell::new(blk1.clone())),
    )));

    let blk2 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[2]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk2.id(), Id::empty().prefix(&[2]).unwrap());

    // adding to the previous preference will update the preference
    assert!(tp.add_block(blk1.clone()).is_ok());
    assert_eq!(tp.preference(), blk1.id());
    assert!(tp.block_preferred(Rc::clone(&blk1_rc)));

    // adding to something other than the previous preference
    // won't update the preference
    assert!(tp.add_block(blk2.clone()).is_ok());
    assert_eq!(tp.preference(), blk1.id());
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_add_to_non_tail --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#AddToNonTailTest"
#[test]
fn test_topological_add_to_non_tail() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 3,
        beta_rogue: 5,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) = Topological::<TestBlock>::new(
        Context::default(),
        params.clone(),
        genesis_blk_id,
        genesis_height,
    )
    .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk1 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk1.id(), Id::empty().prefix(&[1]).unwrap());
    let blk1_rc = Rc::new(RefCell::new(SnowmanBlock::new_with_block(
        params,
        Rc::new(RefCell::new(blk1.clone())),
    )));

    let blk2 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[2]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk2.id(), Id::empty().prefix(&[2]).unwrap());

    // adding to the previous preference will update the preference
    assert!(tp.add_block(blk1.clone()).is_ok());
    assert_eq!(tp.preference(), blk1.id());
    assert!(tp.block_preferred(Rc::clone(&blk1_rc)));

    // adding to something other than the previous preference
    // won't update the preference
    assert!(tp.add_block(blk2.clone()).is_ok());
    assert_eq!(tp.preference(), blk1.id());
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_add_unknown --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#AddToUnknownTest"
#[test]
fn test_topological_add_unknown() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 3,
        beta_rogue: 5,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let parent_blk = TestBlock::new(
        TestDecidable::new(
            Id::empty().prefix(&[1]).unwrap(),
            Status::Unknown(String::from("...")),
        ),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(parent_blk.id(), Id::empty().prefix(&[1]).unwrap());
    assert_eq!(parent_blk.parent(), genesis_blk_id);

    let blk = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[2]).unwrap(), Status::Processing),
        parent_blk.id(),
        Ok(()),
        Bytes::new(),
        parent_blk.height() + 1,
        0,
    );
    assert_eq!(blk.id(), Id::empty().prefix(&[2]).unwrap());
    assert_eq!(blk.parent(), parent_blk.id());

    // adding a block with an unknown parent means the parent
    // must have already been rejected
    // thus the block should be immediately rejected
    let added_blk_rc = tp.add_block(blk.clone()).unwrap();
    assert_eq!(added_blk_rc.borrow().status().unwrap(), Status::Rejected);
    assert_eq!(tp.preference(), genesis_blk_id);
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_status_or_processing_previously_accepted --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#StatusOrProcessingPreviouslyAcceptedTest"
#[test]
fn test_topological_status_or_processing_previously_accepted() {
    use crate::snowman::block::test_block::TestBlock;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 3,
        beta_rogue: 5,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, genesis_blk) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let borrowed_genesis_blk = genesis_blk.borrow();
    assert!(borrowed_genesis_blk.accepted());

    assert!(!tp.block_processing(genesis_blk_id));
    assert!(tp.block_decided(Rc::clone(&genesis_blk)));
    assert!(tp.block_preferred(Rc::clone(&genesis_blk)));
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_status_or_processing_previously_rejected --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#StatusOrProcessingPreviouslyRejectedTest"
#[test]
fn test_topological_status_or_processing_previously_rejected() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 3,
        beta_rogue: 5,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) = Topological::<TestBlock>::new(
        Context::default(),
        params.clone(),
        genesis_blk_id,
        genesis_height,
    )
    .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Rejected),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk.id(), Id::empty().prefix(&[1]).unwrap());
    assert_eq!(blk.status(), Status::Rejected);
    let blk_rc = Rc::new(RefCell::new(SnowmanBlock::new_with_block(
        params,
        Rc::new(RefCell::new(blk.clone())),
    )));

    assert!(!tp.block_processing(blk.id()));
    assert!(tp.block_decided(Rc::clone(&blk_rc)));
    assert!(!tp.block_preferred(Rc::clone(&blk_rc)));
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_status_or_processing_unissued --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#StatusOrProcessingUnissuedTest"
#[test]
fn test_topological_status_or_processing_unissued() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 3,
        beta_rogue: 5,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) = Topological::<TestBlock>::new(
        Context::default(),
        params.clone(),
        genesis_blk_id,
        genesis_height,
    )
    .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk.id(), Id::empty().prefix(&[1]).unwrap());
    assert_eq!(blk.status(), Status::Processing);
    let snowman_blk_rc = Rc::new(RefCell::new(SnowmanBlock::new_with_block(
        params,
        Rc::new(RefCell::new(blk.clone())),
    )));

    assert!(!tp.block_processing(blk.id()));
    assert!(!tp.block_decided(Rc::clone(&snowman_blk_rc)));
    assert!(!tp.block_preferred(Rc::clone(&snowman_blk_rc)));
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_status_or_processing_issued --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#StatusOrProcessingIssuedTest"
#[test]
fn test_topological_status_or_processing_issued() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 3,
        beta_rogue: 5,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) = Topological::<TestBlock>::new(
        Context::default(),
        params.clone(),
        genesis_blk_id,
        genesis_height,
    )
    .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk.id(), Id::empty().prefix(&[1]).unwrap());
    assert_eq!(blk.status(), Status::Processing);

    let added_blk_rc = tp.add_block(blk.clone()).unwrap();
    assert_eq!(added_blk_rc.borrow().status().unwrap(), Status::Processing);

    assert!(tp.block_processing(blk.id()));
    assert!(!tp.block_decided(Rc::clone(&added_blk_rc)));
    assert!(tp.block_preferred(Rc::clone(&added_blk_rc)));
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_record_poll_accept_single_block --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#RecordPollAcceptSingleBlockTest"
#[test]
fn test_topological_record_poll_accept_single_block() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 2,
        beta_rogue: 3,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) = Topological::<TestBlock>::new(
        Context::default(),
        params.clone(),
        genesis_blk_id,
        genesis_height,
    )
    .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk.id(), Id::empty().prefix(&[1]).unwrap());
    assert_eq!(blk.status(), Status::Processing);

    let added_blk_rc = tp.add_block(blk.clone()).unwrap();
    assert_eq!(added_blk_rc.borrow().status().unwrap(), Status::Processing);

    let votes_bag = Bag::new();
    votes_bag.add_count(&blk.id(), 1);
    assert!(tp.record_poll(votes_bag).is_ok());

    assert!(!tp.finalized());
    assert_eq!(tp.preference(), blk.id());
    assert_eq!(added_blk_rc.borrow().status().unwrap(), Status::Processing);

    let votes_bag = Bag::new();
    votes_bag.add_count(&blk.id(), 1);
    assert!(tp.record_poll(votes_bag).is_ok());

    assert!(tp.finalized());
    assert_eq!(tp.preference(), blk.id());
    assert_eq!(added_blk_rc.borrow().status().unwrap(), Status::Accepted);
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_record_poll_accept_and_reject --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#RecordPollAcceptAndRejectTest"
#[test]
fn test_topological_record_poll_accept_and_reject() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 1,
        beta_rogue: 2,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk1 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk1.id(), Id::empty().prefix(&[1]).unwrap());
    assert_eq!(blk1.status(), Status::Processing);

    let blk2 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[2]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk2.id(), Id::empty().prefix(&[2]).unwrap());
    assert_eq!(blk2.status(), Status::Processing);

    let added_blk1_rc = tp.add_block(blk1.clone()).unwrap();
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk2_rc = tp.add_block(blk2.clone()).unwrap();
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);

    let votes_bag = Bag::new();
    votes_bag.add_count(&blk1.id(), 1);
    assert!(tp.record_poll(votes_bag).is_ok());

    assert!(!tp.finalized());
    assert_eq!(tp.preference(), blk1.id());
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);

    let votes_bag = Bag::new();
    votes_bag.add_count(&blk1.id(), 1);
    assert!(tp.record_poll(votes_bag).is_ok());

    assert!(tp.finalized());
    assert_eq!(tp.preference(), blk1.id());
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Accepted);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Rejected);
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_record_poll_split_vote_no_change --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#RecordPollSplitVoteNoChangeTest"
#[test]
fn test_topological_record_poll_split_vote_no_change() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 2,
        alpha: 2,
        beta_virtuous: 1,
        beta_rogue: 2,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk1 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk1.id(), Id::empty().prefix(&[1]).unwrap());
    assert_eq!(blk1.status(), Status::Processing);

    let blk2 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[2]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk2.id(), Id::empty().prefix(&[2]).unwrap());
    assert_eq!(blk2.status(), Status::Processing);

    let added_blk1_rc = tp.add_block(blk1.clone()).unwrap();
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk2_rc = tp.add_block(blk2.clone()).unwrap();
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);

    let votes_bag = Bag::new();
    votes_bag.add_count(&blk1.id(), 1);
    votes_bag.add_count(&blk2.id(), 1);

    // The first poll will accept shared bits
    assert!(tp.record_poll(votes_bag.clone()).is_ok());

    assert!(!tp.finalized());
    assert_eq!(tp.preference(), blk1.id());
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);

    // TODO: check metrics for "polls_failed" and "polls_successful"

    // The second poll will do nothing
    assert!(tp.record_poll(votes_bag).is_ok());

    assert!(!tp.finalized());
    assert_eq!(tp.preference(), blk1.id());
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);

    // TODO: check metrics for "polls_failed" and "polls_successful"
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_record_poll_when_finalized --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#RecordPollWhenFinalizedTest"
#[test]
fn test_topological_record_poll_when_finalized() {
    use crate::snowman::block::test_block::TestBlock;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 1,
        beta_rogue: 2,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, added_genesis_blk_rc) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);
    assert!(added_genesis_blk_rc.borrow().id().is_none());

    let votes_bag = Bag::new();
    votes_bag.add_count(&genesis_blk_id, 1);
    assert!(tp.record_poll(votes_bag).is_ok());

    assert!(tp.finalized());
    assert_eq!(tp.preference(), genesis_blk_id);
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_record_poll_reject_transitively --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#RecordPollRejectTransitivelyTest"
#[test]
fn test_topological_record_poll_reject_transitively() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 1,
        beta_rogue: 1,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) = Topological::<TestBlock>::new(
        Context::default(),
        params.clone(),
        genesis_blk_id,
        genesis_height,
    )
    .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk0 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk0.id(), Id::empty().prefix(&[1]).unwrap());
    assert_eq!(blk0.status(), Status::Processing);

    let blk1 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[2]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk1.id(), Id::empty().prefix(&[2]).unwrap());
    assert_eq!(blk1.status(), Status::Processing);

    let blk2 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[3]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk2.id(), Id::empty().prefix(&[3]).unwrap());
    assert_eq!(blk2.status(), Status::Processing);

    let added_blk0_rc = tp.add_block(blk0.clone()).unwrap();
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk1_rc = tp.add_block(blk1.clone()).unwrap();
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk2_rc = tp.add_block(blk2.clone()).unwrap();
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);

    // Current graph structure:
    //   G
    //  / \
    // 0   1
    //     |
    //     2
    // Tail = 0

    let votes_bag = Bag::new();
    votes_bag.add_count(&blk0.id(), 1);
    assert!(tp.record_poll(votes_bag).is_ok());

    // Current graph structure:
    // 0
    // Tail = 0

    assert!(tp.finalized());
    assert_eq!(tp.preference(), blk0.id());
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Accepted);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Rejected);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Rejected);
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_record_poll_transitively_reset_confidence --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#RecordPollTransitivelyResetConfidenceTest"
#[test]
fn test_topological_record_poll_transitively_reset_confidence() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 2,
        beta_rogue: 2,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) = Topological::<TestBlock>::new(
        Context::default(),
        params.clone(),
        genesis_blk_id,
        genesis_height,
    )
    .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk0 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk0.id(), Id::empty().prefix(&[1]).unwrap());
    assert_eq!(blk0.status(), Status::Processing);

    let blk1 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[2]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk1.id(), Id::empty().prefix(&[2]).unwrap());
    assert_eq!(blk1.status(), Status::Processing);

    let blk2 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[3]).unwrap(), Status::Processing),
        blk1.id(),
        Ok(()),
        Bytes::new(),
        blk1.height() + 1,
        0,
    );
    assert_eq!(blk2.id(), Id::empty().prefix(&[3]).unwrap());
    assert_eq!(blk2.status(), Status::Processing);

    let blk3 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[4]).unwrap(), Status::Processing),
        blk1.id(),
        Ok(()),
        Bytes::new(),
        blk1.height() + 1,
        0,
    );
    assert_eq!(blk3.id(), Id::empty().prefix(&[4]).unwrap());
    assert_eq!(blk3.status(), Status::Processing);

    let added_blk0_rc = tp.add_block(blk0.clone()).unwrap();
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk1_rc = tp.add_block(blk1.clone()).unwrap();
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk2_rc = tp.add_block(blk2.clone()).unwrap();
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk3_rc = tp.add_block(blk3.clone()).unwrap();
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Processing);

    // Current graph structure:
    //   G
    //  / \
    // 0   1
    //    / \
    //   2   3

    let votes_bag_for_2 = Bag::new();
    votes_bag_for_2.add_count(&blk2.id(), 1);
    assert!(tp.record_poll(votes_bag_for_2).is_ok());

    assert!(!tp.finalized());
    assert_eq!(tp.preference(), blk2.id());
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Processing);

    let votes_bag_empty = Bag::new();
    assert!(tp.record_poll(votes_bag_empty).is_ok());

    assert!(!tp.finalized());
    assert_eq!(tp.preference(), blk2.id());
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Processing);

    let votes_bag_for_2 = Bag::new();
    votes_bag_for_2.add_count(&blk2.id(), 1);
    assert!(tp.record_poll(votes_bag_for_2).is_ok());

    assert!(!tp.finalized());
    assert_eq!(tp.preference(), blk2.id());
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Processing);

    let votes_bag_for_3 = Bag::new();
    votes_bag_for_3.add_count(&blk3.id(), 1);
    assert!(tp.record_poll(votes_bag_for_3).is_ok());

    assert!(!tp.finalized());
    assert_eq!(tp.preference(), blk2.id());
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Rejected);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Accepted);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Processing);

    let votes_bag_for_3 = Bag::new();
    votes_bag_for_3.add_count(&blk3.id(), 1);
    assert!(tp.record_poll(votes_bag_for_3).is_ok());

    assert!(tp.finalized());
    assert_eq!(tp.preference(), blk3.id());
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Rejected);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Accepted);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Rejected);
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Accepted);
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_record_poll_invalid_vote --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#RecordPollInvalidVoteTest"
#[test]
fn test_topological_record_poll_invalid_vote() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 2,
        beta_rogue: 2,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk.id(), Id::empty().prefix(&[1]).unwrap());
    assert_eq!(blk.status(), Status::Processing);

    let added_blk_rc = tp.add_block(blk.clone()).unwrap();
    assert_eq!(added_blk_rc.borrow().status().unwrap(), Status::Processing);

    let valid_votes_bag = Bag::new();
    valid_votes_bag.add_count(&blk.id(), 1);
    assert!(tp.record_poll(valid_votes_bag).is_ok());

    let invalid_votes_bag = Bag::new();
    let unknown_blk_id = Id::empty().prefix(&[2]).unwrap();
    invalid_votes_bag.add_count(&unknown_blk_id, 1);
    assert!(tp.record_poll(invalid_votes_bag).is_ok());

    let valid_votes_bag = Bag::new();
    valid_votes_bag.add_count(&blk.id(), 1);
    assert!(tp.record_poll(valid_votes_bag).is_ok());

    assert!(!tp.finalized());
    assert_eq!(tp.preference(), blk.id());
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_record_poll_transitive_voting --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#RecordPollTransitiveVotingTest"
#[test]
fn test_topological_record_poll_transitive_voting() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 3,
        alpha: 3,
        beta_virtuous: 1,
        beta_rogue: 1,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk0 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk0.id(), Id::empty().prefix(&[1]).unwrap());
    assert_eq!(blk0.status(), Status::Processing);

    let blk1 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[2]).unwrap(), Status::Processing),
        blk0.id(),
        Ok(()),
        Bytes::new(),
        blk0.height() + 1,
        0,
    );
    assert_eq!(blk1.id(), Id::empty().prefix(&[2]).unwrap());
    assert_eq!(blk1.status(), Status::Processing);

    let blk2 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[3]).unwrap(), Status::Processing),
        blk1.id(),
        Ok(()),
        Bytes::new(),
        blk1.height() + 1,
        0,
    );
    assert_eq!(blk2.id(), Id::empty().prefix(&[3]).unwrap());
    assert_eq!(blk2.status(), Status::Processing);

    let blk3 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[4]).unwrap(), Status::Processing),
        blk0.id(),
        Ok(()),
        Bytes::new(),
        blk0.height() + 1,
        0,
    );
    assert_eq!(blk3.id(), Id::empty().prefix(&[4]).unwrap());
    assert_eq!(blk3.status(), Status::Processing);

    let blk4 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[5]).unwrap(), Status::Processing),
        blk3.id(),
        Ok(()),
        Bytes::new(),
        blk3.height() + 1,
        0,
    );
    assert_eq!(blk4.id(), Id::empty().prefix(&[5]).unwrap());
    assert_eq!(blk4.status(), Status::Processing);

    let added_blk0_rc = tp.add_block(blk0.clone()).unwrap();
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk1_rc = tp.add_block(blk1.clone()).unwrap();
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk2_rc = tp.add_block(blk2.clone()).unwrap();
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk3_rc = tp.add_block(blk3.clone()).unwrap();
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk4_rc = tp.add_block(blk4.clone()).unwrap();
    assert_eq!(added_blk4_rc.borrow().status().unwrap(), Status::Processing);

    // Current graph structure:
    //   G
    //   |
    //   0
    //  / \
    // 1   3
    // |   |
    // 2   4
    // Tail = 2

    let votes_0_2_4 = Bag::new();
    votes_0_2_4.add_count(&blk0.id(), 1);
    votes_0_2_4.add_count(&blk2.id(), 1);
    votes_0_2_4.add_count(&blk4.id(), 1);
    assert!(tp.record_poll(votes_0_2_4).is_ok());

    // Current graph structure:
    //   0
    //  / \
    // 1   3
    // |   |
    // 2   4
    // Tail = 2

    assert!(!tp.finalized());
    assert_eq!(tp.preference(), blk2.id());
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Accepted);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk4_rc.borrow().status().unwrap(), Status::Processing);

    let dep_2_2_2 = Bag::new();
    dep_2_2_2.add_count(&blk2.id(), 3);
    assert!(tp.record_poll(dep_2_2_2).is_ok());

    // Current graph structure:
    //   2
    // Tail = 2

    assert!(tp.finalized());
    assert_eq!(tp.preference(), blk2.id());
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Accepted);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Accepted);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Accepted);
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Rejected);
    assert_eq!(added_blk4_rc.borrow().status().unwrap(), Status::Rejected);
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_record_poll_diverged_voting --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#RecordPollDivergedVotingTest"
#[test]
fn test_topological_record_poll_diverged_voting() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 1,
        beta_rogue: 2,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk0 = TestBlock::new(
        TestDecidable::new(
            Id::from_slice(&[0x0f]), // 0b1111
            Status::Processing,
        ),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk0.id(), Id::from_slice(&[0x0f])); // 0b1111
    assert_eq!(blk0.status(), Status::Processing);

    let blk1 = TestBlock::new(
        TestDecidable::new(
            Id::from_slice(&[0x08]), // 0b1000
            Status::Processing,
        ),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk1.id(), Id::from_slice(&[0x08])); // 0b1000
    assert_eq!(blk1.status(), Status::Processing);

    let blk2 = TestBlock::new(
        TestDecidable::new(
            Id::from_slice(&[0x01]), // 0b0001
            Status::Processing,
        ),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk2.id(), Id::from_slice(&[0x01])); // 0b0001
    assert_eq!(blk2.status(), Status::Processing);

    let blk3 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        blk2.id(),
        Ok(()),
        Bytes::new(),
        blk2.height() + 1,
        0,
    );
    assert_eq!(blk3.id(), Id::empty().prefix(&[1]).unwrap());
    assert_eq!(blk3.status(), Status::Processing);

    let added_blk0_rc = tp.add_block(blk0.clone()).unwrap();
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk1_rc = tp.add_block(blk1.clone()).unwrap();
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);

    // The first bit is contested as either 0 or 1. When voting for [block0] and
    // when the first bit is 1, the following bits have been decided to follow
    // the 255 remaining bits of [block0].
    let votes_0 = Bag::new();
    votes_0.add_count(&blk0.id(), 1);
    assert!(tp.record_poll(votes_0).is_ok());

    // Although we are adding in [block2] here - the underlying snowball
    // instance has already decided it is rejected. Snowman doesn't actually
    // know that though, because that is an implementation detail of the
    // Snowball trie that is used.
    let added_blk2_rc = tp.add_block(blk2.clone()).unwrap();
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);

    // Because [block2] is effectively rejected, [block3] is also effectively
    // rejected.
    let added_blk3_rc = tp.add_block(blk3.clone()).unwrap();
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Processing);

    // Current graph structure:
    //       G
    //     /   \
    //    *     |
    //   / \    |
    //  0   2   1
    //      |
    //      3
    // Tail = 0

    // Transitively votes for [block2] by voting for its child [block3].
    // Because [block2] shares the first bit with [block0] and the following
    // bits have been finalized for [block0], the voting results in accepting
    // [block0]. When [block0] is accepted, [block1] and [block2] are rejected
    // as conflicting. [block2]'s child, [block3], is then rejected
    // transitively.
    let votes_3 = Bag::new();
    votes_3.add_count(&blk3.id(), 1);
    assert!(tp.record_poll(votes_3).is_ok());

    assert!(tp.finalized());
    assert_eq!(tp.preference(), blk0.id());
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Accepted);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Rejected);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Rejected);
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Rejected);
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_record_poll_diverged_voting_with_no_conflicting_bit --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#RecordPollDivergedVotingWithNoConflictingBitTest"
#[test]
fn test_topological_record_poll_diverged_voting_with_no_conflicting_bit() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 1,
        beta_rogue: 2,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk0 = TestBlock::new(
        TestDecidable::new(
            Id::from_slice(&[0x06]), // 0b0110
            Status::Processing,
        ),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk0.id(), Id::from_slice(&[0x06])); // 0b0110
    assert_eq!(blk0.status(), Status::Processing);

    let blk1 = TestBlock::new(
        TestDecidable::new(
            Id::from_slice(&[0x08]), // 0b1000
            Status::Processing,
        ),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk1.id(), Id::from_slice(&[0x08])); // 0b1000
    assert_eq!(blk1.status(), Status::Processing);

    let blk2 = TestBlock::new(
        TestDecidable::new(
            Id::from_slice(&[0x01]), // 0b0001
            Status::Processing,
        ),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk2.id(), Id::from_slice(&[0x01])); // 0b0001
    assert_eq!(blk2.status(), Status::Processing);

    let blk3 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        blk2.id(),
        Ok(()),
        Bytes::new(),
        blk2.height() + 1,
        0,
    );
    assert_eq!(blk3.id(), Id::empty().prefix(&[1]).unwrap());
    assert_eq!(blk3.status(), Status::Processing);

    let added_blk0_rc = tp.add_block(blk0.clone()).unwrap();
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk1_rc = tp.add_block(blk1.clone()).unwrap();
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);

    // When voting for [block0], we end up finalizing the first bit as 0. The
    // second bit is contested as either 0 or 1. For when the second bit is 1,
    // the following bits have been decided to follow the 254 remaining bits of
    // [block0].
    let votes_0 = Bag::new();
    votes_0.add_count(&blk0.id(), 1);
    assert!(tp.record_poll(votes_0).is_ok());

    // Although we are adding in [block2] here - the underlying snowball
    // instance has already decided it is rejected. Snowman doesn't actually
    // know that though, because that is an implementation detail of the
    // Snowball trie that is used.
    let added_blk2_rc = tp.add_block(blk2.clone()).unwrap();
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);

    // Because [block2] is effectively rejected, [block3] is also effectively
    // rejected.
    let added_blk3_rc = tp.add_block(blk3.clone()).unwrap();
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Processing);

    // Current graph structure:
    //       G
    //     /   \
    //    *     |
    //   / \    |
    //  0   1   2
    //          |
    //          3
    // Tail = 0

    // Transitively votes for [block2] by voting for its child [block3]. Because
    // [block2] doesn't share any processing bits with [block0] or [block1], the
    // votes are over only rejected bits. Therefore, the votes for [block2] are
    // dropped. Although the votes for [block3] are still applied, [block3] will
    // only be marked as accepted after [block2] is marked as accepted; which
    // will never happen.
    let votes_3 = Bag::new();
    votes_3.add_count(&blk3.id(), 1);
    assert!(tp.record_poll(votes_3).is_ok());

    assert!(!tp.finalized());
    assert_eq!(tp.preference(), blk0.id());
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Processing);
}

/*
/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_record_poll_diverged_voting_with_invalid_block_id_unknown_tail --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#RecordPollDivergedVotingWithInvalidBlockIDUnknownTailTest"
#[test]
fn test_topological_record_poll_diverged_voting_with_invalid_block_id_unknown_tail() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 1,
        beta_rogue: 2,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk0 = TestBlock::new(
        TestDecidable::new(
            Id::from_slice(&[0x03]), // 0b0011
            Status::Processing,
        ),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk0.id(), Id::from_slice(&[0x03])); // 0b0011
    assert_eq!(blk0.status(), Status::Processing);

    let blk1 = TestBlock::new(
        TestDecidable::new(
            Id::from_slice(&[0x08]), // 0b1000
            Status::Processing,
        ),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk1.id(), Id::from_slice(&[0x08])); // 0b1000
    assert_eq!(blk1.status(), Status::Processing);

    let blk2 = TestBlock::new(
        TestDecidable::new(
            Id::from_slice(&[0x01]), // 0b0001
            Status::Processing,
        ),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    assert_eq!(blk2.id(), Id::from_slice(&[0x01])); // 0b0001
    assert_eq!(blk2.status(), Status::Processing);

    let blk3 = TestBlock::new(
        TestDecidable::new(
            Id::from_slice(&[0x03]), // 0b0011
            Status::Processing,
        ),
        blk2.id(),
        Ok(()),
        Bytes::new(),
        blk2.height() + 1,
        0,
    );
    assert_eq!(blk3.id(), Id::from_slice(&[0x03]));
    assert_eq!(blk3.status(), Status::Processing);

    let added_blk0_rc = tp.add_block(blk0.clone()).unwrap();
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk1_rc = tp.add_block(blk1.clone()).unwrap();
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);

    let votes_0 = Bag::new();
    votes_0.add_count(&blk0.id(), 1);
    assert!(tp.record_poll(votes_0).is_ok());

    let added_blk2_rc = tp.add_block(blk2.clone()).unwrap();
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);

    let added_blk3_rc = tp.add_block(blk3.clone()).unwrap();
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Processing);
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Processing);

    // Current graph structure:
    //     G
    //  /  |  \
    // 0   1   2
    //         |
    //         3
    // Tail = 3

    // Transitively increases block2 confidence by voting for its child block3.
    // Because block2 shares the first bit with block0 but block0 was added first,
    // the voting results in accepting block0 instead.
    // When block2 is rejected, its child gets rejected transitively.
    let votes_3 = Bag::new();
    votes_3.add_count(&blk3.id(), 1);
    assert!(tp.record_poll(votes_3).is_ok());

    assert!(!tp.finalized());
    assert_eq!(tp.preference(), blk0.id());
    assert_eq!(added_blk0_rc.borrow().status().unwrap(), Status::Accepted);
    assert_eq!(added_blk1_rc.borrow().status().unwrap(), Status::Rejected);
    assert_eq!(added_blk2_rc.borrow().status().unwrap(), Status::Rejected);
    assert_eq!(added_blk3_rc.borrow().status().unwrap(), Status::Rejected);
}
*/

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_record_poll_change_preferred_chain --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#RecordPollChangePreferredChainTest"
#[test]
fn test_topological_record_poll_change_preferred_chain() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 10,
        beta_rogue: 10,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let a1 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    let b1 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[2]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    let a2 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[3]).unwrap(), Status::Processing),
        a1.id(),
        Ok(()),
        Bytes::new(),
        a1.height() + 1,
        0,
    );
    let b2 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[4]).unwrap(), Status::Processing),
        b1.id(),
        Ok(()),
        Bytes::new(),
        b1.height() + 1,
        0,
    );

    let a1_rc = tp.add_block(a1.clone()).unwrap();
    assert_eq!(a1_rc.borrow().status().unwrap(), Status::Processing);

    let a2_rc = tp.add_block(a2.clone()).unwrap();
    assert_eq!(a2_rc.borrow().status().unwrap(), Status::Processing);

    let b1_rc = tp.add_block(b1.clone()).unwrap();
    assert_eq!(b1_rc.borrow().status().unwrap(), Status::Processing);

    let b2_rc = tp.add_block(b2.clone()).unwrap();
    assert_eq!(b2_rc.borrow().status().unwrap(), Status::Processing);

    assert_eq!(tp.preference(), a2.id());
    assert!(tp.block_preferred(Rc::clone(&a1_rc)));
    assert!(tp.block_preferred(Rc::clone(&a2_rc)));
    assert!(!tp.block_preferred(Rc::clone(&b1_rc)));
    assert!(!tp.block_preferred(Rc::clone(&b2_rc)));

    let votes_b2 = Bag::new();
    votes_b2.add_count(&b2.id(), 1);
    assert!(tp.record_poll(votes_b2).is_ok());

    assert_eq!(tp.preference(), b2.id());
    assert!(!tp.block_preferred(Rc::clone(&a1_rc)));
    assert!(!tp.block_preferred(Rc::clone(&a2_rc)));
    assert!(tp.block_preferred(Rc::clone(&b1_rc)));
    assert!(tp.block_preferred(Rc::clone(&b2_rc)));

    let votes_a1 = Bag::new();
    votes_a1.add_count(&a1.id(), 1);
    assert!(tp.record_poll(votes_a1).is_ok());

    let votes_a1 = Bag::new();
    votes_a1.add_count(&a1.id(), 1);
    assert!(tp.record_poll(votes_a1).is_ok());

    assert_eq!(tp.preference(), a2.id());
    assert!(tp.block_preferred(Rc::clone(&a1_rc)));
    assert!(tp.block_preferred(Rc::clone(&a2_rc)));
    assert!(!tp.block_preferred(Rc::clone(&b1_rc)));
    assert!(!tp.block_preferred(Rc::clone(&b2_rc)));
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_error_on_initial_reject --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#ErrorOnInitialRejectionTest"
#[test]
fn test_topological_error_on_initial_reject() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use avalanche_types::errors::Error;
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 1,
        beta_rogue: 1,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let rejected_blk = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Rejected),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );

    let mut rejected_decidable =
        TestDecidable::new(Id::empty().prefix(&[2]).unwrap(), Status::Processing);
    rejected_decidable.set_reject_result(Err(Error::Other {
        message: "test error".to_string(),
        retryable: false,
    }));
    let blk = TestBlock::new(
        rejected_decidable,
        rejected_blk.id(),
        Ok(()),
        Bytes::new(),
        rejected_blk.height() + 1,
        0,
    );

    let res = tp.add_block(blk.clone());
    assert!(res.is_err());
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_error_on_initial_accept --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#ErrorOnAcceptTest"
#[test]
fn test_topological_error_on_initial_accept() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use avalanche_types::errors::Error;
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 1,
        beta_rogue: 1,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let mut failing_decidable =
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing);
    failing_decidable.set_accept_result(Err(Error::Other {
        message: "test error".to_string(),
        retryable: false,
    }));
    let blk = TestBlock::new(
        failing_decidable,
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );

    let added_blk = tp.add_block(blk.clone()).unwrap();
    assert_eq!(added_blk.borrow().status().unwrap(), Status::Processing);

    let votes = Bag::new();
    votes.add_count(&blk.id(), 1);
    assert!(tp.record_poll(votes).is_err());
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_error_on_reject_sibling --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#ErrorOnRejectSiblingTest"
#[test]
fn test_topological_error_on_reject_sibling() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use avalanche_types::errors::Error;
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 1,
        beta_rogue: 1,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk0 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );

    let mut failing_decidable =
        TestDecidable::new(Id::empty().prefix(&[2]).unwrap(), Status::Processing);
    failing_decidable.set_reject_result(Err(Error::Other {
        message: "test error".to_string(),
        retryable: false,
    }));
    let blk1 = TestBlock::new(
        failing_decidable,
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );

    let added_blk0 = tp.add_block(blk0.clone()).unwrap();
    assert_eq!(added_blk0.borrow().status().unwrap(), Status::Processing);

    let added_blk1 = tp.add_block(blk1.clone()).unwrap();
    assert_eq!(added_blk1.borrow().status().unwrap(), Status::Processing);

    let votes0 = Bag::new();
    votes0.add_count(&blk0.id(), 1);
    assert!(tp.record_poll(votes0).is_err());
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_error_on_transitive_reject --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#ErrorOnTransitiveRejectionTest"
#[test]
fn test_topological_error_on_transitive_reject() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use avalanche_types::errors::Error;
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 1,
        beta_rogue: 1,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk0 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[1]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );

    let blk1 = TestBlock::new(
        TestDecidable::new(Id::empty().prefix(&[2]).unwrap(), Status::Processing),
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );

    let mut failing_decidable =
        TestDecidable::new(Id::empty().prefix(&[3]).unwrap(), Status::Processing);
    failing_decidable.set_reject_result(Err(Error::Other {
        message: "test error".to_string(),
        retryable: false,
    }));
    let blk2 = TestBlock::new(
        failing_decidable,
        blk1.id(),
        Ok(()),
        Bytes::new(),
        blk1.height() + 1,
        0,
    );

    let added_blk0 = tp.add_block(blk0.clone()).unwrap();
    assert_eq!(added_blk0.borrow().status().unwrap(), Status::Processing);

    let added_blk1 = tp.add_block(blk1.clone()).unwrap();
    assert_eq!(added_blk1.borrow().status().unwrap(), Status::Processing);

    let added_blk2 = tp.add_block(blk2.clone()).unwrap();
    assert_eq!(added_blk2.borrow().status().unwrap(), Status::Processing);

    let votes0 = Bag::new();
    votes0.add_count(&blk0.id(), 1);
    assert!(tp.record_poll(votes0).is_err());
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_error_on_decided_block --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#ErrorOnAddDecidedBlock"
#[test]
fn test_topological_error_on_decided_block() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::test_decidable::TestDecidable;
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 1,
        beta_rogue: 1,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk = TestBlock::new(
        TestDecidable::new(Id::from_slice(&[0x03]), Status::Accepted), // 0b0011
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    let res = tp.add_block(blk.clone());
    assert!(res.is_err());
    match res {
        Ok(_) => panic!("unexpected Ok"),
        Err(e) => assert!(e.contains(&"duplicate block add")),
    }
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_error_on_add_duplicate_block_id --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#ErrorOnAddDuplicateBlockID"
#[test]
fn test_topological_error_on_add_duplicate_block_id() {
    use crate::snowman::block::test_block::TestBlock;
    use avalanche_types::choices::{decidable::Decidable, test_decidable::TestDecidable};
    use bytes::Bytes;

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let genesis_blk_id = Id::empty().prefix(&[0]).unwrap();
    let genesis_height = 0_u64;

    let params = crate::Parameters {
        k: 1,
        alpha: 1,
        beta_virtuous: 1,
        beta_rogue: 1,
        concurrent_repolls: 1,
        optimal_processing: 1,
        max_outstanding_items: 1,
        max_item_processing_time: 1,
        mixed_query_num_push_to_validators: 0,
        mixed_query_num_push_to_non_validators: 0,
    };
    let (tp, _) =
        Topological::<TestBlock>::new(Context::default(), params, genesis_blk_id, genesis_height)
            .expect("failed to create Topological");
    assert_eq!(tp.preference(), genesis_blk_id);

    let blk0 = TestBlock::new(
        TestDecidable::new(Id::from_slice(&[0x03]), Status::Processing), // 0b0011
        genesis_blk_id,
        Ok(()),
        Bytes::new(),
        genesis_height + 1,
        0,
    );
    let blk1 = TestBlock::new(
        TestDecidable::new(Id::from_slice(&[0x03]), Status::Processing), // 0b0011
        blk0.id(),
        Ok(()),
        Bytes::new(),
        blk0.height() + 1,
        0,
    );

    let added_blk0 = tp.add_block(blk0.clone()).unwrap();
    assert_eq!(added_blk0.borrow().status().unwrap(), Status::Processing);

    let res = tp.add_block(blk1.clone());
    assert!(res.is_err());
    match res {
        Ok(_) => panic!("unexpected Ok"),
        Err(e) => assert!(e.contains(&"duplicate block add")),
    }
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_randomized_consistency --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#RandomizedConsistencyTest"
#[test]
fn test_topological_randomized_consistency() {
    // TODO
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_metrics_processing_error --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#MetricsProcessingErrorTest"
#[test]
fn test_topological_metrics_processing_error() {
    // TODO
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_metrics_accepted_error --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#MetricsAcceptedErrorTest"
#[test]
fn test_topological_metrics_accepted_error() {
    // TODO
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::topological::test_topological_metrics_rejected_error --exact --show-output
/// ref. "avalanchego/snow/consensus/snowman#MetricsRejectedErrorTest"
#[test]
fn test_topological_metrics_rejected_error() {
    // TODO
}
