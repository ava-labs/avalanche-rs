//! Credential transaction type.
use crate::{ids, key};
use serde::{Deserialize, Serialize};

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/avm#FxCredential>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Credential {
    pub fx_id: ids::Id, // skip serialization due to serialize:"false"
    pub cred: key::secp256k1::txs::Credential,
}

impl Default for Credential {
    fn default() -> Self {
        Self {
            fx_id: ids::Id::empty(),
            cred: key::secp256k1::txs::Credential::default(),
        }
    }
}
