//! Constants for the Avalanche network.
use std::collections::HashMap;

use lazy_static::lazy_static;

pub const DEFAULT_CUSTOM_NETWORK_ID: u32 = 1000000;

pub const FALLBACK_HRP: &str = "custom";

lazy_static! {
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/constants>
    pub static ref NETWORK_ID_TO_NETWORK_NAME: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(1, "mainnet");
        m.insert(2, "cascade");
        m.insert(3, "denali");
        m.insert(4, "everest");
        m.insert(5, "fuji");
        m.insert(12345, "local");
        m
    };

    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/constants>
    pub static ref NETWORK_NAME_TO_NETWORK_ID: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        m.insert("mainnet", 1);
        m.insert("cascade", 2);
        m.insert("denali", 3);
        m.insert("everest", 4);
        m.insert("fuji", 5);
        m.insert("local", 12345);
        m
    };

    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/constants>
    pub static ref NETWORK_ID_TO_HRP: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(1, "avax");
        m.insert(2, "cascade");
        m.insert(3, "denali");
        m.insert(4, "everest");
        m.insert(5, "fuji");
        m.insert(12345, "local");
        m
    };

    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/constants>
    pub static ref HRP_TO_NETWORK_ID: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        m.insert("avax", 1);
        m.insert("cascade", 2);
        m.insert("denali", 3);
        m.insert("everest", 4);
        m.insert("fuji", 5);
        m.insert("local", 12345);
        m
    };
}
