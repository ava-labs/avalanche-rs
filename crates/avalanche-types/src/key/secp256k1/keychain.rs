use std::collections::HashMap;

use crate::{ids::short, key};
use serde::{Deserialize, Serialize};

/// Support multiple keys as a chain.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#Keychain>
/// ref. <https://github.com/ava-labs/avalanchego/blob/v1.7.9/wallet/chain/p/builder.go>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Keychain<T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly> {
    pub keys: Vec<T>,
    pub short_addr_to_key_index: HashMap<short::Id, u32>,
}

impl<T> Keychain<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
{
    pub fn new(keys: Vec<T>) -> Self {
        let mut short_addr_to_key_index = HashMap::new();
        for (pos, k) in keys.iter().enumerate() {
            short_addr_to_key_index.insert(k.short_address().unwrap(), pos as u32);
        }
        Self {
            keys,
            short_addr_to_key_index,
        }
    }

    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#Keychain.Get>
    pub fn get(&self, short_addr: &short::Id) -> Option<T> {
        self.short_addr_to_key_index
            .get(short_addr)
            .map(|k| self.keys[(*k) as usize].clone())
    }

    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#Keychain.Match>
    pub fn match_threshold(
        &self,
        output_owners: &key::secp256k1::txs::OutputOwners,
        time: u64,
    ) -> Option<(Vec<u32>, Vec<T>)> {
        if output_owners.locktime > time {
            // output owners are still locked
            return None;
        }

        let mut sig_indices: Vec<u32> = Vec::new();
        let mut keys: Vec<T> = Vec::new();
        for (pos, addr) in output_owners.addresses.iter().enumerate() {
            let key = self.get(addr);
            if key.is_none() {
                continue;
            }
            sig_indices.push(pos as u32);
            keys.push(key.unwrap());

            if (keys.len() as u32) == output_owners.threshold {
                break;
            }
        }

        let n = keys.len();
        if (n as u32) == output_owners.threshold {
            Some((sig_indices, keys))
        } else {
            None
        }
    }

    /// Returns "None" if the threshold is NOT met.
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/secp256k1fx#Keychain.Spend>
    /// TODO: support spend on "secp256k1fx::MintOutput"
    pub fn spend(
        &self,
        output: &key::secp256k1::txs::transfer::Output,
        time: u64,
    ) -> Option<(key::secp256k1::txs::transfer::Input, Vec<T>)> {
        let res = self.match_threshold(&output.output_owners, time);
        let threshold_met = res.is_some();
        if !threshold_met {
            return None;
        }

        let (sig_indices, keys) = res.unwrap();
        Some((
            key::secp256k1::txs::transfer::Input {
                amount: output.amount,
                sig_indices,
            },
            keys,
        ))
    }
}
