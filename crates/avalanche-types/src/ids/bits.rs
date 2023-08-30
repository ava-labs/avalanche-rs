//! Bitwise comparison operations used by avalanche consensus.
use crate::ids::Id;

pub const NUM_BITS: usize = 256;
const BITS_PER_BYTES: usize = 8;

/// Returns "true" if two Ids are equal for the range [start, stop).
/// This does bit-per-bit comparison for the Id type of [u8; ID_LEN].
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#EqualSubset>
pub fn equal_subset(start: usize, stop: usize, id1: &Id, id2: &Id) -> bool {
    if stop == 0 {
        return true;
    }

    let stop = stop - 1; // -1 for end index
    if start > stop {
        return true;
    }
    if stop >= NUM_BITS {
        return false;
    }

    let start_index = (start / BITS_PER_BYTES) as usize;
    let stop_index = (stop / BITS_PER_BYTES) as usize;

    if start_index + 1 < stop_index
        && id1.0[(start_index + 1)..stop_index] != id2.0[(start_index + 1)..stop_index]
    {
        return false;
    }

    let start_bit = (start % BITS_PER_BYTES) as usize; // index in the byte that the first bit is at
    let stop_bit = (stop % BITS_PER_BYTES) as usize; // index in the byte that the last bit is at

    let start_mask: i32 = -1 << start_bit; // 111...0... The number of 0s is equal to start_bit
    let stop_mask: i32 = (1 << (stop_bit + 1)) - 1; // 000...1... The number of 1s is equal to stop_bit + 1

    if start_index == stop_index {
        // if looking at same byte, both masks need to be applied
        let mask = start_mask & stop_mask;

        let b1 = mask & id1.0[start_index] as i32;
        let b2 = mask & id2.0[start_index] as i32;

        return b1 == b2;
    }

    let start1 = start_mask & id1.0[start_index] as i32;
    let start2 = start_mask & id2.0[start_index] as i32;

    let stop1 = stop_mask & id1.0[stop_index] as i32;
    let stop2 = stop_mask & id2.0[stop_index] as i32;

    start1 == start2 && stop1 == stop2
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bits::test_equal_subset --exact --show-output
#[test]
fn test_equal_subset() {
    // ref. TestEqualSubsetEarlyStop
    let id1 = Id::from_slice(&vec![0xf0, 0x0f]);
    let id2 = Id::from_slice(&vec![0xf0, 0x1f]);

    // println!("");
    // for c in &id1.0 {
    //     print!("{:08b} ", *c);
    // }
    // println!("");
    // for c in &id2.0 {
    //     print!("{:08b} ", *c);
    // }
    //
    // big endian - most significant byte first, 0x1 == 00000001
    // 11110000 00001111 00000000 ...
    // 11110000 00011111 00000000 ...

    assert!(equal_subset(0, 12, &id1, &id2));
    assert!(!equal_subset(0, 13, &id1, &id2));

    // ref. TestEqualSubsetLateStart
    let id1 = Id::from_slice(&vec![0x1f, 0xf8]);
    let id2 = Id::from_slice(&vec![0x10, 0x08]);

    // println!("");
    // for c in &id1.0 {
    //     print!("{:08b} ", *c);
    // }
    // println!("");
    // for c in &id2.0 {
    //     print!("{:08b} ", *c);
    // }
    //
    // big endian - most significant byte first, 0x1 == 00000001
    // 00011111 11111000 00000000 ...
    // 00010000 00001000 00000000 ...

    assert!(equal_subset(4, 12, &id1, &id2));
    assert!(!equal_subset(4, 13, &id1, &id2));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bits::test_equal_subset_same_byte --exact --show-output
/// ref. "TestEqualSubsetSameByte"
#[test]
fn test_equal_subset_same_byte() {
    let id1 = Id::from_slice(&vec![0x18]);
    let id2 = Id::from_slice(&vec![0xfc]);

    // println!("");
    // for c in &id1.0 {
    //     print!("{:08b} ", *c);
    // }
    // println!("");
    // for c in &id2.0 {
    //     print!("{:08b} ", *c);
    // }
    //
    // big endian - most significant byte first, 0x1 == 00000001
    // 00011000 00000000 ...
    // 11111100 00000000 ...

    assert!(equal_subset(3, 5, &id1, &id2));
    assert!(!equal_subset(2, 5, &id1, &id2));
    assert!(!equal_subset(3, 6, &id1, &id2));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bits::test_equal_subset_bad_middle --exact --show-output
/// ref. "TestEqualSubsetBadMiddle"
#[test]
fn test_equal_subset_bad_middle() {
    let id1 = Id::from_slice(&vec![0x18, 0xe8, 0x55]);
    let id2 = Id::from_slice(&vec![0x18, 0x8e, 0x55]);

    // println!("");
    // for c in &id1.0 {
    //     print!("{:08b} ", *c);
    // }
    // println!("");
    // for c in &id2.0 {
    //     print!("{:08b} ", *c);
    // }
    //
    // big endian - most significant byte first, 0x1 == 00000001
    // 00011000 11101000 01010101 00000000 ...
    // 00011000 10001110 01010101 00000000 ...

    assert!(!equal_subset(0, 8 * 3, &id1, &id2));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bits::test_equal_subset_out_of_bounds --exact --show-output
/// ref. "TestEqualSubsetOutOfBounds"
#[test]
fn test_equal_subset_out_of_bounds() {
    let id1 = Id::from_slice(&vec![0x18, 0xe8, 0x55]);
    let id2 = Id::from_slice(&vec![0x18, 0x8e, 0x55]);
    assert!(!equal_subset(0, 500, &id1, &id2));
}

/// Returns the "id1" index of the first different bit in the range [start, stop).
/// This does bit-per-bit comparison for the Id type of [u8; ID_LEN].
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#FirstDifferenceSubset>
pub fn first_difference_subset(start: usize, stop: usize, id1: &Id, id2: &Id) -> (usize, bool) {
    if stop == 0 {
        return (0, false);
    }

    let stop = stop - 1; // -1 for end index
    if start > stop {
        return (0, false);
    }
    if stop >= NUM_BITS {
        return (0, false);
    }

    let start_index = (start / BITS_PER_BYTES) as usize;
    let stop_index = (stop / BITS_PER_BYTES) as usize;

    let start_bit = (start % BITS_PER_BYTES) as usize; // index in the byte that the first bit is at
    let stop_bit = (stop % BITS_PER_BYTES) as usize; // index in the byte that the last bit is at

    let start_mask: i32 = -1 << start_bit; // 111...0... The number of 0s is equal to start_bit
    let stop_mask: i32 = (1 << (stop_bit + 1)) - 1; // 000...1... The number of 1s is equal to stop_bit + 1

    if start_index == stop_index {
        // if looking at same byte, both masks need to be applied
        let mask = start_mask & stop_mask;

        let b1 = mask & id1.0[start_index] as i32;
        let b2 = mask & id2.0[start_index] as i32;
        if b1 == b2 {
            return (0, false);
        }

        let bd = b1 ^ b2;
        return (
            bd.trailing_zeros() as usize + start_index * BITS_PER_BYTES,
            true,
        );
    }

    let start1 = start_mask & id1.0[start_index] as i32;
    let start2 = start_mask & id2.0[start_index] as i32;
    if start1 != start2 {
        let bd = start1 ^ start2;
        return (
            bd.trailing_zeros() as usize + start_index * BITS_PER_BYTES,
            true,
        );
    }

    // check interior bits
    for idx in (start_index + 1)..stop_index {
        let b1 = id1.0[idx];
        let b2 = id2.0[idx];
        if b1 != b2 {
            let bd = b1 ^ b2;
            return (bd.trailing_zeros() as usize + idx * BITS_PER_BYTES, true);
        }
    }

    let stop1 = stop_mask & id1.0[stop_index] as i32;
    let stop2 = stop_mask & id2.0[stop_index] as i32;
    if stop1 != stop2 {
        let bd = stop1 ^ stop2;
        return (
            bd.trailing_zeros() as usize + stop_index * BITS_PER_BYTES,
            true,
        );
    }

    (0, false) // no difference found
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bits::test_first_difference_subset --exact --show-output
#[test]
fn test_first_difference_subset() {
    // ref. TestFirstDifferenceSubsetEarlyStop
    let id1 = Id::from_slice(&vec![0xf0, 0x0f]);
    let id2 = Id::from_slice(&vec![0xf0, 0x1f]);

    // println!("");
    // for c in &id1.0 {
    //     print!("{:08b} ", *c);
    // }
    // println!("");
    // for c in &id2.0 {
    //     print!("{:08b} ", *c);
    // }
    //
    // big endian - most significant byte first, 0x1 == 00000001
    // 11110000 00001111 00000000 ...
    // 11110000 00011111 00000000 ...

    assert_eq!(first_difference_subset(0, 12, &id1, &id2), (0, false));
    assert_eq!(first_difference_subset(0, 13, &id1, &id2), (12, true));

    // ref. TestFirstDifferenceEqualByte4
    let id1 = Id::from_slice(&vec![0x10]);
    let id2 = Id::from_slice(&vec![0x00]);

    // println!("");
    // for c in &id1.0 {
    //     print!("{:08b} ", *c);
    // }
    // println!("");
    // for c in &id2.0 {
    //     print!("{:08b} ", *c);
    // }
    //
    // big endian - most significant byte first, 0x1 == 00000001
    // 00100000 00000000 ...
    // 00000000 00000000 ...

    assert_eq!(first_difference_subset(0, 4, &id1, &id2), (0, false));
    assert_eq!(first_difference_subset(0, 5, &id1, &id2), (4, true));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bits::test_first_difference_equal_byte_5 --exact --show-output
/// ref. TestFirstDifferenceEqualByte5
#[test]
fn test_first_difference_equal_byte_5() {
    let id1 = Id::from_slice(&vec![0x20]);
    let id2 = Id::from_slice(&vec![0x00]);

    // println!("");
    // for c in &id1.0 {
    //     print!("{:08b} ", *c);
    // }
    // println!("");
    // for c in &id2.0 {
    //     print!("{:08b} ", *c);
    // }
    //
    // big endian - most significant byte first, 0x1 == 00000001
    // 00100000 00000000 ...
    // 00000000 00000000 ...

    assert_eq!(first_difference_subset(0, 5, &id1, &id2), (0, false));
    assert_eq!(first_difference_subset(0, 6, &id1, &id2), (5, true));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bits::test_first_difference_subset_middle --exact --show-output
/// ref. TestFirstDifferenceSubsetMiddle
#[test]
fn test_first_difference_subset_middle() {
    let id1 = Id::from_slice(&vec![0xf0, 0x0f, 0x11]);
    let id2 = Id::from_slice(&vec![0xf0, 0x1f, 0xff]);

    // println!("");
    // for c in &id1.0 {
    //     print!("{:08b} ", *c);
    // }
    // println!("");
    // for c in &id2.0 {
    //     print!("{:08b} ", *c);
    // }
    //
    // big endian - most significant byte first, 0x1 == 00000001
    // 11110000 00001111 00010001 00000000 ...
    // 11110000 00011111 11111111 00000000 ...

    assert_eq!(first_difference_subset(0, 24, &id1, &id2), (12, true));
    assert_eq!(first_difference_subset(0, 12, &id1, &id2), (0, false));
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bits::test_first_difference_vacuous --exact --show-output
/// ref. TestFirstDifferenceVacuous
#[test]
fn test_first_difference_vacuous() {
    let id1 = Id::from_slice(&vec![0xf0, 0x0f, 0x11]);
    let id2 = Id::from_slice(&vec![0xf0, 0x1f, 0xff]);

    // println!("");
    // for c in &id1.0 {
    //     print!("{:08b} ", *c);
    // }
    // println!("");
    // for c in &id2.0 {
    //     print!("{:08b} ", *c);
    // }
    //
    // big endian - most significant byte first, 0x1 == 00000001
    // 11110000 00001111 00010001 00000000 ...
    // 11110000 00011111 11111111 00000000 ...

    assert_eq!(first_difference_subset(0, 0, &id1, &id2), (0, false));
}

#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::fmt::Debug,
    std::hash::Hash,
)]
pub enum Bit {
    Zero,
    One,
}

impl std::convert::From<usize> for Bit {
    fn from(v: usize) -> Self {
        assert!(v <= 1);
        match v {
            0 => Bit::Zero,
            1 => Bit::One,
            _ => panic!("unexpected bit value {}", v),
        }
    }
}

impl Bit {
    /// Returns the `usize` value of the enum member.
    pub fn as_usize(&self) -> usize {
        match self {
            Bit::Zero => 0,
            Bit::One => 1,
        }
    }
}

/// Set that can contain uints in the range [0, 64).
/// All functions are O(1). The zero value is the empty set.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#BitSet64>
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Set64(u64);

impl Set64 {
    pub fn new() -> Self {
        Self(0_u64)
    }

    /// Add \[i\] to the set of ints.
    pub fn add(&mut self, i: u64) {
        self.0 |= 1 << i;
    }

    /// Adds all the elements in \[s\] to this set.
    pub fn union(&mut self, s: Set64) {
        self.0 |= s.0;
    }

    /// Takes the intersection of \[s\] with this set.
    pub fn intersection(&mut self, s: Set64) {
        self.0 &= s.0;
    }

    /// Removes all the elements in \[s\] from this set.
    pub fn difference(&mut self, s: Set64) {
        // ref. *bs &^= s
        self.0 &= !(s.0);
    }

    /// Removes \[i\] from the set of ints with bitclear (AND NOT) operation.
    pub fn remove(&mut self, i: u64) {
        // ref. *bs &^= 1 << i
        self.0 &= !(1 << i);
    }

    /// Removes all elements from this set.
    pub fn clear(&mut self) {
        self.0 = 0;
    }

    /// Returns true if \[i\] was previously added to this set.
    pub fn contains(&self, i: u64) -> bool {
        (self.0 & (1 << i)) != 0
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> u32 {
        // ref. bits.OnesCount64
        u64::count_ones(self.0)
    }

    /// Returns true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for Set64 {
    fn default() -> Self {
        Self::new()
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for Set64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#16x}", self.0)
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- ids::bits::test_bit_set --exact --show-output
#[test]
fn test_bit_set() {
    let mut bs1 = Set64::new();
    assert!(bs1.len() == 0);

    bs1.add(5);
    assert!(bs1.len() == 1);
    assert!(bs1.contains(5));

    bs1.add(10);
    assert!(bs1.len() == 2);
    assert!(bs1.contains(5));
    assert!(bs1.contains(10));

    bs1.add(10);
    assert!(bs1.len() == 2);
    assert!(bs1.contains(5));
    assert!(bs1.contains(10));

    let mut bs2 = Set64::new();
    assert!(bs2.len() == 0);

    bs2.add(0);
    assert!(bs2.len() == 1);
    assert!(bs2.contains(0));

    bs2.union(bs1);
    assert!(bs1.len() == 2);
    assert!(bs1.contains(5));
    assert!(bs1.contains(10));
    assert!(bs2.len() == 3);
    assert!(bs2.contains(0));
    assert!(bs2.contains(5));
    assert!(bs2.contains(10));

    bs1.clear();
    assert!(bs1.len() == 0);
    assert!(bs2.len() == 3);
    assert!(bs2.contains(0));
    assert!(bs2.contains(5));
    assert!(bs2.contains(10));

    bs1.add(63);
    assert!(bs1.len() == 1);
    assert!(bs1.contains(63));

    bs1.add(1);
    assert!(bs1.len() == 2);
    assert!(bs1.contains(1));
    assert!(bs1.contains(63));

    bs1.remove(63);
    assert!(bs1.len() == 1);
    assert!(bs1.contains(1));

    let mut bs3 = Set64::new();
    bs3.add(0);
    bs3.add(2);
    bs3.add(5);

    let mut bs4 = Set64::new();
    bs4.add(2);
    bs4.add(5);

    bs3.intersection(bs4);

    assert!(bs3.len() == 2);
    assert!(bs3.contains(2));
    assert!(bs3.contains(5));
    assert!(bs4.len() == 2);
    assert!(bs4.contains(2));
    assert!(bs4.contains(5));

    let mut bs5 = Set64::new();
    bs5.add(7);
    bs5.add(11);
    bs5.add(9);

    let mut bs6 = Set64::new();
    bs6.add(9);
    bs6.add(11);

    bs5.difference(bs6);

    assert!(bs5.len() == 1);
    assert!(bs5.contains(7));
    assert!(bs6.len() == 2);
    assert!(bs6.contains(9));
    assert!(bs6.contains(11));
}
