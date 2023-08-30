use std::cell::Cell;

use crate::snowman::snowball::{self, binary};
use avalanche_types::ids::{bag::Bag, bits, Id};

/// Represents a binary node with either no child, or a single child.
/// It handles the voting on a range of identical, virtuous, snowball instances.
/// ref. "avalanchego/snow/consensus/snowball/tree.go#binaryNode"
#[derive(Clone, Debug)]
pub struct Node {
    /// Parameters inherited from the tree.
    pub parameters: crate::Parameters,

    /// Runs snowball logic.
    pub snowball: binary::Snowball,

    /// The choices preferred at every branch in their sub-tree.
    pub preferences: Cell<[Id; 2]>,

    /// The index in the Id of the choice that this node is deciding on.
    /// Will be in the range [0, 256)
    pub bit: Cell<i64>,

    /// Used as an optimization to prevent needless tree traversals.
    /// It is the continuation of shouldReset in the Tree struct.
    pub should_reset: Cell<[bool; 2]>,

    /// The child is the potentially none, node that votes on the next bits
    /// in the decision.
    pub child0: Option<Box<snowball::Node>>,
    pub child1: Option<Box<snowball::Node>>,
}

impl Node {
    pub fn preference(&self) -> Id {
        let pref = self.snowball.preference() as usize;
        let preferences = self.preferences.take();
        let preference = preferences[pref];
        self.preferences.set(preferences);
        preference
    }

    pub fn decided_prefix(&self) -> i64 {
        self.bit.get()
    }

    pub fn finalized(&self) -> bool {
        self.snowball.finalized()
    }

    pub fn add(&mut self, id: &Id) -> snowball::Node {
        let bit = id.bit(self.bit.get() as usize);
        let child = match bit {
            bits::Bit::Zero => self.child0.clone(),
            bits::Bit::One => self.child1.clone(),
        };

        // If child is nil, then we are running an instance on the last bit. Finding
        // two hashes that are equal up to the last bit would be really cool though.
        // Regardless, the case is handled
        if let Some(boxed_child) = child.clone() {
            // +1 is used because we already explicitly check the p.bit bit
            if bits::equal_subset(
                self.bit.get() as usize + 1,
                boxed_child.decided_prefix() as usize,
                &self.preferences.get()[bit.as_usize()],
                id,
            ) {
                let boxed_child = child.unwrap();
                let added_child = match *boxed_child {
                    snowball::Node::Unary(mut unary_node) => unary_node.add(id),
                    snowball::Node::Binary(mut binary_node) => binary_node.add(id),
                };
                match bit {
                    bits::Bit::Zero => self.child0 = Some(Box::new(added_child)),
                    bits::Bit::One => self.child1 = Some(Box::new(added_child)),
                }
            }
        }

        // If child is nil, then the id has already been added to the tree, so
        // nothing should be done
        // If the decided prefix isn't matched, then a previous decision has made
        // the id that is being added to have already been rejected
        snowball::Node::Binary(self.clone())
    }

    /// Returns the new node and whether the vote was successful.
    /// ref. "avalanchego/snow/consensus/tree.go" "binaryNode.RecordPoll"
    pub fn record_poll(&mut self, votes: Bag, reset: bool) -> (snowball::Node, bool) {
        // The list of votes we are passed is split into votes for bit 0
        // and votes for bit 1
        let split_votes = votes.split(self.bit.get() as usize);

        let mut bit = 0;
        // We only care about which bit is set if a successful poll can happen
        if split_votes[1].len() >= self.parameters.alpha as u32 {
            bit = 1;
        }

        let mut updated_should_reset = self.should_reset.take();
        if reset {
            self.snowball.record_unsuccessful_poll();
            updated_should_reset[bit] = true;
            // 1-bit isn't set here because it is set below anyway
        }
        // they didn't get the threshold of votes
        updated_should_reset[1 - bit] = true;
        self.should_reset.set(updated_should_reset);

        if split_votes[bit].len() < self.parameters.alpha as u32 {
            // pruned votes < alpha; didn't get enough votes
            self.snowball.record_unsuccessful_poll();

            // The winning child didn't get enough votes either
            updated_should_reset[bit] = true;
            self.should_reset.set(updated_should_reset);

            return (snowball::Node::Binary(self.clone()), false);
        }

        // this bit got alpha votes, it was a successful poll
        self.snowball.record_successful_poll(bit as i64);

        match bit {
            0 => {
                if let Some(child) = self.child0.clone() {
                    // The votes are filtered to ensure that they are votes
                    // that should count for the child
                    match *child {
                        snowball::Node::Unary(mut unary_node) => {
                            let filtered_votes = split_votes[bit].filter(
                                self.bit.get() as usize + 1,
                                unary_node.decided_prefix() as usize,
                                &self.preferences.get()[bit],
                            );

                            let (new_child, _) = unary_node
                                .record_poll(filtered_votes, self.should_reset.get()[bit]);
                            if self.snowball.finalized() {
                                // If we are decided here, that means we must have decided
                                // due to this poll. Therefore, we must have decided on bit.
                                return (new_child, true);
                            }

                            let mut updated_preferences = self.preferences.take();
                            let new_child_preference = match &new_child {
                                snowball::Node::Unary(n) => n.preference(),
                                snowball::Node::Binary(n) => n.preference(),
                            };
                            updated_preferences[bit] = new_child_preference;
                            self.preferences.set(updated_preferences);

                            self.child0 = Some(Box::new(new_child));
                        }
                        snowball::Node::Binary(mut binary_node) => {
                            let filtered_votes = split_votes[bit].filter(
                                self.bit.get() as usize + 1,
                                binary_node.decided_prefix() as usize,
                                &self.preferences.get()[bit],
                            );

                            let (new_child, _) = binary_node
                                .record_poll(filtered_votes, self.should_reset.get()[bit]);
                            if self.snowball.finalized() {
                                // If we are decided here, that means we must have decided
                                // due to this poll. Therefore, we must have decided on bit.
                                return (new_child, true);
                            }

                            let mut updated_preferences = self.preferences.take();
                            let new_child_preference = match &new_child {
                                snowball::Node::Unary(n) => n.preference(),
                                snowball::Node::Binary(n) => n.preference(),
                            };
                            updated_preferences[bit] = new_child_preference;
                            self.preferences.set(updated_preferences);

                            self.child0 = Some(Box::new(new_child));
                        }
                    };
                }
            }
            1 => {
                if let Some(child) = self.child1.clone() {
                    // The votes are filtered to ensure that they are votes
                    // that should count for the child
                    match *child {
                        snowball::Node::Unary(mut unary_node) => {
                            let filtered_votes = split_votes[bit].filter(
                                self.bit.get() as usize + 1,
                                unary_node.decided_prefix() as usize,
                                &self.preferences.get()[bit],
                            );

                            let (new_child, _) = unary_node
                                .record_poll(filtered_votes, self.should_reset.get()[bit]);
                            if self.snowball.finalized() {
                                // If we are decided here, that means we must have decided
                                // due to this poll. Therefore, we must have decided on bit.
                                return (new_child, true);
                            }

                            let mut updated_preferences = self.preferences.take();
                            let new_child_preference = match &new_child {
                                snowball::Node::Unary(n) => n.preference(),
                                snowball::Node::Binary(n) => n.preference(),
                            };
                            updated_preferences[bit] = new_child_preference;
                            self.preferences.set(updated_preferences);

                            self.child1 = Some(Box::new(new_child));
                        }
                        snowball::Node::Binary(mut binary_node) => {
                            let filtered_votes = split_votes[bit].filter(
                                self.bit.get() as usize + 1,
                                binary_node.decided_prefix() as usize,
                                &self.preferences.get()[bit],
                            );

                            let (new_child, _) = binary_node
                                .record_poll(filtered_votes, self.should_reset.get()[bit]);
                            if self.snowball.finalized() {
                                // If we are decided here, that means we must have decided
                                // due to this poll. Therefore, we must have decided on bit.
                                return (new_child, true);
                            }

                            let mut updated_preferences = self.preferences.take();
                            let new_child_preference = match &new_child {
                                snowball::Node::Unary(n) => n.preference(),
                                snowball::Node::Binary(n) => n.preference(),
                            };
                            updated_preferences[bit] = new_child_preference;
                            self.preferences.set(updated_preferences);

                            self.child1 = Some(Box::new(new_child));
                        }
                    };
                }
            }
            _ => panic!("unexpected preference bit {}", bit),
        }

        // We passed the reset down
        updated_should_reset[bit] = false;
        self.should_reset.set(updated_should_reset);

        (snowball::Node::Binary(self.clone()), true)
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Bit = {}", self.snowball, self.decided_prefix())
    }
}
