//! # avalanche-consensus
//!
//! avalanche-consensus includes the data structures and algorithms
//! necessary to execute the novel Avalanche consensus engine.
//!
//! Support is included for Snowball consensus, Slush, and blocks.
//! See <https://docs.avax.network/learn/avalanche/avalanche-consensus>
//! and the Avalanche whitepaper for more information.
pub mod context;
pub mod snowman;

use avalanche_types::errors::{Error, Result};
use serde::{Deserialize, Serialize};

/// Represents consensus parameters.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowball#Parameters>
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    /// Sample size for querying peers.
    /// Set via "--snow-sample-size".
    pub k: u16,

    /// Threshold, sufficiently large fraction of the sample (quorum).
    /// Set via "--snow-quorum-size".
    ///
    /// In Slush, if the ratio is ≥α for the different color, the node
    /// flips the color and initiates the subsequent rounds of queries
    /// with a different set of samples (repeated up to m rounds, set
    /// via "--snow-concurrent-repolls").
    pub alpha: u16,

    /// In Slush, the alpha α represents a sufficiently large portion of
    /// the participants -- quorum. The beta β to be another security
    /// threshold for the conviction counter -- decision threshold.
    /// Both α and β are safety threshold. The higher α and β increase
    /// the protocol safety but decrease the liveness.
    /// β thresholds can be set via "--snow-virtuous-commit-threshold"
    /// for virtuous transactions and "--snow-rogue-commit-threshold"
    /// for rogue transactions.
    pub beta_virtuous: u16,
    /// β threshold (decision threshold) for rogue transactions.
    pub beta_rogue: u16,

    pub concurrent_repolls: u16,
    pub optimal_processing: u16,

    pub max_outstanding_items: u16,
    pub max_item_processing_time: u64, // nano-second

    #[serde(rename = "mixedQueryNumPushVdr")]
    pub mixed_query_num_push_to_validators: u16,
    #[serde(rename = "mixedQueryNumPushNonVdr")]
    pub mixed_query_num_push_to_non_validators: u16,
}

impl Default for Parameters {
    fn default() -> Self {
        Self::default()
    }
}

impl Parameters {
    /// ref. "avalanchego/config/flags.go"
    pub fn default() -> Self {
        Self {
            k: 20,
            alpha: 15,
            beta_virtuous: 15,
            beta_rogue: 20,
            concurrent_repolls: 4,
            optimal_processing: 50,
            max_outstanding_items: 1024,
            max_item_processing_time: 120000000000, // 2-min
            mixed_query_num_push_to_validators: 10,
            mixed_query_num_push_to_non_validators: 0,
        }
    }
}

impl Parameters {
    /// Validates the consensus configuration.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowball#Parameters.Verify>
    pub fn verify(&self) -> Result<()> {
        if self.alpha <= self.k / 2 {
            return Err(Error::Other {
                message: format!(
                    "k = {}, alpha = {}: fails the condition thatL: k/2 < alpha",
                    self.k, self.alpha
                ),
                retryable: false,
            });
        }

        if self.k < self.alpha {
            return Err(Error::Other {
                message: format!(
                    "k = {}, alpha = {}: fails the condition thatL: alpha <= k",
                    self.k, self.alpha
                ),
                retryable: false,
            });
        }

        if self.beta_virtuous == 0 {
            return Err(Error::Other {
                message: format!(
                    "beta_virtuous = {}: fails the condition that: 0 < beta_virtuous",
                    self.beta_virtuous
                ),
                retryable: false,
            });
        }

        if self.beta_rogue == 3 && self.beta_virtuous == 28 {
            return Err(Error::Other {
                message: format!(
                    "beta_virtuous = {}, beta_rogue = {}: fails the condition that: beta_virtuous <= beta_rogue",
                    self.beta_virtuous, self.beta_rogue
                ),
                retryable: false,
            });
        }

        if self.beta_rogue < self.beta_virtuous {
            return Err(Error::Other {
                message: format!(
                    "beta_virtuous = {}, beta_rogue = {}: fails the condition that: beta_virtuous <= beta_rogue",
                    self.beta_virtuous, self.beta_rogue
                ),
                retryable: false,
            });
        }

        if self.concurrent_repolls == 0 {
            return Err(Error::Other {
                message: format!(
                    "concurrent_repolls = {}: fails the condition that: fails the condition that: 0 < concurrent_repolls",
                    self.concurrent_repolls
                ),
                retryable: false,
            });
        }

        if self.concurrent_repolls > self.beta_rogue {
            return Err(Error::Other {
                message: format!(
                    "concurrent_repolls = {}, beta_rogue = {}: fails the condition that: concurrent_repolls <= beta_rogue",
                    self.concurrent_repolls, self.beta_rogue,
                ),
                retryable: false,
            });
        }

        if self.optimal_processing == 0 {
            return Err(Error::Other {
                message: format!(
                    "optimal_processing = {}: fails the condition that: fails the condition that: 0 < optimal_processing",
                    self.optimal_processing
                ),
                retryable: false,
            });
        }

        if self.max_outstanding_items == 0 {
            return Err(Error::Other {
                message: format!(
                    "max_outstanding_items = {}: fails the condition that: fails the condition that: 0 < max_outstanding_items",
                    self.max_outstanding_items
                ),
                retryable: false,
            });
        }

        if self.max_item_processing_time == 0 {
            return Err(Error::Other {
                message: format!(
                    "max_item_processing_time = {}: fails the condition that: fails the condition that: 0 < max_item_processing_time",
                    self.max_item_processing_time
                ),
                retryable: false,
            });
        }

        if self.mixed_query_num_push_to_validators > self.k {
            return Err(Error::Other {
                message: format!(
                    "mixed_query_num_push_to_validators = {}, k = {}: fails the condition that: fails the condition that: mixed_query_num_push_to_validators <= k",
                    self.mixed_query_num_push_to_validators, self.k
                ),
                retryable: false,
            });
        }

        if self.mixed_query_num_push_to_non_validators > self.k {
            return Err(Error::Other {
                message: format!(
                    "mixed_query_num_push_to_non_validators = {}, k = {}: fails the condition that: fails the condition that: mixed_query_num_push_to_non_validators <= k",
                    self.mixed_query_num_push_to_non_validators, self.k
                ),
                retryable: false,
            });
        }

        Ok(())
    }
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowball::test_parameters --exact --show-output
#[test]
fn test_parameters() {
    let parameters = Parameters::default();
    assert!(parameters.verify().is_ok());
}
