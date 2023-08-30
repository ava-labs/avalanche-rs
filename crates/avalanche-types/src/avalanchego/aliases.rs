//! Chain alias functionality for the avalanchego node.
use std::collections::HashMap;

/// A map from blockchainIDs to custom aliases
/// ref. <https://docs.avax.network/nodes/maintain/avalanchego-config-flags#--chain-aliases-file-string>.
pub type Aliases = HashMap<String, Vec<String>>;

#[test]
fn test_aliases_json() {
    let json = r#"{"q2aTwKuyzgs8pynF7UXBZCU7DejbZbZ6EUyHr3JQzYgwNPUPi": ["DFK"]}"#;
    let aliases: Aliases = serde_json::from_str(json).unwrap();
    assert_eq!(
        aliases
            .get("q2aTwKuyzgs8pynF7UXBZCU7DejbZbZ6EUyHr3JQzYgwNPUPi")
            .unwrap(),
        &vec!["DFK".to_string()]
    )
}
