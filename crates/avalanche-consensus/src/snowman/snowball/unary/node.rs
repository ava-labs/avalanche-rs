use std::cell::Cell;

use crate::snowman::snowball::{self, binary, unary};
use avalanche_types::ids::{bag::Bag, bits, Id};

/// Represents a unary node with either no child, or a single child.
/// It handles the voting on a range of identical, virtuous, snowball instances.
/// ref. <https://github.com/ava-labs/avalanchego/blob/master/snow/consensus/snowball/tree.go> "unaryNode"
#[derive(Clone, Debug)]
pub struct Node {
    /// Parameters inherited from the tree that contains this node.
    pub parameters: crate::Parameters,

    /// Runs snowball logic.
    pub snowball: unary::Snowball,

    /// The choice that is preferred at every branch in this sub-tree.
    pub preference: Cell<Id>,

    /// The last bit index in the prefix that is assumed to be decided.
    /// In snowball binary decomposition, values are being voted on 256-bit
    /// hash values. Thus, the index ranges are [0, 255).
    pub decided_prefix: Cell<i64>,

    /// The last bit in the prefix that this node transitively references.
    /// Ranges (decided_prefix, 256).
    pub common_prefix: Cell<i64>,

    /// Used as an optimization to prevent needless tree traversals.
    /// It is the continuation of "should_reset" in the Tree struct.
    pub should_reset: Cell<bool>,

    /// The child is the possibly none, node that votes on the next bits
    /// in the decision.
    pub child: Option<Box<snowball::Node>>,
}

impl Node {
    pub fn preference(&self) -> Id {
        self.preference.get()
    }

    pub fn decided_prefix(&self) -> i64 {
        self.decided_prefix.get()
    }

    pub fn common_prefix(&self) -> i64 {
        self.common_prefix.get()
    }

    pub fn finalized(&self) -> bool {
        self.snowball.finalized()
    }

    /// This is by far the most complicated function in this algorithm.
    /// The intuition is that this instance represents a series of consecutive unary
    /// snowball instances, and this function's purpose is convert one of these unary
    /// snowball instances into a binary snowball instance.
    ///
    /// There are 5 possible cases:
    ///
    /// Case #1.
    /// If none of these instances should be split, split a child.
    /// For example, insert "000 01" to "000", with the common prefix "000".
    ///
    /// Case #2.
    /// A series of only one unary instance must be split.
    /// For example inserting "1" to "0" returns a binary choice.
    ///
    /// Case #3.
    /// If first bit differs, it must be split.
    /// For example, inserting "10" to "00" returns a binary choice.
    ///
    /// Case #4.
    /// If the last bit differs, it must be split.
    /// For example, inserting "01" to "00" returns a binary choice in its child.
    ///
    /// Case #5.
    /// If a bit differs in its interior bit, it must be split.
    /// For example, inserting "010" to "000" returns a binary choice.
    ///
    /// ref. <https://github.com/ava-labs/avalanchego/blob/master/snow/consensus/snowball/tree.go>
    pub fn add(&mut self, new_choice: &Id) -> snowball::Node {
        if self.finalized() {
            // tree is finalized, or leaf node
            return snowball::Node::Unary(self.clone());
        }

        let (index, found) = bits::first_difference_subset(
            self.decided_prefix() as usize,
            self.common_prefix() as usize,
            &self.preference(),
            new_choice,
        );
        if !found {
            // no difference, thus this node shouldn't be split (case #1)
            // e.g., insert "000 01" to "000"
            if let Some(child) = self.child.clone() {
                let added_child = match *child {
                    snowball::Node::Unary(mut unary_node) => unary_node.add(new_choice),
                    snowball::Node::Binary(mut binary_node) => binary_node.add(new_choice),
                };
                self.child = Some(Box::new(added_child));
            }

            // if child is none, then we are attempting to add the same choice
            // into the tree, which should be a no-op
        } else {
            // difference is found, thus split

            // currently preferred bit
            let bit = self.preference().bit(index);
            let mut b = binary::node::Node {
                parameters: self.parameters.clone(),

                snowball: self
                    .snowball
                    .extend(self.parameters.beta_rogue.into(), bit.as_usize() as i64),

                preferences: Cell::new([Id::empty(), Id::empty()]),
                bit: Cell::new(index as i64),
                should_reset: Cell::new([self.should_reset.get(), self.should_reset.get()]),

                child0: None,
                child1: None,
            };
            let mut updated_preferences = b.preferences.take();
            updated_preferences[bit.as_usize()] = self.preference();
            updated_preferences[1 - bit.as_usize()] = *new_choice;
            b.preferences.set(updated_preferences);

            let new_child = Self {
                parameters: self.parameters.clone(),
                snowball: unary::Snowball::new(self.parameters.beta_virtuous.into()),
                preference: Cell::new(*new_choice),

                // new child assumes this branch has decided in it's favor
                decided_prefix: Cell::new(index as i64 + 1),
                // new child has no conflicts under this branch
                common_prefix: Cell::new(bits::NUM_BITS as i64),

                should_reset: Cell::new(false),
                child: None,
            };

            // this node was only voting over one bit (case #2)
            // e.g., inserting "1" to "0" returns a binary choice
            if self.decided_prefix() == self.common_prefix() - 1 {
                match bit {
                    bits::Bit::Zero => {
                        b.child0 = self.child.clone();
                        if self.child.is_some() {
                            b.child1 = Some(Box::new(snowball::Node::Unary(new_child)));
                        }
                    }
                    bits::Bit::One => {
                        b.child1 = self.child.clone();
                        if self.child.is_some() {
                            b.child0 = Some(Box::new(snowball::Node::Unary(new_child)));
                        }
                    }
                }
                return snowball::Node::Binary(b);
            }

            // this node was split on the first bit (case #3)
            // e.g., inserting "10" to "00" returns a binary choice
            if index as i64 == self.decided_prefix() {
                let decided_prefix = self.decided_prefix.take();
                self.decided_prefix.set(decided_prefix + 1);

                match bit {
                    bits::Bit::Zero => {
                        b.child0 = Some(Box::new(snowball::Node::Unary(self.clone())));
                        b.child1 = Some(Box::new(snowball::Node::Unary(new_child)));
                    }
                    bits::Bit::One => {
                        b.child0 = Some(Box::new(snowball::Node::Unary(new_child)));
                        b.child1 = Some(Box::new(snowball::Node::Unary(self.clone())));
                    }
                }

                return snowball::Node::Binary(b);
            }

            // this node was split on the last bit (case #4)
            // e.g., inserting "01" to "00" returns a binary choice in its child
            if index as i64 == self.common_prefix() - 1 {
                let common_prefix = self.common_prefix.take();
                self.common_prefix.set(common_prefix - 1);

                match bit {
                    bits::Bit::Zero => {
                        b.child0 = self.child.clone();
                        if self.child.is_some() {
                            b.child1 = Some(Box::new(snowball::Node::Unary(new_child)));
                        }
                    }
                    bits::Bit::One => {
                        b.child1 = self.child.clone();
                        if self.child.is_some() {
                            b.child0 = Some(Box::new(snowball::Node::Unary(new_child)));
                        }
                    }
                }

                self.child = Some(Box::new(snowball::Node::Binary(b)));
                return snowball::Node::Unary(self.clone());
            }

            // this node was split on an interior bit (case #5)
            // e.g., inserting "010" to "000" returns a binary choice
            let original_decided_prefix = self.decided_prefix.take();
            self.decided_prefix.set(index as i64 + 1);

            match bit {
                bits::Bit::Zero => {
                    b.child0 = Some(Box::new(snowball::Node::Unary(self.clone())));
                    b.child1 = Some(Box::new(snowball::Node::Unary(new_child)));
                }
                bits::Bit::One => {
                    b.child0 = Some(Box::new(snowball::Node::Unary(new_child)));
                    b.child1 = Some(Box::new(snowball::Node::Unary(self.clone())));
                }
            }

            return snowball::Node::Unary(Self {
                parameters: self.parameters.clone(),
                snowball: self.snowball.clone(),
                preference: Cell::new(self.preference()),
                decided_prefix: Cell::new(original_decided_prefix),
                common_prefix: Cell::new(index as i64),
                should_reset: Cell::new(false),
                child: Some(Box::new(snowball::Node::Binary(b))),
            });
        }

        // do nothing, the choice was already rejected
        snowball::Node::Unary(self.clone())
    }

    /// Returns the new node and whether the vote was successful.
    /// ref. "avalanchego/snow/consensus/tree.go" "unaryNode.RecordPoll"
    pub fn record_poll(&mut self, votes: Bag, reset: bool) -> (snowball::Node, bool) {
        // we are guaranteed that the votes are of IDs that have previously been added
        // this ensures that the provided votes all have the same bits in the
        // range [u.decidedPrefix, u.commonPrefix) as in u.preference

        // If my parent didn't get enough votes previously, then neither did I
        if reset {
            self.snowball.record_unsuccessful_poll();
            self.should_reset.set(true); // Make sure my child is also reset correctly
        }

        if votes.len() < self.parameters.alpha as u32 {
            // didn't get enough votes
            // I must reset and my child must reset as well
            self.snowball.record_unsuccessful_poll();
            self.should_reset.set(true);

            return (snowball::Node::Unary(self.clone()), false);
        }

        // got enough votes this time
        self.snowball.record_successful_poll();

        if self.child.is_some() {
            // we are guaranteed that u.commonPrefix will equal
            // u.child.DecidedPrefix(). Otherwise, there must have been a
            // decision under this node, which isn't possible because
            // beta1 <= beta2. That means that filtering the votes between
            // u.commonPrefix and u.child.DecidedPrefix() would always result in
            // the same set being returned.

            let (new_child, _) = {
                if let Some(child) = self.child.clone() {
                    match *child {
                        snowball::Node::Unary(mut unary_node) => {
                            unary_node.record_poll(votes, self.should_reset.get())
                        }
                        snowball::Node::Binary(mut binary_node) => {
                            binary_node.record_poll(votes, self.should_reset.get())
                        }
                    }
                } else {
                    panic!("unexpected None child");
                }
            };

            if self.finalized() {
                // If I'm now decided, return my child
                return (new_child, true);
            }

            // The child's preference may have changed
            self.preference.set(new_child.preference());

            self.child = Some(Box::new(new_child));
        }

        // Now that I have passed my votes to my child,
        // I don't need to reset them
        self.should_reset.set(false);
        (snowball::Node::Unary(self.clone()), true)
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} Bits = [{}, {})",
            self.snowball,
            self.decided_prefix(),
            self.common_prefix()
        )
    }
}
