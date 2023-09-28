//! Binary slush instance algorithm.
pub mod node;

use std::cell::Cell;

/// Implements a binary slush instance.
/// ref. <https://github.com/ava-labs/avalanchego/blob/master/snow/consensus/snowball/binary_slush.go> "binarySlush"
#[derive(Clone, Debug)]
pub struct Slush {
    /// Represents the last choice (or preference) of the last successful poll.
    /// Set to the initial choice until there's another successful poll.
    preference: Cell<i64>,
}

impl Slush {
    pub fn new(choice: i64) -> Self {
        Self {
            preference: Cell::new(choice),
        }
    }

    pub fn preference(&self) -> i64 {
        self.preference.get()
    }

    pub fn record_successful_poll(&self, choice: i64) {
        self.preference.set(choice);
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for Slush {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SL(Preference = {})", self.preference())
    }
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::binary::test_slush --exact --show-output
#[test]
fn test_slush() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let sl = Slush::new(0);
    assert_eq!(sl.preference(), 0);

    sl.record_successful_poll(1);
    assert_eq!(sl.preference(), 1);
}

/// Implements a binary snowflake instance.
/// ref. <https://github.com/ava-labs/avalanchego/blob/master/snow/consensus/snowball/binary_snowflake.go> "binarySnowflake"
#[derive(Clone, Debug)]
pub struct Snowflake {
    /// Wraps the slush logic.
    slush: Slush,

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

    /// Set to "true" once the required number of "consecutive" polls
    /// has been reached. This is used for preventing the state change.
    finalized: Cell<bool>,
}

impl Snowflake {
    pub fn new(beta: i64, choice: i64, confidence: i64, finalized: bool) -> Self {
        Self {
            slush: Slush::new(choice),
            beta: Cell::new(beta),
            confidence: Cell::new(confidence),
            finalized: Cell::new(finalized),
        }
    }

    pub fn preference(&self) -> i64 {
        self.slush.preference()
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

    pub fn record_successful_poll(&self, choice: i64) {
        if self.finalized() {
            // already decided
            return;
        }

        if self.preference() == choice {
            self.confidence.set(self.confidence.get() + 1);
        } else {
            // 1 because this poll itself is a successful poll
            self.confidence.set(1);
        }

        self.finalized.set(self.confidence.get() >= self.beta());
        self.slush.record_successful_poll(choice);
    }

    pub fn record_unsuccessful_poll(&self) {
        self.confidence.set(0);
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for Snowflake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SF(Confidence = {}, Finalized = {}, {})",
            self.confidence(),
            self.finalized(),
            self.slush
        )
    }
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::binary::test_snowflake --exact --show-output
/// ref. "TestBinarySnowflake"
#[test]
fn test_snowflake() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let beta = 2_i64;
    let (blue, red) = (0_i64, 1_i64);

    // initial choice "red" "1"
    let snf = Snowflake::new(beta, red, 0, false);
    assert_eq!(snf.beta(), beta);
    assert_eq!(snf.preference(), red);
    assert!(!snf.finalized(), "finalized too early");

    // slush changes its preference to "blue" on this successful poll
    snf.record_successful_poll(blue);
    assert_eq!(snf.preference(), blue);
    assert!(!snf.finalized(), "finalized too early");

    // slush changes its preference to "red" on this successful poll
    snf.record_successful_poll(red);
    assert_eq!(snf.preference(), red);
    assert!(!snf.finalized(), "finalized too early");

    // slush changes its preference to "blue" on this successful poll
    snf.record_successful_poll(blue);
    assert_eq!(snf.preference(), blue);
    assert!(!snf.finalized(), "finalized too early");

    // reaching the threshold of 2 with two consecutive polls
    snf.record_successful_poll(blue);
    assert_eq!(snf.preference(), blue);
    assert!(snf.finalized(), "didn't finalize correctly");

    log::info!("{snf}");
}

/// Implements a binary snowball instance.
/// ref. <https://github.com/ava-labs/avalanchego/blob/master/snow/consensus/snowball/binary_snowball.go> "binarySnowball"
#[derive(Clone, Debug)]
pub struct Snowball {
    /// Wraps the binary snowflake logic.
    snowflake: Snowflake,

    /// Represents the choice with the largest number of "consecutive" successful polls.
    /// Ties are broken by switching the choice lazily.
    preference: Cell<i64>,

    /// Represents the total number of successful network polls between 0 and 1.
    num_successful_polls: Cell<[i64; 2]>,
}

impl Snowball {
    pub fn new(snowflake: Snowflake, choice: i64, polls: [i64; 2]) -> Self {
        Self {
            snowflake,
            preference: Cell::new(choice),
            num_successful_polls: Cell::new(polls),
        }
    }

    pub fn preference(&self) -> i64 {
        // It is possible, with low probability, that the snowflake preference is
        // not equal to the snowball preference when snowflake finalizes.
        // However, this case is handled for completion.
        // Therefore, if snowflake is finalized, then our finalized snowflake
        // choice should be preferred.
        if self.snowflake.finalized() {
            return self.snowflake.preference();
        }
        self.preference.get()
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

    pub fn num_successful_polls(&self, idx: usize) -> i64 {
        let polls = self.num_successful_polls.take();
        let n = polls[idx];
        self.num_successful_polls.set(polls);
        n
    }

    pub fn record_successful_poll(&self, choice: i64) {
        let mut polls = self.num_successful_polls.take();
        polls[choice as usize] += 1;

        let (count, count_other) = (polls[choice as usize], polls[1 - choice as usize]);
        self.num_successful_polls.set(polls);

        if count > count_other {
            self.preference.set(choice);
        }

        self.snowflake.record_successful_poll(choice);
    }

    pub fn record_unsuccessful_poll(&self) {
        self.snowflake.record_unsuccessful_poll()
    }
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for Snowball {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SB(Preference = {}, NumSuccessfulPolls[0] = {}, NumSuccessfulPolls[1] = {}, {})",
            self.preference.get(),
            self.num_successful_polls(0),
            self.num_successful_polls(1),
            self.snowflake
        )
    }
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::binary::test_snowball --exact --show-output
/// ref. "TestBinarySnowball"
#[test]
fn test_snowball() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let beta = 2_i64;
    let (red, blue) = (0_i64, 1_i64);
    let snb = Snowball::new(Snowflake::new(beta, red, 0, false), red, [0, 0]);
    assert_eq!(snb.beta(), beta);
    assert_eq!(snb.preference(), red);
    assert!(!snb.finalized(), "finalized too early");

    // initial choice "red" count is 0,
    // thus one successful poll on "blue" should flip the preference
    snb.record_successful_poll(blue);
    assert_eq!(snb.preference(), blue);
    assert!(!snb.finalized(), "finalized too early");

    // preference is only updated when the count other is greater than current
    snb.record_successful_poll(red);
    assert_eq!(snb.preference(), blue);
    assert!(!snb.finalized(), "finalized too early");

    snb.record_successful_poll(blue);
    assert_eq!(snb.preference(), blue);
    assert!(!snb.finalized(), "finalized too early");

    // now confidence >= beta
    snb.record_successful_poll(blue);
    assert_eq!(snb.preference(), blue);
    assert!(snb.finalized(), "didn't finalize correctly");

    log::info!("{snb}");
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::binary::test_snowball_record_unsuccessful_poll --exact --show-output
/// ref. "TestBinarySnowballRecordUnsuccessfulPoll"
#[test]
fn test_snowball_record_unsuccessful_poll() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let beta = 2_i64;
    let (red, blue) = (0_i64, 1_i64);
    let snb = Snowball::new(Snowflake::new(beta, red, 0, false), red, [0, 0]);
    assert_eq!(snb.beta(), beta);
    assert_eq!(snb.preference(), red);
    assert!(!snb.finalized());

    // initial choice "red" count is 0,
    // thus one successful poll on "blue" should flip the preference
    snb.record_successful_poll(blue);
    assert_eq!(snb.preference(), blue);
    assert!(!snb.finalized(), "finalized too early");

    snb.record_unsuccessful_poll();
    snb.record_successful_poll(blue);
    assert_eq!(snb.preference(), blue);
    assert!(!snb.finalized(), "finalized too early");

    // now confidence >= beta
    snb.record_successful_poll(blue);
    assert_eq!(snb.preference(), blue);
    assert!(snb.finalized(), "finalized too late");
    assert_eq!(snb.num_successful_polls(red as usize), 0);
    assert_eq!(snb.num_successful_polls(blue as usize), 3);

    log::info!("{snb}");
    assert_eq!(snb.to_string(), "SB(Preference = 1, NumSuccessfulPolls[0] = 0, NumSuccessfulPolls[1] = 3, SF(Confidence = 2, Finalized = true, SL(Preference = 1)))");
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::binary::test_snowball_accept_weird_color --exact --show-output
/// ref. "TestBinarySnowballAcceptWeirdColor"
#[test]
fn test_snowball_accept_weird_color() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let beta = 2_i64;
    let (blue, red) = (0_i64, 1_i64);
    let snb = Snowball::new(Snowflake::new(beta, red, 0, false), red, [0, 0]);
    assert_eq!(snb.beta(), beta);
    assert_eq!(snb.preference(), red);
    assert!(!snb.finalized());

    snb.record_successful_poll(red);
    snb.record_unsuccessful_poll();
    assert_eq!(snb.preference(), red);
    assert!(!snb.finalized(), "finalized too early");

    snb.record_successful_poll(red);
    snb.record_unsuccessful_poll();
    assert_eq!(snb.preference(), red);
    assert!(!snb.finalized(), "finalized too early");

    // preference is only updated when the count other is greater than current
    snb.record_successful_poll(blue);
    assert_eq!(snb.preference(), red);
    assert!(!snb.finalized(), "finalized too early");

    // now confidence >= beta
    snb.record_successful_poll(blue);
    assert_eq!(snb.preference(), blue);
    assert!(snb.finalized(), "finalized too late");

    log::info!("{snb}");
    assert_eq!(snb.to_string(), "SB(Preference = 1, NumSuccessfulPolls[0] = 2, NumSuccessfulPolls[1] = 2, SF(Confidence = 2, Finalized = true, SL(Preference = 0)))");
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::snowball::binary::test_snowball_lock_color --exact --show-output
/// ref. "TestBinarySnowballLockColor"
#[test]
fn test_snowball_lock_color() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let beta = 1_i64;
    let (red, blue) = (0_i64, 1_i64);
    let snb = Snowball::new(Snowflake::new(beta, red, 0, false), red, [0, 0]);
    assert_eq!(snb.beta(), beta);
    assert_eq!(snb.preference(), red);
    assert!(!snb.finalized(), "finalized too early");

    // now confidence >= beta
    snb.record_successful_poll(red);
    assert_eq!(snb.preference(), red);
    assert!(snb.finalized(), "finalized too late");

    // cannot flip the preference once finalized
    snb.record_successful_poll(blue);
    assert_eq!(snb.preference(), red);
    assert!(snb.finalized(), "finalized too late");

    // cannot flip the preference once finalized
    snb.record_successful_poll(blue);
    assert_eq!(snb.preference(), red);
    assert!(snb.finalized(), "finalized too late");

    log::info!("{snb}");
    assert_eq!(snb.to_string(), "SB(Preference = 1, NumSuccessfulPolls[0] = 1, NumSuccessfulPolls[1] = 2, SF(Confidence = 1, Finalized = true, SL(Preference = 0)))");
}
