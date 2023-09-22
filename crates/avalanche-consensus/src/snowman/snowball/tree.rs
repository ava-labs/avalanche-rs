//! A merkle-patricia trie used for Avalanche consensus.
use std::cell::Cell;

use crate::snowman::snowball::{self, unary};
use avalanche_types::ids::{bag::Bag, bits, Id};

/// Represents a snowball instance that processes the query results
/// according to the snow protocol, using a modified PATRICIA trie,
///
/// The radix in mathematics represents the maximum number of children
/// per node. For example, a regular, prefix trie is an un-compacted
/// 26-radix tree when using alphabets a-z: PATRICIA trie is a binary
/// radix trie with radix 2. PATRICIA trie traverses the tree according
/// to the bits of the search key.
///
/// "Tree" is to "snowball/snowman", "Directed" is to "snowstorm/avalanche".
/// "Tree" implements "snowball.Consensus".
/// "Directed" implements "snowstorm.Consensus".
/// "snowman.Topological" implements "snowman.Consensus".
/// "avalanche.Topological" implements "avalanche.Consensus".
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowball#Tree>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowball#Consensus>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowstorm#Directed>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Topological>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#Consensus>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/avalanche#Topological>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/avalanche#Consensus>
///
/// ref. <https://en.wikipedia.org/wiki/Radix>
/// ref. <https://en.wikipedia.org/wiki/Radix_tree>
/// ref. <https://dl.acm.org/doi/10.1145/321479.321481>
/// ref. <https://www.avalabs.org/whitepapers>
#[derive(Clone, Debug)]
pub struct Tree {
    /// Contains all configurations of this snowball instance.
    /// The parameters must be "verified" before instantiation.
    pub parameters: crate::Parameters,

    /// Root node that represents the first snowball instance in the tree.
    /// And it contains references to all other snowball instances in the tree.
    pub node: Box<snowball::Node>,

    /// Used as an optimization to prevent needless tree traversals.
    /// If a snowball instance does not get an alpha majority,
    /// that instance needs to reset by calling "record_unsuccessful_poll".
    /// Because the tree splits votes based on the branch, when an instance
    /// doesn't get an alpha majority, none of the children of this instance
    /// can get an alpha majority. To avoid calling "record_unsuccessful_poll"
    /// on the full sub-tree of a node that didn't get an alpha majority,
    /// "should_reset" is used to indicate that any subsequent traversal into
    /// this sub-tree should call "record_unsuccessful_poll" before performing
    /// any other action.
    pub should_reset: Cell<bool>,
}

impl Tree {
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowball#Tree.Initialize>
    pub fn new(parameters: crate::Parameters, choice: Id) -> Self {
        let beta_virtuous = parameters.beta_virtuous as i64;
        let unary_snowball = unary::Snowball::new(beta_virtuous);
        let u = unary::node::Node {
            parameters: parameters.clone(),
            snowball: unary_snowball,
            preference: Cell::new(choice),
            decided_prefix: Cell::new(0),
            common_prefix: Cell::new(bits::NUM_BITS as i64),
            should_reset: Cell::new(false),
            child: None,
        };
        Self {
            parameters,
            node: Box::new(snowball::Node::Unary(u)),
            should_reset: Cell::new(false),
        }
    }

    /// Returns the copied parameters that describe this snowman instance.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowball#Tree.Parameters>
    pub fn parameters(&self) -> crate::Parameters {
        self.parameters.clone()
    }

    /// Returns the current preferred choice to be finalized.
    pub fn preference(&self) -> Id {
        self.node.preference()
    }

    /// Returns the number of assumed decided bits of this node.
    pub fn decided_prefix(&self) -> i64 {
        self.node.decided_prefix()
    }

    /// Returns "true" when the choice has been finalized.
    pub fn finalized(&self) -> bool {
        self.node.finalized()
    }

    /// Adds a new choice to vote on and returns the new node.
    /// NOTE: Most bits do not have conflicts, and having a conflict in the
    /// last bit is equally as improbable as hash collision.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowball#Tree.Add>
    pub fn add(&mut self, new_choice: &Id) {
        if !bits::equal_subset(
            0,
            self.decided_prefix() as usize,
            &self.preference(),
            new_choice,
        ) {
            // already decided against this new ID
            return;
        }

        // equal subsets mean conflict has been found!
        let added_node = self.node.add(new_choice);
        self.node = Box::new(added_node);
    }

    /// Records the results of a network poll. Assumes all choices
    /// have been previously added.
    ///
    /// If the consensus instance was not previously finalized,
    /// this function returns "true" if the poll was successful
    /// and "false" if the poll was unsuccessful.
    ///
    /// If the consensus instance was previously finalized,
    /// this function may return "true" or "false".
    /// ref. <https://github.com/ava-labs/avalanchego/commit/533fe9146557a4c66cf1bdc1f5a2949b5e9b5ec7>
    ///
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowball#Consensus>
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowball#Tree.RecordPoll>
    pub fn record_poll(&mut self, votes: &Bag) -> bool {
        // get the assumed decided prefix of the root node
        let decided_prefix = self.decided_prefix();

        // if any of the bits differ from the preference in this prefix,
        // the vote is for a rejected operation, thus filter out these invalid votes
        let filtered_votes = votes.filter(0, decided_prefix as usize, &self.preference());

        // now that the votes have been restricted to the valid votes,
        // pass them into the first snowball instance
        let (polled_node, successful) = self
            .node
            .record_poll(filtered_votes, self.should_reset.get());
        self.node = Box::new(polled_node);

        // as we just passed the reset into the snowball instance,
        // we should no longer reset
        self.should_reset.set(false);

        return successful;
    }

    /// Resets the snowflake counters of this consensus instance.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowball#Tree.RecordUnsuccessfulPoll>
    pub fn record_unsuccessful_poll(&self) {
        self.should_reset.set(true);
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut prefixes: Vec<String> = vec![String::new()];
        let mut nodes: Vec<snowball::Node> = vec![*self.node.clone()];
        while let Some(prefix) = prefixes.pop() {
            write!(f, "{}", prefix)?;
            let new_prefix = format!("{}    ", prefix);
            match nodes.pop().unwrap() {
                snowball::Node::Unary(n) => {
                    writeln!(f, "{}", n)?;

                    if n.child.is_some() {
                        prefixes.push(new_prefix);
                        nodes.push(*n.child.as_ref().unwrap().clone());
                    }
                }
                snowball::Node::Binary(n) => {
                    writeln!(f, "{}", n)?;

                    if n.child0.is_some() {
                        prefixes.push(new_prefix.clone());
                        prefixes.push(new_prefix);

                        nodes.push(*n.child1.as_ref().unwrap().clone());
                        nodes.push(*n.child0.as_ref().unwrap().clone());
                    }
                }
            }
        }
        Ok(())
    }
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_singletone --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballSingleton"
#[test]
fn test_tree_snowball_singletone() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let red = Id::empty().prefix(&[0]).unwrap();
    let blue = Id::empty().prefix(&[1]).unwrap();

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 2,
            beta_rogue: 5,
            ..Default::default()
        },
        red.clone(),
    );
    assert!(!(tree.finalized()));

    let one_red = Bag::new();
    one_red.add_count(&red, 1);
    assert!(tree.record_poll(&one_red));
    assert!(!(tree.finalized()));

    let empty = Bag::new();
    assert!(!(tree.record_poll(&empty)));
    assert!(!(tree.finalized()));

    assert!(tree.record_poll(&one_red));
    assert!(!(tree.finalized()));

    assert!(tree.record_poll(&one_red));
    assert_eq!(tree.preference(), red);
    assert!(tree.finalized());

    tree.add(&blue);

    let one_blue = Bag::new();
    one_blue.add_count(&blue, 1);

    // because the tree is already finalized,
    // record_poll returns either true or false
    tree.record_poll(&one_blue);
    assert_eq!(tree.preference(), red);
    assert!(tree.finalized());
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_record_unsuccessful_poll --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballRecordUnsuccessfulPoll"
#[test]
fn test_tree_snowball_record_unsuccessful_poll() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let red = Id::empty().prefix(&[0]).unwrap();

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 3,
            beta_rogue: 5,
            ..Default::default()
        },
        red.clone(),
    );
    assert!(!(tree.finalized()));

    let one_red = Bag::new();
    one_red.add_count(&red, 1);
    assert!(tree.record_poll(&one_red));
    assert!(!(tree.finalized()));

    tree.record_unsuccessful_poll();

    assert!(tree.record_poll(&one_red));
    assert!(!(tree.finalized()));

    assert!(tree.record_poll(&one_red));
    assert!(!(tree.finalized()));

    assert!(tree.record_poll(&one_red));
    assert_eq!(tree.preference(), red);
    assert!(tree.finalized());
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_binary --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballBinary"
#[test]
fn test_tree_snowball_binary() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let red = Id::empty().prefix(&[0]).unwrap();
    let blue = Id::empty().prefix(&[1]).unwrap();

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 1,
            beta_rogue: 2,
            ..Default::default()
        },
        red.clone(),
    );
    assert!(!(tree.finalized()));

    tree.add(&blue);
    assert_eq!(tree.preference(), red);
    assert!(!(tree.finalized()));

    let one_blue = Bag::new();
    one_blue.add_count(&blue, 1);
    assert!(tree.record_poll(&one_blue));
    assert_eq!(tree.preference(), blue);
    assert!(!(tree.finalized()));

    let one_red = Bag::new();
    one_red.add_count(&red, 1);
    assert!(tree.record_poll(&one_red));
    assert_eq!(tree.preference(), blue);
    assert!(!(tree.finalized()));

    assert!(tree.record_poll(&one_blue));
    assert_eq!(tree.preference(), blue);
    assert!(!(tree.finalized()));

    assert!(tree.record_poll(&one_blue));
    assert_eq!(tree.preference(), blue);
    assert!(tree.finalized());
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_last_binary --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballLastBinary"
#[test]
fn test_tree_snowball_last_binary() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let zero = Id::empty();
    let one = Id::from_slice(&[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x80,
    ]);

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 2,
            beta_rogue: 2,
            ..Default::default()
        },
        zero.clone(),
    );
    assert!(!(tree.finalized()));

    tree.add(&one);
    tree.add(&one); // should do nothing
    assert_eq!(tree.preference(), zero);
    assert!(!(tree.finalized()));

    log::info!("{}", tree);
    assert_eq!(tree.to_string(), "SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [0, 255)
    SB(Preference = 0, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 255
");

    let one_bag = Bag::new();
    one_bag.add_count(&one, 1);

    assert!(tree.record_poll(&one_bag));
    assert_eq!(tree.preference(), one);
    assert!(!(tree.finalized()));

    log::info!("{}", tree);
    assert_eq!(tree.to_string(), "SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = false)) Bits = [0, 255)
    SB(Preference = 1, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 1, SF(Confidence = 1, Finalized = false, SL(Preference = 1))) Bit = 255
");

    assert!(tree.record_poll(&one_bag));
    assert_eq!(tree.preference(), one);
    assert!(tree.finalized());

    log::info!("{}", tree);
    assert_eq!(tree.to_string(), "SB(Preference = 1, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 2, SF(Confidence = 2, Finalized = true, SL(Preference = 1))) Bit = 255
");
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_add_previously_rejected --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballAddPreviouslyRejected"
#[test]
fn test_tree_snowball_add_previously_rejected() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let zero = Id::from_slice(&[0b00000000]);
    let one = Id::from_slice(&[0b00000001]);
    let two = Id::from_slice(&[0b00000010]);
    let four = Id::from_slice(&[0b00000100]);

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 1,
            beta_rogue: 2,
            ..Default::default()
        },
        zero.clone(),
    );
    assert!(!(tree.finalized()));

    tree.add(&one);
    tree.add(&four);

    assert_eq!(tree.preference(), zero);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(tree.to_string(), "SB(Preference = 0, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 2)
        SB(Preference = 0, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 2
            SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
            SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
");

    let zero_bag = Bag::new();
    zero_bag.add_count(&zero, 1);
    assert!(tree.record_poll(&zero_bag));

    assert_eq!(tree.preference(), zero);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(tree.to_string(), "SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 2
        SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [3, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
");

    tree.add(&two);
    assert_eq!(tree.preference(), zero);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(tree.to_string(), "SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 2
        SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [3, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
");
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_new_unary --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballNewUnary"
#[test]
fn test_tree_snowball_new_unary() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let zero = Id::from_slice(&[0b00000000]);
    let one = Id::from_slice(&[0b00000001]);

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 2,
            beta_rogue: 3,
            ..Default::default()
        },
        zero.clone(),
    );
    assert!(!(tree.finalized()));

    tree.add(&one);
    assert_eq!(tree.preference(), zero);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(tree.to_string(), "SB(Preference = 0, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
");

    let one_bag = Bag::new();
    one_bag.add_count(&one, 1);
    assert!(tree.record_poll(&one_bag));

    assert_eq!(tree.preference(), one);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(tree.to_string(), "SB(Preference = 1, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 1, SF(Confidence = 1, Finalized = false, SL(Preference = 1))) Bit = 0
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
    SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = false)) Bits = [1, 256)
");

    assert!(tree.record_poll(&one_bag));

    assert_eq!(tree.preference(), one);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(tree.to_string(), "SB(Preference = 1, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 2, SF(Confidence = 2, Finalized = false, SL(Preference = 1))) Bit = 0
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
    SB(NumSuccessfulPolls = 2, SF(Confidence = 2, Finalized = true)) Bits = [1, 256)
");

    assert!(tree.record_poll(&one_bag));

    assert_eq!(tree.preference(), one);
    assert!(tree.finalized());
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(NumSuccessfulPolls = 3, SF(Confidence = 3, Finalized = true)) Bits = [1, 256)
"
    );
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_transitive_reset --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballTransitiveReset"
#[test]
fn test_tree_snowball_transitive_reset() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let zero = Id::from_slice(&[0b00000000]);
    let two = Id::from_slice(&[0b00000010]);
    let eight = Id::from_slice(&[0b00001000]);

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 2,
            beta_rogue: 2,
            ..Default::default()
        },
        zero.clone(),
    );
    assert!(!(tree.finalized()));

    tree.add(&two);
    tree.add(&eight);

    assert_eq!(tree.preference(), zero);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [0, 1)
    SB(Preference = 0, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 1
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 3)
            SB(Preference = 0, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 3
                SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [4, 256)
                SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [4, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
"
    );

    let zero_bag = Bag::new();
    zero_bag.add_count(&zero, 1);
    assert!(tree.record_poll(&zero_bag));

    assert_eq!(tree.preference(), zero);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = false)) Bits = [0, 1)
    SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 1
        SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = false)) Bits = [2, 3)
            SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 3
                SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = false)) Bits = [4, 256)
                SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [4, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
"
    );

    let empty_bag = Bag::new();
    assert!(!(tree.record_poll(&empty_bag)));

    assert_eq!(tree.preference(), zero);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(NumSuccessfulPolls = 1, SF(Confidence = 0, Finalized = false)) Bits = [0, 1)
    SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 1
        SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = false)) Bits = [2, 3)
            SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 3
                SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = false)) Bits = [4, 256)
                SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [4, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
"
    );

    assert!(tree.record_poll(&zero_bag));

    assert_eq!(tree.preference(), zero);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(NumSuccessfulPolls = 2, SF(Confidence = 1, Finalized = false)) Bits = [0, 1)
    SB(Preference = 0, NumSuccessfulPolls[0] = 2, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 1
        SB(NumSuccessfulPolls = 2, SF(Confidence = 1, Finalized = false)) Bits = [2, 3)
            SB(Preference = 0, NumSuccessfulPolls[0] = 2, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 3
                SB(NumSuccessfulPolls = 2, SF(Confidence = 1, Finalized = false)) Bits = [4, 256)
                SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [4, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
"
    );

    assert!(tree.record_poll(&zero_bag));

    assert_eq!(tree.preference(), zero);
    assert!(tree.finalized());
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(NumSuccessfulPolls = 3, SF(Confidence = 2, Finalized = true)) Bits = [4, 256)
"
    );
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_trinary --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballTrinary"
#[test]
fn test_tree_snowball_trinary() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let red = Id::empty().prefix(&[0]).unwrap();
    let blue = Id::empty().prefix(&[1]).unwrap();
    let green = Id::empty().prefix(&[2]).unwrap();

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 1,
            beta_rogue: 2,
            ..Default::default()
        },
        green.clone(),
    );
    assert!(!(tree.finalized()));

    tree.add(&red);
    tree.add(&blue);

    //       *
    //      / \
    //     R   *
    //        / \
    //       G   B

    assert_eq!(tree.preference(), green);
    assert!(!(tree.finalized()));

    let red_bag = Bag::new();
    red_bag.add_count(&red, 1);
    assert!(tree.record_poll(&red_bag));

    assert_eq!(tree.preference(), red);
    assert!(!(tree.finalized()));

    let blue_bag = Bag::new();
    blue_bag.add_count(&blue, 1);
    assert!(tree.record_poll(&blue_bag));

    assert_eq!(tree.preference(), red);
    assert!(!(tree.finalized()));

    // Here is a case where voting for a color makes a different color become
    // the preferred color. This is intended behavior.
    let green_bag = Bag::new();
    green_bag.add_count(&green, 1);
    assert!(tree.record_poll(&green_bag));

    assert_eq!(tree.preference(), blue);
    assert!(!(tree.finalized()));

    // Red has already been rejected here, so this is not a successful poll.
    assert!(!(tree.record_poll(&red_bag)));

    assert_eq!(tree.preference(), blue);
    assert!(!(tree.finalized()));

    assert!(tree.record_poll(&green_bag));

    assert_eq!(tree.preference(), green);
    assert!(!(tree.finalized()));
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_close_trinary --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballCloseTrinary"
#[test]
fn test_tree_snowball_close_trinary() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let yellow = Id::from_slice(&[0x01]);
    let cyan = Id::from_slice(&[0x02]);
    let magenta = Id::from_slice(&[0x03]);

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 1,
            beta_rogue: 2,
            ..Default::default()
        },
        yellow.clone(),
    );
    assert!(!(tree.finalized()));

    tree.add(&cyan);
    tree.add(&magenta);

    //       *
    //      / \
    //     C   *
    //        / \
    //       Y   M

    assert_eq!(tree.preference(), yellow);
    assert!(!(tree.finalized()));

    let yellow_bag = Bag::new();
    yellow_bag.add_count(&yellow, 1);
    assert!(tree.record_poll(&yellow_bag));

    assert_eq!(tree.preference(), yellow);
    assert!(!(tree.finalized()));

    let magenta_bag = Bag::new();
    magenta_bag.add_count(&magenta, 1);
    assert!(tree.record_poll(&magenta_bag));

    assert_eq!(tree.preference(), yellow);
    assert!(!(tree.finalized()));

    // Cyan has already been rejected here, so these are not successful polls.
    let cyan_bag = Bag::new();
    cyan_bag.add_count(&cyan, 1);
    assert!(!(tree.record_poll(&cyan_bag)));

    assert_eq!(tree.preference(), yellow);
    assert!(!(tree.finalized()));

    assert!(!(tree.record_poll(&cyan_bag)));

    assert_eq!(tree.preference(), yellow);
    assert!(!(tree.finalized()));
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_add_rejected --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballAddRejected"
#[test]
fn test_tree_snowball_add_rejected() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    log::info!("{:#010b}", 0x00); // 0 =  0b00000000
    log::info!("{:#010b}", 0x01); // 1 =  0b00000001
    log::info!("{:#010b}", 0x0a); // 10 = 0b00001010
    log::info!("{:#010b}", 0x04); // 4 =  0b00000100
    assert_eq!(0x00, 0b00000000);
    assert_eq!(0x01, 0b00000001);
    assert_eq!(0x0a, 0b00001010);
    assert_eq!(0x04, 0b00000100);

    let c0000 = Id::from_slice(&[0b00000000]); // 0000
    let c1000 = Id::from_slice(&[0b00000001]); // 1000
    let c0101 = Id::from_slice(&[0b00001010]); // 0101
    let c0010 = Id::from_slice(&[0b00000100]); // 0010

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 1,
            beta_rogue: 2,
            ..Default::default()
        },
        c0000.clone(),
    );
    assert!(!(tree.finalized()));

    tree.add(&c1000);
    tree.add(&c0010);

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));

    let c0010_bag = Bag::new();
    c0010_bag.add_count(&c0010, 1);
    assert!(tree.record_poll(&c0010_bag));

    assert_eq!(tree.preference(), c0010);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(Preference = 1, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 1, SF(Confidence = 1, Finalized = false, SL(Preference = 1))) Bit = 2
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
        SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [3, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
"
    );

    tree.add(&c0101);

    assert_eq!(tree.preference(), c0010);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(Preference = 1, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 1, SF(Confidence = 1, Finalized = false, SL(Preference = 1))) Bit = 2
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
        SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [3, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
"
    );
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_reset_child --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballResetChild"
#[test]
fn test_tree_snowball_reset_child() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    log::info!("{:#010b}", 0x00); // 0 = 0b00000000
    log::info!("{:#010b}", 0x01); // 1 = 0b00000001
    log::info!("{:#010b}", 0x02); // 2 = 0b00000010
    assert_eq!(0x00, 0b00000000);
    assert_eq!(0x01, 0b00000001);
    assert_eq!(0x02, 0b00000010);

    let c0000 = Id::from_slice(&[0b00000000]); // 0000
    let c1000 = Id::from_slice(&[0b00000001]); // 1000
    let c0100 = Id::from_slice(&[0b00000010]); // 0100

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 1,
            beta_rogue: 2,
            ..Default::default()
        },
        c0000.clone(),
    );
    assert!(!(tree.finalized()));

    tree.add(&c0100);
    tree.add(&c1000);

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));

    let c0000_bag = Bag::new();
    c0000_bag.add_count(&c0000, 1);
    assert!(tree.record_poll(&c0000_bag));

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 1
        SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [2, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
"
    );

    let empty_bag = Bag::new();
    assert!(!(tree.record_poll(&empty_bag)));

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 1
        SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [2, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
"
    );

    assert!(tree.record_poll(&c0000_bag));

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 2, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(Preference = 0, NumSuccessfulPolls[0] = 2, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 1
        SB(NumSuccessfulPolls = 2, SF(Confidence = 1, Finalized = true)) Bits = [2, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
"
    );
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_reset_sibling --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballResetSibling"
#[test]
fn test_tree_snowball_reset_sibling() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let c0000 = Id::from_slice(&[0b00000000]); // 0000
    let c1000 = Id::from_slice(&[0b00000001]); // 1000
    let c0100 = Id::from_slice(&[0b00000010]); // 0100

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 1,
            beta_rogue: 2,
            ..Default::default()
        },
        c0000.clone(),
    );
    assert!(!(tree.finalized()));

    tree.add(&c0100);
    tree.add(&c1000);

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));

    let c0100_bag = Bag::new();
    c0100_bag.add_count(&c0100, 1);
    assert!(tree.record_poll(&c0100_bag));

    assert_eq!(tree.preference(), c0100);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(Preference = 1, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 1, SF(Confidence = 1, Finalized = false, SL(Preference = 1))) Bit = 1
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
        SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [2, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
"
    );

    let c1000_bag = Bag::new();
    c1000_bag.add_count(&c1000, 1);
    assert!(tree.record_poll(&c1000_bag));

    assert_eq!(tree.preference(), c0100);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 1, SF(Confidence = 1, Finalized = false, SL(Preference = 1))) Bit = 0
    SB(Preference = 1, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 1, SF(Confidence = 1, Finalized = false, SL(Preference = 1))) Bit = 1
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
        SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [2, 256)
    SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [1, 256)
"
    );

    assert!(tree.record_poll(&c0100_bag));

    assert_eq!(tree.preference(), c0100);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 2, NumSuccessfulPolls[1] = 1, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(Preference = 1, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 2, SF(Confidence = 1, Finalized = false, SL(Preference = 1))) Bit = 1
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
        SB(NumSuccessfulPolls = 2, SF(Confidence = 1, Finalized = true)) Bits = [2, 256)
    SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [1, 256)
"
    );
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_5_colors --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowball5Colors"
#[test]
fn test_tree_snowball_5_colors() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let mut colors: Vec<Id> = Vec::new();
    for i in 0..5 {
        colors.push(Id::empty().prefix(&[i as u64]).unwrap());
    }

    let mut tree0 = Tree::new(
        crate::Parameters {
            k: 5,
            alpha: 5,
            beta_virtuous: 20,
            beta_rogue: 30,
            ..Default::default()
        },
        colors[4].clone(),
    );
    assert!(!(tree0.finalized()));
    tree0.add(&colors[0]);
    tree0.add(&colors[1]);
    tree0.add(&colors[2]);
    tree0.add(&colors[3]);

    let mut tree1 = Tree::new(
        crate::Parameters {
            k: 5,
            alpha: 5,
            beta_virtuous: 20,
            beta_rogue: 30,
            ..Default::default()
        },
        colors[3].clone(),
    );
    assert!(!(tree1.finalized()));
    tree1.add(&colors[0]);
    tree1.add(&colors[1]);
    tree1.add(&colors[2]);
    tree1.add(&colors[4]);

    log::info!("{}", tree0);
    log::info!("{}", tree1);

    let tree0_str = format!("{}", tree0);
    let tree0_cnt = tree0_str.matches("    ").count();
    let tree1_str = format!("{}", tree1);
    let tree1_cnt = tree1_str.matches("    ").count();

    assert_eq!(tree0_cnt, tree1_cnt);
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_fine_grained --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballFineGrained"
#[test]
fn test_tree_snowball_fine_grained() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    log::info!("{:#010b}", 0x00); // 0 = 0b00000000
    log::info!("{:#010b}", 0x01); // 1 = 0b00000001
    log::info!("{:#010b}", 0x03); // 3 = 0b00000011
    log::info!("{:#010b}", 0x04); // 4 = 0b00000100
    assert_eq!(0x00, 0b00000000);
    assert_eq!(0x01, 0b00000001);
    assert_eq!(0x03, 0b00000011);
    assert_eq!(0x04, 0b00000100);

    let c0000 = Id::from_slice(&[0b00000000]); // 0000
    let c1000 = Id::from_slice(&[0b00000001]); // 1000
    let c1100 = Id::from_slice(&[0b00000011]); // 1100
    let c0010 = Id::from_slice(&[0b00000100]); // 0010

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 1,
            beta_rogue: 2,
            ..Default::default()
        },
        c0000.clone(),
    );
    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [0, 256)
"
    );

    tree.add(&c1100);

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
"
    );

    tree.add(&c1000);

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
    SB(Preference = 1, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 1))) Bit = 1
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
"
    );

    tree.add(&c0010);

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 2)
        SB(Preference = 0, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 2
            SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
            SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
    SB(Preference = 1, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 1))) Bit = 1
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
"
    );

    let c0000_bag = Bag::new();
    c0000_bag.add_count(&c0000, 1);
    assert!(tree.record_poll(&c0000_bag));

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 2
        SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [3, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
    SB(Preference = 1, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 1))) Bit = 1
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [2, 256)
"
    );

    let c0010_bag = Bag::new();
    c0010_bag.add_count(&c0010, 1);
    assert!(tree.record_poll(&c0010_bag));

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 1, SF(Confidence = 1, Finalized = false, SL(Preference = 1))) Bit = 2
    SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [3, 256)
    SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [3, 256)
"
    );

    assert!(tree.record_poll(&c0010_bag));

    assert_eq!(tree.preference(), c0010);
    assert!(tree.finalized());
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(NumSuccessfulPolls = 2, SF(Confidence = 2, Finalized = true)) Bits = [3, 256)
"
    );
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_double_add --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballDoubleAdd"
#[test]
fn test_tree_snowball_double_add() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let red = Id::empty().prefix(&[0]).unwrap();

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 3,
            beta_rogue: 5,
            ..Default::default()
        },
        red.clone(),
    );
    assert!(!(tree.finalized()));

    tree.add(&red);

    assert_eq!(tree.preference(), red);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [0, 256)
"
    );
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_consistent --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballConsistent"
#[test]
fn test_tree_snowball_consistent() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    // TODO(gyuho): implement this with "Network" implementation
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::tree::test_tree_snowball_filter_binary_children --exact --show-output
/// ref. "avalanchego/snow/consensus/snowball#TestSnowballFilterBinaryChildren"
#[test]
fn test_tree_snowball_filter_binary_children() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let c0000 = Id::from_slice(&[0b00000000]); // 0000
    let c1000 = Id::from_slice(&[0b00000001]); // 1000
    let c0100 = Id::from_slice(&[0b00000010]); // 0100
    let c0010 = Id::from_slice(&[0b00000100]); // 0010

    let mut tree = Tree::new(
        crate::Parameters {
            k: 1,
            alpha: 1,
            beta_virtuous: 1,
            beta_rogue: 2,
            ..Default::default()
        },
        c0000.clone(),
    );
    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [0, 256)
"
    );

    tree.add(&c1000);

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
"
    );

    tree.add(&c0010);

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 2)
        SB(Preference = 0, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 2
            SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
            SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
"
    );

    let c0000_bag = Bag::new();
    c0000_bag.add_count(&c0000, 1);
    assert!(tree.record_poll(&c0000_bag));

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 2
        SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [3, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
"
    );

    tree.add(&c0100);

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 0
    SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0))) Bit = 2
        SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [3, 256)
        SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [1, 256)
"
    );

    let c0100_bag = Bag::new();
    c0100_bag.add_count(&c0100, 1);
    assert!(tree.record_poll(&c0100_bag));

    assert_eq!(tree.preference(), c0000);
    assert!(!(tree.finalized()));
    log::info!("{}", tree);
    assert_eq!(
        tree.to_string(),
        "SB(Preference = 0, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 0, SF(Confidence = 0, Finalized = false, SL(Preference = 0))) Bit = 2
    SB(NumSuccessfulPolls = 1, SF(Confidence = 1, Finalized = true)) Bits = [3, 256)
    SB(NumSuccessfulPolls = 0, SF(Confidence = 0, Finalized = false)) Bits = [3, 256)
"
    );
}
