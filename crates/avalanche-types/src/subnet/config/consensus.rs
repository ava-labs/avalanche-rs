use serde::{Deserialize, Serialize};

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowball#Parameters>
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/avalanche#Parameters>
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SnowballParameters {
    /// Sample size.
    pub k: i32,
    /// Quorum size.
    pub alpha: i32,

    /// Virtuous commit threshold.
    pub beta_virtuous: i32,
    /// Rogue commit threshold.
    pub beta_rogue: i32,

    /// Minimum number of concurrent polls for finalizing consensus.
    pub concurrent_repolls: i32,
    /// Optimal number of processing containers in consensus.
    pub optimal_processing: i32,

    /// Maximum number of processing items to be considered healthy.
    pub max_outstanding_items: i32,
    /// Maximum amount of time an item should be processing and still be healthy.
    pub max_item_processing_time: i64,

    pub mixed_query_num_push_vdr: i32,
    pub mixed_query_num_push_non_vdr: i32,
}

impl Default for SnowballParameters {
    fn default() -> Self {
        Self::default()
    }
}

impl SnowballParameters {
    /// The defaults do not match with the ones in avalanchego,
    /// as this is for avalanche-ops based deployments.
    pub fn default() -> Self {
        Self {
            k: 20,
            alpha: 15,
            beta_virtuous: 15,
            beta_rogue: 20,
            concurrent_repolls: 4,
            optimal_processing: 50,
            max_outstanding_items: 1024,
            max_item_processing_time: 2 * 1000 * 1000 * 1000, // 2-minute
            mixed_query_num_push_vdr: 10,
            mixed_query_num_push_non_vdr: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    /// Embeds "SnowballParameters" at the same level as other fields.
    #[serde(flatten)]
    pub snowball_parameters: SnowballParameters,
    pub parents: i32,
    pub batch_size: i32,
}

impl Default for Parameters {
    fn default() -> Self {
        Self::default()
    }
}

impl Parameters {
    /// The defaults do not match with the ones in avalanchego,
    /// as this is for avalanche-ops based deployments.
    pub fn default() -> Self {
        Self {
            snowball_parameters: SnowballParameters::default(),
            parents: 5,
            batch_size: 30,
        }
    }
}
