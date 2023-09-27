//! Unary snowflake algorithm.
pub mod node;

use std::cell::Cell;

use crate::snowman::snowball::binary;

/// Implements a unary snowflake instance (BFT).
/// ref. <https://github.com/ava-labs/avalanchego/blob/master/snow/consensus/snowball/unary_snowflake.go> "unarySnowflake"
#[derive(Clone, Debug)]
pub struct Snowflake {
    /// Represents the number of "consecutive" successful queries
    /// required for the finalization (e.g., quorum).
    ///
    /// The alpha α represents a sufficiently large portion of the
    /// participants -- quorum. The beta β is another security threshold
    /// for the conviction counter -- decision threshold.
    beta: Cell<i64>,

    /// Represents the number of "consecutive" successful polls
    /// that have returned the preference.
    confidence: Cell<i64>,

    /// Set "true" when the required number of "consecutive" polls has
    /// been reached. This is used for preventing the state change.
    finalized: Cell<bool>,
}

impl Snowflake {
    pub fn new(beta: i64) -> Self {
        Self {
            beta: Cell::new(beta),
            confidence: Cell::new(0),
            finalized: Cell::new(false),
        }
    }

    pub fn beta(&self) -> i64 {
        self.beta.get()
    }

    pub fn confidence(&self) -> i64 {
        self.confidence.get()
    }

    pub fn finalized(&self) -> bool {
        self.finalized.get()
    }

    pub fn record_successful_poll(&self) {
        let confidence = self.confidence.get() + 1;
        self.confidence.set(confidence);

        if !self.finalized.get() {
            self.finalized.set(confidence >= self.beta());
        }
    }

    pub fn record_unsuccessful_poll(&self) {
        self.confidence.set(0);
    }

    /// Extends to the binary snowflake instance with the `choice` as the preference.
    /// ref. "avalanchego/snow/consensus/snowball.unarySnowflake.Extend"
    pub fn extend(&self, beta: i64, choice: i64) -> binary::Snowflake {
        binary::Snowflake::new(beta, choice, self.confidence(), self.finalized())
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for Snowflake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SF(Confidence = {}, Finalized = {})",
            self.confidence(),
            self.finalized()
        )
    }
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::unary::test_snowflake --exact --show-output
/// ref. "TestUnarySnowflake"
#[test]
fn test_snowflake() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let beta = 2_i64;
    let snf = Snowflake::new(beta);

    snf.record_successful_poll();
    assert_eq!(snf.beta(), beta);
    assert_eq!(snf.confidence(), 1);
    assert!(!snf.finalized());

    snf.record_unsuccessful_poll();
    assert_eq!(snf.beta(), beta);
    assert_eq!(snf.confidence(), 0);
    assert!(!snf.finalized());

    // only one successful poll, so not finalized yet
    snf.record_successful_poll();
    assert_eq!(snf.beta(), beta);
    assert_eq!(snf.confidence(), 1);
    assert!(!snf.finalized());

    // beta must've been reached
    // after two consecutive successful polls
    snf.record_successful_poll();
    assert_eq!(snf.beta(), beta);
    assert_eq!(snf.confidence(), 2);
    assert!(snf.finalized());

    // "finalized" should not change once finalized before
    snf.record_unsuccessful_poll();
    assert_eq!(snf.beta(), beta);
    assert_eq!(snf.confidence(), 0);
    assert!(snf.finalized());

    // "finalized" should not change once finalized before
    snf.record_successful_poll();
    assert_eq!(snf.beta(), beta);
    assert_eq!(snf.confidence(), 1);
    assert!(snf.finalized());

    log::info!("{snf}");
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::unary::test_snowflake_extend --exact --show-output
/// ref. "TestUnarySnowflake"
#[test]
fn test_snowflake_extend() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let beta = 2_i64;
    let snf = Snowflake::new(beta);
    assert_eq!(snf.beta(), beta);

    snf.record_successful_poll();
    assert_eq!(snf.confidence(), 1);
    assert!(!snf.finalized());

    snf.record_unsuccessful_poll();
    assert_eq!(snf.confidence(), 0);
    assert!(!snf.finalized());

    // only one successful poll thus "not" finalized yet
    snf.record_successful_poll();
    assert_eq!(snf.confidence(), 1);
    assert!(!snf.finalized());

    let snf_binary = snf.extend(beta, 0);
    assert_eq!(snf_binary.beta(), beta);

    snf_binary.record_unsuccessful_poll();
    snf_binary.record_successful_poll(1);
    assert!(!snf_binary.finalized());

    // two consecutive polls on the choice "1"
    // reaching the threshold "beta" 2, thus finalized
    snf_binary.record_successful_poll(1);
    assert_eq!(snf_binary.preference(), 1);
    assert!(snf_binary.finalized());

    log::info!("{snf_binary}");

    // one more consecutive successful thus finalized
    snf.record_successful_poll();
    assert_eq!(snf.confidence(), 2);
    assert!(snf.finalized());

    // another unsucessful poll should NOT change the finalized state
    snf.record_unsuccessful_poll();
    assert_eq!(snf.confidence(), 0);
    assert!(snf.finalized());

    snf.record_successful_poll();
    assert_eq!(snf.confidence(), 1);
    assert!(snf.finalized());
}

/// Implements a unary snowball instance.
/// ref. <https://github.com/ava-labs/avalanchego/blob/master/snow/consensus/snowball/unary_snowball.go> "unarySnowball"
#[derive(Clone, Debug)]
pub struct Snowball {
    /// Wraps the unary snowflake logic.
    snowflake: Snowflake,

    /// Represents the total number of successful network polls.
    num_successful_polls: Cell<i64>,
}

impl Snowball {
    pub fn new(beta: i64) -> Self {
        Self {
            snowflake: Snowflake::new(beta),
            num_successful_polls: Cell::new(0),
        }
    }

    pub fn beta(&self) -> i64 {
        self.snowflake.beta()
    }

    pub fn confidence(&self) -> i64 {
        self.snowflake.confidence()
    }

    pub fn finalized(&self) -> bool {
        self.snowflake.finalized()
    }

    pub fn num_successful_polls(&self) -> i64 {
        self.num_successful_polls.get()
    }

    pub fn record_successful_poll(&self) {
        let polls = self.num_successful_polls.get() + 1;
        self.num_successful_polls.set(polls);

        self.snowflake.record_successful_poll();
    }

    pub fn record_unsuccessful_poll(&self) {
        self.snowflake.record_unsuccessful_poll()
    }

    /// Extends to the binary snowball instance with the `choice` as the preference.
    pub fn extend(&self, beta: i64, choice: i64) -> binary::Snowball {
        let mut polls = [0_i64, 0_i64];
        polls[choice as usize] = self.num_successful_polls.get();

        binary::Snowball::new(
            binary::Snowflake::new(
                beta,
                choice,
                self.snowflake.confidence(),
                self.snowflake.finalized(),
            ),
            choice,
            polls,
        )
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for Snowball {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SB(NumSuccessfulPolls = {}, {})",
            self.num_successful_polls(),
            self.snowflake
        )
    }
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::unary::test_snowball_unary --exact --show-output
/// ref. "TestUnarySnowball"
#[test]
fn test_snowball_unary() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let beta = 2_i64;
    let snb = Snowball::new(beta);
    assert_eq!(snb.beta(), beta);

    snb.record_successful_poll();
    assert_eq!(snb.confidence(), 1);
    assert!(!snb.finalized());
    assert_eq!(snb.num_successful_polls(), 1);

    snb.record_unsuccessful_poll();
    assert_eq!(snb.confidence(), 0);
    assert!(!snb.finalized());
    assert_eq!(snb.num_successful_polls(), 1);

    // total "two" successful but not consecutive
    // thus not finalized yet
    snb.record_successful_poll();
    assert_eq!(snb.confidence(), 1);
    assert!(!snb.finalized());
    assert_eq!(snb.num_successful_polls(), 2);

    // "two" consecutive polls thus finalized
    snb.record_successful_poll();
    assert_eq!(snb.beta(), beta);
    assert_eq!(snb.confidence(), 2);
    assert!(snb.finalized());
    assert_eq!(snb.num_successful_polls(), 3);

    // following unsuccessful poll should not change the finalized state
    snb.record_unsuccessful_poll();
    assert_eq!(snb.confidence(), 0);
    assert!(snb.finalized());

    snb.record_successful_poll();
    assert_eq!(snb.confidence(), 1);
    assert!(snb.finalized());

    log::info!("{snb}");
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::unary::test_snowball_extend --exact --show-output
/// ref. "TestUnarySnowball"
#[test]
fn test_snowball_extend() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let beta = 2_i64;
    let snb = Snowball::new(beta);
    assert_eq!(snb.beta(), beta);

    snb.record_successful_poll();
    assert_eq!(snb.confidence(), 1);
    assert!(!snb.finalized());
    assert_eq!(snb.num_successful_polls(), 1);

    snb.record_unsuccessful_poll();
    assert_eq!(snb.confidence(), 0);
    assert!(!snb.finalized());
    assert_eq!(snb.num_successful_polls(), 1);

    snb.record_successful_poll();
    assert_eq!(snb.confidence(), 1);
    assert!(!snb.finalized());
    assert_eq!(snb.num_successful_polls(), 2);

    let snb_binary = snb.extend(beta, 0);
    log::info!("{snb_binary}");
    assert_eq!(snb_binary.to_string(), "SB(Preference = 0, NumSuccessfulPolls[0] = 2, NumSuccessfulPolls[1] = 0, SF(Confidence = 1, Finalized = false, SL(Preference = 0)))");
    assert_eq!(snb_binary.beta(), beta);
    assert_eq!(snb_binary.confidence(), 1);
    assert!(!snb_binary.finalized());
    assert_eq!(snb_binary.preference(), 0);
    assert_eq!(snb_binary.num_successful_polls(0), 2);
    assert_eq!(snb_binary.num_successful_polls(1), 0);

    snb_binary.record_unsuccessful_poll();
    for _ in 0..3 {
        assert_eq!(snb_binary.preference(), 0);
        assert!(!snb_binary.finalized());

        snb_binary.record_successful_poll(1);
        snb_binary.record_unsuccessful_poll();
    }
    assert_eq!(snb_binary.confidence(), 0);

    // no consecutive successful poll >= beta yet
    assert!(!snb_binary.finalized());

    assert_eq!(snb_binary.preference(), 1);
    assert_eq!(snb_binary.num_successful_polls(0), 2);
    assert_eq!(snb_binary.num_successful_polls(1), 3);

    // one consecutive poll for the choice "1"
    snb_binary.record_successful_poll(1);
    assert_eq!(snb_binary.preference(), 1);
    assert_eq!(snb_binary.confidence(), 1);
    assert!(!snb_binary.finalized());

    // two consecutive polls for the choice "1"
    snb_binary.record_successful_poll(1);
    assert_eq!(snb_binary.preference(), 1);
    assert_eq!(snb_binary.confidence(), 2);
    assert!(snb_binary.finalized());

    log::info!("{snb}");
    assert_eq!(
        snb.to_string(),
        "SB(NumSuccessfulPolls = 2, SF(Confidence = 1, Finalized = false))"
    );
}
