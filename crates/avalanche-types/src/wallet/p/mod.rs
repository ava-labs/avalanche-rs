pub mod add_permissionless_validator;
pub mod add_subnet_validator;
pub mod add_validator;
pub mod create_chain;
pub mod create_subnet;
pub mod export;
pub mod import;

use std::{cmp, time::SystemTime};

use crate::{
    errors::{Error, Result},
    ids::{self, node},
    jsonrpc::client::p as client_p,
    key, platformvm, txs, wallet,
};

impl<T> wallet::Wallet<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
{
    #[must_use]
    pub fn p(&self) -> P<T> {
        P {
            inner: self.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct P<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
{
    pub inner: crate::wallet::Wallet<T>,
}

impl<T> P<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
{
    /// Fetches the current balance of the wallet owner from the specified HTTP endpoint.
    pub async fn balance_with_endpoint(&self, http_rpc: &str) -> Result<u64> {
        let resp = client_p::get_balance(http_rpc, &self.inner.p_address).await?;
        let cur_balance = resp
            .result
            .expect("unexpected None GetBalanceResult")
            .balance;
        Ok(cur_balance)
    }

    /// Fetches the current balance of the wallet owner from all endpoints
    /// in the same order of "self.http_rpcs".
    pub async fn balances(&self) -> Result<Vec<u64>> {
        let mut balances = Vec::new();
        for http_rpc in self.inner.base_http_urls.iter() {
            let balance = self.balance_with_endpoint(http_rpc).await?;
            balances.push(balance);
        }
        Ok(balances)
    }

    /// Fetches the current balance of the wallet owner.
    pub async fn balance(&self) -> Result<u64> {
        self.balance_with_endpoint(&self.inner.pick_base_http_url().1)
            .await
    }

    /// Fetches UTXOs for "P" chain.
    /// TODO: cache this like avalanchego
    pub async fn utxos(&self) -> Result<Vec<txs::utxo::Utxo>> {
        let resp =
            client_p::get_utxos(&self.inner.pick_base_http_url().1, &self.inner.p_address).await?;
        let utxos = resp
            .result
            .expect("unexpected None GetUtxosResult")
            .utxos
            .expect("unexpected None Utxos");
        Ok(utxos)
    }

    /// Returns "true" if the node_id is a current primary network validator.
    pub async fn is_primary_network_validator(&self, node_id: &node::Id) -> Result<bool> {
        let resp =
            client_p::get_primary_network_validators(&self.inner.pick_base_http_url().1).await?;
        let resp = resp
            .result
            .expect("unexpected None GetCurrentValidatorResult");
        let validators = resp.validators.expect("unexpected None vaidators");
        for validator in validators.iter() {
            log::info!("listing primary network validator {}", node_id);
            if validator.node_id.eq(node_id) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Returns "true" if the node_id is a current subnet validator.
    pub async fn is_subnet_validator(
        &self,
        node_id: &node::Id,
        subnet_id: &ids::Id,
    ) -> Result<bool> {
        let resp = client_p::get_subnet_validators(
            &self.inner.pick_base_http_url().1,
            &subnet_id.to_string(),
        )
        .await?;
        let resp = resp
            .result
            .expect("unexpected None GetCurrentValidatorResult");
        let validators = resp.validators.expect("unexpected None vaidators");
        for validator in validators.iter() {
            log::info!(
                "listing subnet validator {} for subnet {}",
                node_id,
                subnet_id
            );
            if validator.node_id.eq(node_id) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.4/vms/platformvm/utxo/handler.go#L169> "Spend"
    /// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.4/wallet/chain/p/builder.go#L325-L358> "NewAddValidatorTx"
    /// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.4/vms/platformvm/txs/builder/builder.go#L428> "NewAddValidatorTx"
    async fn spend(
        &self,
        amount: u64,
        fee: u64,
    ) -> Result<(
        Vec<txs::transferable::Input>,
        Vec<txs::transferable::Output>,
        Vec<txs::transferable::Output>,
        Vec<Vec<T>>,
    )> {
        let utxos = self.utxos().await?;

        let now_unix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("unexpected None duration_since")
            .as_secs();

        let mut ins: Vec<txs::transferable::Input> = Vec::new();
        let mut returned_outputs: Vec<txs::transferable::Output> = Vec::new();
        let mut staked_outputs: Vec<txs::transferable::Output> = Vec::new();
        let mut signers: Vec<Vec<T>> = Vec::new();

        // amount of AVAX that has been staked
        let mut amount_staked: u64 = 0_u64;

        // consume locked UTXOs
        for utxo in utxos.iter() {
            // no need to consume more locked AVAX
            // because it already has consumed more than the target stake amount
            if amount_staked >= amount {
                break;
            }

            // only staking avax so ignore other assets
            if utxo.asset_id != self.inner.avax_asset_id {
                continue;
            }

            // check "*platformvm.StakeableLockOut"
            if utxo.stakeable_lock_out.is_none() {
                // output is not locked, thus handle this in the next iteration
                continue;
            }

            // check locktime
            let out = utxo.stakeable_lock_out.clone().unwrap();
            if out.locktime <= now_unix {
                // output is no longer locked, thus handle in the next iteration
                continue;
            }

            // check "*secp256k1fx.TransferOutput"
            let inner = out.clone().transfer_output;
            let res = self.inner.keychain.spend(&inner, now_unix);
            if res.is_none() {
                // cannot spend the output, move onto next
                continue;
            }
            let (transfer_input, in_signers) = res.unwrap();

            let mut remaining_value = transfer_input.amount;
            let amount_to_stake = cmp::min(
                amount - amount_staked, // amount we still need to stake
                remaining_value,        // amount available to stake
            );
            amount_staked += amount_to_stake;
            remaining_value -= amount_to_stake;

            // add input to the consumed inputs
            ins.push(txs::transferable::Input {
                utxo_id: utxo.utxo_id.clone(),
                asset_id: utxo.asset_id,
                stakeable_lock_in: Some(platformvm::txs::StakeableLockIn {
                    locktime: out.locktime,
                    transfer_input,
                }),
                ..txs::transferable::Input::default()
            });

            // add output to the staked outputs
            staked_outputs.push(txs::transferable::Output {
                asset_id: utxo.asset_id,
                stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
                    locktime: out.clone().locktime,
                    transfer_output: key::secp256k1::txs::transfer::Output {
                        amount: remaining_value,
                        output_owners: out.clone().transfer_output.output_owners,
                    },
                }),
                ..txs::transferable::Output::default()
            });

            if remaining_value > 0 {
                // this input provided more value than was needed to be locked
                // some must be returned
                returned_outputs.push(txs::transferable::Output {
                    asset_id: utxo.asset_id,
                    stakeable_lock_out: Some(platformvm::txs::StakeableLockOut {
                        locktime: out.clone().locktime,
                        transfer_output: key::secp256k1::txs::transfer::Output {
                            amount: amount_to_stake,
                            output_owners: out.clone().transfer_output.output_owners,
                        },
                    }),
                    ..txs::transferable::Output::default()
                });
            }

            signers.push(in_signers);
        }

        // amount of AVAX that has been burned
        let mut amount_burned = 0_u64;

        for utxo in utxos.iter() {
            // have staked/burned more AVAX than we need
            // thus no need to consume more AVAX
            if amount_burned >= fee && amount_staked >= amount {
                break;
            }

            // only burn AVAX, thus ignore other assets
            if utxo.asset_id != self.inner.avax_asset_id {
                continue;
            }

            let (skip, out) = {
                if utxo.transfer_output.is_some() {
                    let out = utxo.transfer_output.clone().unwrap();
                    (false, out)
                } else {
                    let inner = utxo.stakeable_lock_out.clone().unwrap();
                    (inner.locktime > now_unix, inner.transfer_output)
                }
            };
            // output is currently locked, so this output cannot be burned
            // or it may have already been consumed above
            if skip {
                continue;
            }

            let res = self.inner.keychain.spend(&out, now_unix);
            if res.is_none() {
                // cannot spend the output, move onto next
                continue;
            }
            let (transfer_input, in_signers) = res.unwrap();

            // ref. https://github.com/ava-labs/subnet-cli/blob/6bbe9f4aff353b812822af99c08133af35dbc6bd/client/p.go#L763
            let mut remaining_value = transfer_input.amount;
            let amount_to_burn = cmp::min(
                fee - amount_burned, // amount we still need to burn
                remaining_value,     // amount available to burn
            );
            amount_burned += amount_to_burn;
            remaining_value -= amount_to_burn;

            let amount_to_stake = cmp::min(
                amount - amount_staked, // amount we still need to stake
                remaining_value,        // amount available to stake
            );
            amount_staked += amount_to_stake;
            remaining_value -= amount_to_stake;

            // add the input to the consumed inputs
            ins.push(txs::transferable::Input {
                utxo_id: utxo.utxo_id.clone(),
                asset_id: utxo.asset_id,
                transfer_input: Some(transfer_input),
                ..txs::transferable::Input::default()
            });

            if amount_to_stake > 0 {
                // some of this input was put for staking
                staked_outputs.push(txs::transferable::Output {
                    asset_id: utxo.asset_id,
                    transfer_output: Some(key::secp256k1::txs::transfer::Output {
                        amount: amount_to_stake,
                        output_owners: key::secp256k1::txs::OutputOwners {
                            locktime: 0,
                            threshold: 1,
                            addresses: vec![self.inner.short_address.clone()],
                        },
                    }),
                    ..txs::transferable::Output::default()
                });
            }

            if remaining_value > 0 {
                // this input had extra value, so some must be returned
                returned_outputs.push(txs::transferable::Output {
                    asset_id: utxo.asset_id,
                    transfer_output: Some(key::secp256k1::txs::transfer::Output {
                        amount: remaining_value,
                        output_owners: key::secp256k1::txs::OutputOwners {
                            locktime: 0,
                            threshold: 1,
                            addresses: vec![self.inner.short_address.clone()],
                        },
                    }),
                    ..txs::transferable::Output::default()
                });
            }

            signers.push(in_signers);
        }

        log::info!(
            "provided keys have balance (unlocked/burned amount so far, locked/staked amount so far) ({}, {}) and need ({}, {})",
            amount_burned,
            amount_staked,
            fee,
            amount
        );
        if amount_burned < fee || amount_staked < amount {
            return Err(Error::Other {
                message: format!(
                    "provided keys have balance (unlocked/burned amount so far, locked/staked amount so far) ({}, {}) but need ({}, {})",
                    amount_burned,
                    amount_staked,
                    fee,
                    amount
                ),
                retryable: false,
            });
        }

        // TODO: for now just ignore "signers" in the sorting
        // since the wallet currently only supports one key
        ins.sort();
        returned_outputs.sort();
        staked_outputs.sort();

        Ok((ins, returned_outputs, staked_outputs, signers))
    }

    /// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.4/vms/platformvm/utxo/handler.go#L411> "Authorize"
    /// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.4/wallet/chain/p/builder.go#L360-L390> "NewAddSubnetValidatorTx"
    /// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.4/vms/platformvm/txs/builder/builder.go#L512> "NewAddSubnetValidatorTx"
    async fn authorize(
        &self,
        subnet_id: ids::Id,
    ) -> Result<(key::secp256k1::txs::Input, Vec<Vec<T>>)> {
        log::info!("authorizing subnet {}", subnet_id);

        let tx =
            client_p::get_tx(&self.inner.pick_base_http_url().1, &subnet_id.to_string()).await?;
        if let Some(tx_result) = tx.result {
            let output_owners = tx_result.tx.unsigned_tx.output_owners;

            let now_unix = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("unexpected None duration_since")
                .as_secs();

            let res = self
                .inner
                .keychain
                .match_threshold(&output_owners, now_unix);
            let threshold_met = res.is_some();
            if !threshold_met {
                return Err(Error::Other {
                    message: "no threshold met, can't sign".to_string(),
                    retryable: false,
                });
            }
            let (sig_indices, keys) = res.unwrap();

            return Ok((
                key::secp256k1::txs::Input {
                    // if empty, it errors with "unauthorized subnet modification: input has less signers than expected"
                    sig_indices,
                },
                vec![keys],
            ));
        }

        return Err(Error::Other {
            message: "empty get tx result".to_string(),
            retryable: false,
        });
    }

    /// Subnet validators must validate the primary network.
    #[must_use]
    pub fn add_validator(&self) -> add_validator::Tx<T> {
        add_validator::Tx::new(self)
    }

    #[must_use]
    pub fn add_permissionless_validator(&self) -> add_permissionless_validator::Tx<T> {
        add_permissionless_validator::Tx::new(self)
    }

    /// Once subnet is created, the avalanche node must whitelist the subnet Id
    /// (the returned/confirmed transaction Id).
    #[must_use]
    pub fn create_subnet(&self) -> create_subnet::Tx<T> {
        create_subnet::Tx::new(self)
    }

    /// Once the subnet is created, the subnet needs to add new validators
    /// for the subnet itself.
    #[must_use]
    pub fn add_subnet_validator(&self) -> add_subnet_validator::Tx<T> {
        add_subnet_validator::Tx::new(self)
    }

    /// Once the subnet validators are added, each virtual machine must create
    /// its own blockchain and use the chain Id as the RPC endpoint.
    #[must_use]
    pub fn create_chain(&self) -> create_chain::Tx<T> {
        create_chain::Tx::new(self)
    }

    #[must_use]
    pub fn export(&self) -> export::Tx<T> {
        export::Tx::new(self)
    }

    #[must_use]
    pub fn import(&self) -> import::Tx<T> {
        import::Tx::new(self)
    }
}
