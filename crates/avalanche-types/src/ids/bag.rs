//! The bag abstraction used to group votes in voting.
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::ids::{bits, Id};

/// Represents a bag of multiple Ids for binary voting.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#Bag>
pub struct Bag {
    counts: Rc<RefCell<HashMap<Id, u32>>>,
    size: Cell<u32>,

    mode: Cell<Id>,
    mode_freq: Cell<u32>,

    threshold: Cell<u32>,
    met_threshold: Rc<RefCell<HashSet<Id>>>,
}

impl Default for Bag {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Bag {
    fn clone(&self) -> Self {
        self.deep_copy()
    }
}

impl Bag {
    pub fn new() -> Self {
        Self {
            counts: Rc::new(RefCell::new(HashMap::new())),
            size: Cell::new(0),

            mode: Cell::new(Id::empty()),
            mode_freq: Cell::new(0_u32),

            threshold: Cell::new(0_u32),
            met_threshold: Rc::new(RefCell::new(HashSet::new())),
        }
    }

    pub fn deep_copy(&self) -> Self {
        Self {
            counts: Rc::new(RefCell::new(self.counts())),
            size: Cell::new(self.len()),

            mode: Cell::new(self.mode()),
            mode_freq: Cell::new(self.mode_frequency()),

            threshold: Cell::new(self.threshold()),
            met_threshold: Rc::new(RefCell::new(self.met_threshold())),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size.get() == 0
    }

    pub fn len(&self) -> u32 {
        self.size.get()
    }

    pub fn mode(&self) -> Id {
        self.mode.get()
    }

    pub fn mode_frequency(&self) -> u32 {
        self.mode_freq.get()
    }

    pub fn threshold(&self) -> u32 {
        self.threshold.get()
    }

    /// Returns the Ids that have been seen at least threshold times.
    pub fn met_threshold(&self) -> HashSet<Id> {
        self.met_threshold.borrow().clone()
    }

    pub fn list(&self) -> Vec<Id> {
        let mut ids = Vec::with_capacity(self.counts.borrow().len());
        ids.extend(self.counts.borrow().keys().copied());
        ids
    }

    pub fn counts(&self) -> HashMap<Id, u32> {
        self.counts.borrow().clone()
    }

    pub fn set_threshold(&self, threshold: u32) {
        if self.threshold.get().eq(&threshold) {
            return;
        }

        self.threshold.set(threshold);
        self.met_threshold.borrow_mut().clear();

        for (vote, count) in self.counts.borrow().iter() {
            if *count >= threshold {
                self.met_threshold.borrow_mut().insert(*vote);
            }
        }
    }

    pub fn add_count(&self, id: &Id, count: u32) {
        if count == 0 {
            return;
        }

        let mut borrowed_mut_counts = self.counts.borrow_mut();
        let current_count = borrowed_mut_counts.get(id).unwrap_or(&0);
        let total_count = *current_count + count;

        borrowed_mut_counts.insert(*id, total_count);

        self.size.set(self.size.get() + count);

        if total_count > self.mode_freq.get() {
            self.mode.set(*id);
            self.mode_freq.set(total_count);
        }
        if total_count >= self.threshold.get() {
            self.met_threshold.borrow_mut().insert(*id);
        }
    }

    pub fn count(&self, id: &Id) -> u32 {
        let borrowed_counts = self.counts.borrow();
        let current_count = borrowed_counts.get(id).unwrap_or(&0);
        *current_count
    }

    pub fn equals(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        {
            for (vote, count) in self.counts.borrow().iter() {
                let cnt = *count;

                let borrowed_other_counts = other.counts.borrow();
                let found = borrowed_other_counts.get(vote);
                if found.is_none() {
                    return false;
                }
                let other_count = found.unwrap_or(&0);
                let other_cnt = *other_count;
                if cnt != other_cnt {
                    return false;
                }
            }
            true
        }
    }

    /// While retaining the same count values, only selects the IDs
    /// that have the same bits in the range of [start, end).
    pub fn filter(&self, start: usize, end: usize, id: &Id) -> Self {
        let new_bag = Self::new();
        for (vote, count) in self.counts.borrow().iter() {
            let count = *count;

            if bits::equal_subset(start, end, id, vote) {
                new_bag.add_count(vote, count);
            }
        }
        new_bag
    }

    /// Retaining the same count values, only selects the IDs that
    /// in the 0th index have a 0 at bit \[index\],
    /// and all ids in the 1st index have a 1 at bit \[index\].
    pub fn split(&self, index: usize) -> [Self; 2] {
        let split_votes = [Self::new(), Self::new()];

        for (vote, count) in self.counts.borrow().iter() {
            let count = *count;

            let bit = vote.bit(index);
            split_votes[bit.as_usize()].add_count(vote, count);
        }

        split_votes
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bag::test_bag_add --exact --show-output
/// ref. "TestBagAdd"
#[test]
fn test_bag_add() {
    let id0 = Id::empty();
    let id1 = Id::from_slice(&[1_u8]);

    let bag = Bag::new();

    assert_eq!(bag.count(&id0), 0);
    assert_eq!(bag.count(&id1), 0);
    assert_eq!(bag.len(), 0);
    assert_eq!(bag.list().len(), 0);
    assert_eq!(bag.mode(), Id::empty());
    assert_eq!(bag.mode_frequency(), 0);
    assert_eq!(bag.threshold(), 0);
    assert_eq!(bag.met_threshold().len(), 0);

    bag.add_count(&id0, 1);
    assert_eq!(bag.count(&id0), 1);
    assert_eq!(bag.count(&id1), 0);
    assert_eq!(bag.len(), 1);
    assert_eq!(bag.list().len(), 1);
    assert_eq!(bag.mode(), id0);
    assert_eq!(bag.mode_frequency(), 1);
    assert_eq!(bag.threshold(), 0);
    assert_eq!(bag.met_threshold().len(), 1);

    bag.add_count(&id0, 1);
    assert_eq!(bag.count(&id0), 2);
    assert_eq!(bag.count(&id1), 0);
    assert_eq!(bag.len(), 2);
    assert_eq!(bag.list().len(), 1);
    assert_eq!(bag.mode(), id0);
    assert_eq!(bag.mode_frequency(), 2);
    assert_eq!(bag.threshold(), 0);
    assert_eq!(bag.met_threshold().len(), 1);

    bag.add_count(&id1, 3);
    assert_eq!(bag.count(&id0), 2);
    assert_eq!(bag.count(&id1), 3);
    assert_eq!(bag.len(), 5);
    assert_eq!(bag.list().len(), 2);
    assert_eq!(bag.mode(), id1);
    assert_eq!(bag.mode_frequency(), 3);
    assert_eq!(bag.threshold(), 0);
    assert_eq!(bag.met_threshold().len(), 2);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bag::test_bag_set_threshold --exact --show-output
/// ref. "TestBagSetThreshold"
#[test]
fn test_bag_set_threshold() {
    let id0 = Id::empty();
    let id1 = Id::from_slice(&[1_u8]);

    let bag = Bag::new();
    bag.add_count(&id0, 2);
    bag.add_count(&id1, 3);

    bag.set_threshold(0);
    assert_eq!(bag.count(&id0), 2);
    assert_eq!(bag.count(&id1), 3);
    assert_eq!(bag.len(), 5);
    assert_eq!(bag.list().len(), 2);
    assert_eq!(bag.mode(), id1);
    assert_eq!(bag.mode_frequency(), 3);
    assert_eq!(bag.threshold(), 0);
    assert_eq!(bag.met_threshold().len(), 2);

    bag.set_threshold(3);
    assert_eq!(bag.count(&id0), 2);
    assert_eq!(bag.count(&id1), 3);
    assert_eq!(bag.len(), 5);
    assert_eq!(bag.list().len(), 2);
    assert_eq!(bag.mode(), id1);
    assert_eq!(bag.mode_frequency(), 3);
    assert_eq!(bag.threshold(), 3);
    assert_eq!(bag.met_threshold().len(), 1);
    assert!(bag.met_threshold().contains(&id1));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bag::test_bag_filter --exact --show-output
/// ref. "TestBagFilter"
#[test]
fn test_bag_filter() {
    let id0 = Id::empty();
    let id1 = Id::from_slice(&[1_u8]);
    let id2 = Id::from_slice(&[2_u8]);

    let bag = Bag::new();

    bag.add_count(&id0, 1);
    bag.add_count(&id1, 3);
    bag.add_count(&id2, 5);

    let even = bag.filter(0, 1, &id0);
    assert_eq!(even.count(&id0), 1);
    assert_eq!(even.count(&id1), 0);
    assert_eq!(even.count(&id2), 5);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bag::test_bag_split --exact --show-output
/// ref. "TestBagSplit"
#[test]
fn test_bag_split() {
    let id0 = Id::empty();
    let id1 = Id::from_slice(&[1_u8]);
    let id2 = Id::from_slice(&[2_u8]);

    let bag = Bag::new();

    bag.add_count(&id0, 1);
    bag.add_count(&id1, 3);
    bag.add_count(&id2, 5);

    let bags = bag.split(0);
    let evens = &bags[0];
    let odds = &bags[1];

    assert_eq!(evens.count(&id0), 1);
    assert_eq!(evens.count(&id1), 0);
    assert_eq!(evens.count(&id2), 5);
    assert_eq!(odds.count(&id0), 0);
    assert_eq!(odds.count(&id1), 3);
    assert_eq!(odds.count(&id2), 0);
}

const MIN_UNIQUE_BAG_SIZE: usize = 16;

/// Maps from an Id to the BitSet.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#UniqueBag>
pub struct Unique(Rc<RefCell<HashMap<Id, Rc<RefCell<bits::Set64>>>>>);

impl Unique {
    pub fn new() -> Self {
        let b: HashMap<Id, Rc<RefCell<bits::Set64>>> = HashMap::with_capacity(MIN_UNIQUE_BAG_SIZE);
        Self(Rc::new(RefCell::new(b)))
    }

    pub fn union_set(&self, id: Id, set: bits::Set64) {
        if let Some(v) = self.0.borrow().get(&id) {
            v.borrow_mut().union(set);
            return;
        }

        self.0.borrow_mut().insert(id, Rc::new(RefCell::new(set)));
    }

    pub fn difference_set(&self, id: Id, set: bits::Set64) {
        if let Some(v) = self.0.borrow().get(&id) {
            v.borrow_mut().difference(set)
        }
    }

    pub fn add(&self, set_id: u64, ids: Vec<Id>) {
        let mut bs = bits::Set64::new();
        bs.add(set_id);

        for id in ids.iter() {
            self.union_set(*id, bs);
        }
    }

    pub fn difference(&self, diff: &Unique) {
        for (id, v) in self.0.borrow().iter() {
            if let Some(vv) = diff.0.borrow().get(id) {
                v.borrow_mut().difference(vv.borrow().clone());
            }
        }
    }

    pub fn get_set(&self, id: &Id) -> bits::Set64 {
        if let Some(v) = self.0.borrow().get(id) {
            v.borrow().clone()
        } else {
            bits::Set64::new()
        }
    }

    pub fn remove_set(&self, id: &Id) {
        self.0.borrow_mut().remove(id);
    }

    pub fn list(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = Vec::new();
        for (id, _) in self.0.borrow().iter() {
            ids.push(id.clone())
        }
        ids
    }

    pub fn bag(&self, alpha: u32) -> Bag {
        let bag = Bag::new();
        bag.set_threshold(alpha);

        for (id, bs) in self.0.borrow().iter() {
            bag.add_count(id, bs.borrow().len());
        }
        bag
    }

    pub fn clear(&self) {
        self.0.borrow_mut().clear()
    }
}

impl Default for Unique {
    fn default() -> Self {
        Self::new()
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for Unique {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UniqueBag: (Size = {})", self.0.borrow().len())?;
        for (id, set) in self.0.borrow().iter() {
            write!(f, "\n    ID[{}]: Members = {}", id, set.borrow())?;
        }
        Ok(())
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bag::test_unique_bag --exact --show-output
/// ref. "TestUniqueBag"
#[test]
fn test_unique_bag() {
    let ub1 = Unique::new();
    assert_eq!(ub1.list().len(), 0);

    let id1 = Id::empty().prefix(&[1_u64]).unwrap();
    let id2 = Id::empty().prefix(&[2_u64]).unwrap();

    let ub2 = Unique::new();
    ub2.add(1, vec![id1, id2]);

    assert!(ub2.get_set(&id1).contains(1));
    assert!(ub2.get_set(&id2).contains(1));

    let mut bs1 = bits::Set64::new();
    bs1.add(2);
    bs1.add(4);

    let ub3 = Unique::new();
    ub3.union_set(id1, bs1);

    bs1.clear();
    let mut bs1 = ub3.get_set(&id1);

    assert_eq!(bs1.len(), 2);
    assert!(bs1.contains(2));
    assert!(bs1.contains(4));

    bs1.clear();

    let ub4 = Unique::new();
    ub4.add(1, vec![id1]);
    ub4.add(2, vec![id1]);
    ub4.add(5, vec![id2]);
    ub4.add(8, vec![id2]);

    let ub5 = Unique::new();
    ub5.add(5, vec![id2]);
    ub5.add(5, vec![id1]);

    ub4.difference(&ub5);
    assert_eq!(ub5.list().len(), 2);

    let ub4_id1 = ub4.get_set(&id1);
    assert_eq!(ub4_id1.len(), 2);
    assert!(ub4_id1.contains(1));
    assert!(ub4_id1.contains(2));

    let ub4_id2 = ub4.get_set(&id2);
    assert_eq!(ub4_id2.len(), 1);
    assert!(ub4_id2.contains(8));

    let ub6 = Unique::new();
    ub6.add(1, vec![id1]);
    ub6.add(2, vec![id1]);
    ub6.add(7, vec![id1]);

    let mut diff_bs = bits::Set64::new();
    diff_bs.add(1);
    diff_bs.add(7);

    ub6.difference_set(id1, diff_bs);

    let ub6_id1 = ub6.get_set(&id1);
    assert_eq!(ub6_id1.len(), 1);
    assert!(ub6_id1.contains(2));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bag::test_unique_bag_clear --exact --show-output
/// ref. "TestUniqueBagClear"
#[test]
fn test_unique_bag_clear() {
    let id1 = Id::empty().prefix(&[1_u64]).unwrap();
    let id2 = Id::empty().prefix(&[2_u64]).unwrap();

    let b = Unique::new();
    b.add(0, vec![id1]);
    b.add(1, vec![id1, id2]);

    b.clear();
    assert_eq!(b.list().len(), 0);

    let bs = b.get_set(&id1);
    assert_eq!(bs.len(), 0);

    let bs = b.get_set(&id2);
    assert_eq!(bs.len(), 0);
}
