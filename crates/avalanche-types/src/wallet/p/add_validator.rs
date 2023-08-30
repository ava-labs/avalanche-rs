use std::time::SystemTime;

use crate::{
    errors::{Error, Result},
    formatting,
    ids::{self, node},
    jsonrpc::client::p as client_p,
    key, platformvm, txs, units,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use tokio::time::{sleep, Duration, Instant};

/// Represents P-chain "AddValidator" transaction.
/// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.4/wallet/chain/p/builder.go#L325-L358> "NewAddValidatorTx"
/// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.4/vms/platformvm/txs/builder/builder.go#L428> "NewAddValidatorTx"
#[derive(Clone, Debug)]
pub struct Tx<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
{
    pub inner: crate::wallet::p::P<T>,

    pub node_id: node::Id,

    /// Denominated in nano-AVAX.
    /// On the X-Chain, one AVAX is 10^9  units.
    /// On the P-Chain, one AVAX is 10^9  units.
    /// On the C-Chain, one AVAX is 10^18 units.
    /// ref. <https://snowtrace.io/unitconverter>
    pub stake_amount: u64,

    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,

    /// Validate reward fee in percent.
    pub reward_fee_percent: u32,

    /// Set "true" to poll transaction status after issuance for its acceptance.
    pub check_acceptance: bool,

    /// Initial wait duration before polling for acceptance.
    pub poll_initial_wait: Duration,
    /// Wait between each poll intervals for acceptance.
    pub poll_interval: Duration,
    /// Maximum duration for polling.
    pub poll_timeout: Duration,

    /// Set to true to return transaction Id for "issue" in dry mode.
    pub dry_mode: bool,
}

impl<T> Tx<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
{
    pub fn new(p: &crate::wallet::p::P<T>) -> Self {
        let now_unix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("unexpected None duration_since")
            .as_secs();

        let start_time = now_unix + 60;
        let native_dt = NaiveDateTime::from_timestamp_opt(start_time as i64, 0).unwrap();
        let start_time = DateTime::<Utc>::from_utc(native_dt, Utc);

        // 15-day + 5-min
        // must be smaller than the primary network default
        // otherwise "staking period must be a subset of the primary network"
        // 15 that subnet validator defaults can be bounded within
        // ref. "Validator.BoundedBy"
        let end_time = now_unix + 14 * 24 * 60 * 60 + 5 * 60;
        let native_dt = NaiveDateTime::from_timestamp_opt(end_time as i64, 0).unwrap();
        let end_time = DateTime::<Utc>::from_utc(native_dt, Utc);

        Self {
            inner: p.clone(),
            node_id: node::Id::empty(),
            stake_amount: 2 * units::KILO_AVAX,
            start_time,
            end_time,
            reward_fee_percent: 2,
            check_acceptance: false,
            poll_initial_wait: Duration::from_secs(62), // enough to elapse validate start time
            poll_interval: Duration::from_secs(1),
            poll_timeout: Duration::from_secs(300),
            dry_mode: false,
        }
    }

    /// Sets the validator node Id.
    #[must_use]
    pub fn node_id(mut self, node_id: node::Id) -> Self {
        self.node_id = node_id;
        self
    }

    /// Sets the stake amount.
    #[must_use]
    pub fn stake_amount(mut self, stake_amount: u64) -> Self {
        self.stake_amount = stake_amount;
        self
    }

    /// Sets the validate start time.
    #[must_use]
    pub fn start_time(mut self, start_time: DateTime<Utc>) -> Self {
        self.start_time = start_time;
        self
    }

    /// Sets the validate start time.
    #[must_use]
    pub fn end_time(mut self, end_time: DateTime<Utc>) -> Self {
        self.end_time = end_time;
        self
    }

    /// Sets the validate start/end time in days from 'offset_seconds' later.
    #[must_use]
    pub fn validate_period_in_days(mut self, days: u64, offset_seconds: u64) -> Self {
        let now_unix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("unexpected None duration_since")
            .as_secs();

        let start_time = now_unix + offset_seconds;
        let native_dt = NaiveDateTime::from_timestamp_opt(start_time as i64, 0).unwrap();
        let start_time = DateTime::<Utc>::from_utc(native_dt, Utc);

        // must be smaller than the primary network default
        // otherwise "staking period must be a subset of the primary network"
        let end_time = now_unix + days * 24 * 60 * 60;
        let native_dt = NaiveDateTime::from_timestamp_opt(end_time as i64, 0).unwrap();
        let end_time = DateTime::<Utc>::from_utc(native_dt, Utc);

        self.start_time = start_time;
        self.end_time = end_time;
        self
    }

    /// Sets the validate reward in percent.
    #[must_use]
    pub fn reward_fee_percent(mut self, reward_fee_percent: u32) -> Self {
        self.reward_fee_percent = reward_fee_percent;
        self
    }

    /// Sets the check acceptance boolean flag.
    #[must_use]
    pub fn check_acceptance(mut self, check_acceptance: bool) -> Self {
        self.check_acceptance = check_acceptance;
        self
    }

    /// Sets the initial poll wait time.
    #[must_use]
    pub fn poll_initial_wait(mut self, poll_initial_wait: Duration) -> Self {
        self.poll_initial_wait = poll_initial_wait;
        self
    }

    /// Sets the poll wait time between intervals.
    #[must_use]
    pub fn poll_interval(mut self, poll_interval: Duration) -> Self {
        self.poll_interval = poll_interval;
        self
    }

    /// Sets the poll timeout.
    #[must_use]
    pub fn poll_timeout(mut self, poll_timeout: Duration) -> Self {
        self.poll_timeout = poll_timeout;
        self
    }

    /// Sets the dry mode boolean flag.
    #[must_use]
    pub fn dry_mode(mut self, dry_mode: bool) -> Self {
        self.dry_mode = dry_mode;
        self
    }

    /// Issues the add validator transaction and returns the transaction Id.
    /// The boolean return represents whether the "add_validator" request was
    /// successfully issued or not (regardless of its acceptance).
    /// If the validator is already a validator, it returns an empty Id and false.
    pub async fn issue(&self) -> Result<(ids::Id, bool)> {
        let picked_http_rpc = self.inner.inner.pick_base_http_url();
        log::info!(
            "adding primary network validator {} with stake amount {} AVAX ({} nAVAX) via {}",
            self.node_id,
            units::cast_xp_navax_to_avax(primitive_types::U256::from(self.stake_amount)),
            self.stake_amount,
            picked_http_rpc.1
        );

        let already_validator = self
            .inner
            .is_primary_network_validator(&self.node_id)
            .await?;
        if already_validator {
            log::warn!(
                "node Id {} is already a validator -- returning empty tx Id",
                self.node_id
            );
            return Ok((ids::Id::empty(), false));
        }

        let cur_balance_p = self.inner.balance().await?;
        if cur_balance_p < self.stake_amount + self.inner.inner.add_primary_network_validator_fee {
            return Err(Error::Other {
                message: format!("key address {} (balance {} nano-AVAX, network {}) does not have enough to cover stake amount + fee {}", self.inner.inner.p_address, cur_balance_p, self.inner.inner.network_name, self.stake_amount + self.inner.inner.add_primary_network_validator_fee),
                retryable: false,
            });
        };
        log::info!(
            "{} current P-chain balance {}",
            self.inner.inner.p_address,
            cur_balance_p
        );

        let (ins, unstaked_outs, staked_outs, signers) = self
            .inner
            .spend(
                self.stake_amount,
                self.inner.inner.add_primary_network_validator_fee,
            )
            .await?;

        let mut tx = platformvm::txs::add_validator::Tx {
            base_tx: txs::Tx {
                network_id: self.inner.inner.network_id,
                blockchain_id: self.inner.inner.blockchain_id_p,
                transferable_outputs: Some(unstaked_outs),
                transferable_inputs: Some(ins),
                ..Default::default()
            },
            validator: platformvm::txs::Validator {
                node_id: self.node_id.clone(),
                start: self.start_time.timestamp() as u64,
                end: self.end_time.timestamp() as u64,
                weight: self.stake_amount,
            },
            stake_transferable_outputs: Some(staked_outs),
            rewards_owner: key::secp256k1::txs::OutputOwners {
                locktime: 0,
                threshold: 1,
                addresses: vec![self.inner.inner.short_address.clone()],
            },
            shares: self.reward_fee_percent * 10000,
            ..Default::default()
        };
        tx.sign(signers).await?;

        if self.dry_mode {
            return Ok((tx.base_tx.metadata.unwrap().id, false));
        }

        let tx_bytes_with_signatures = tx.base_tx.metadata.unwrap().tx_bytes_with_signatures;
        let hex_tx = formatting::encode_hex_with_checksum(&tx_bytes_with_signatures);
        let resp = client_p::issue_tx(&picked_http_rpc.1, &hex_tx).await?;

        if let Some(e) = resp.error {
            // handle duplicate validator
            // ref. "avalanchego/vms/platformvm/txs/executor" "verifyAddValidatorTx"
            let already_validator = e
                .message
                .contains("attempted to issue duplicate validation for");
            if already_validator {
                log::warn!(
                    "node Id {} is already a validator -- returning empty tx Id ({})",
                    self.node_id,
                    e.message
                );
                return Ok((ids::Id::empty(), false));
            }

            return Err(Error::API {
                message: format!("failed to issue add validator transaction {:?}", e),
                retryable: false,
            });
        }

        let tx_id = resp.result.unwrap().tx_id;
        log::info!("{} successfully issued", tx_id);

        if !self.check_acceptance {
            log::debug!("skipping checking acceptance...");
            return Ok((tx_id, true));
        }

        // enough time for txs processing
        log::info!("initial waiting {:?}", self.poll_initial_wait);
        sleep(self.poll_initial_wait).await;

        log::info!("polling to confirm add validator transaction");
        let (start, mut success) = (Instant::now(), false);
        loop {
            let elapsed = start.elapsed();
            if elapsed.gt(&self.poll_timeout) {
                break;
            }

            let resp = client_p::get_tx_status(&picked_http_rpc.1, &tx_id.to_string()).await?;

            let status = resp.result.unwrap().status;
            if status == platformvm::txs::status::Status::Committed {
                log::info!("{} successfully committed", tx_id);
                success = true;
                break;
            }

            log::warn!(
                "{} {} (not accepted yet in {}, elapsed {:?})",
                tx_id,
                status,
                picked_http_rpc.1,
                elapsed
            );
            sleep(self.poll_interval).await;
        }
        if !success {
            return Err(Error::API {
                message: "failed to check acceptance in time".to_string(),
                retryable: true,
            });
        }

        log::info!("polling to confirm validator");
        success = false;
        loop {
            let elapsed = start.elapsed();
            if elapsed.gt(&self.poll_timeout) {
                break;
            }

            let already_validator = self
                .inner
                .is_primary_network_validator(&self.node_id)
                .await?;
            if already_validator {
                log::info!("node Id {} is now a validator", self.node_id);
                success = true;
                break;
            }

            log::warn!(
                "node Id {} is not a validator yet (elapsed {:?})",
                self.node_id,
                elapsed
            );
            sleep(self.poll_interval).await;
        }
        if !success {
            return Err(Error::API {
                message: "failed to check validator acceptance in time".to_string(),
                retryable: true,
            });
        }

        Ok((tx_id, true))
    }
}
