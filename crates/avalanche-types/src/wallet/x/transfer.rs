use std::{cmp, time::SystemTime};

use crate::{
    avm,
    choices::status::Status,
    errors::{Error, Result},
    formatting,
    ids::{self, short},
    jsonrpc::client::x as client_x,
    key, txs,
};
use tokio::time::{sleep, Duration, Instant};

#[derive(Clone, Debug)]
pub struct Tx<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
{
    pub inner: crate::wallet::x::X<T>,

    /// Transfer fund receiver address.
    pub receiver: short::Id,

    /// Transfer amount.
    pub amount: u64,

    /// Set "true" to poll transfer status after issuance for its acceptance.
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
    pub fn new(x: &crate::wallet::x::X<T>) -> Self {
        Self {
            inner: x.clone(),
            receiver: short::Id::empty(),
            amount: 0,
            check_acceptance: false,
            poll_initial_wait: Duration::from_millis(500),
            poll_interval: Duration::from_millis(700),
            poll_timeout: Duration::from_secs(300),
            dry_mode: false,
        }
    }

    /// Sets the transfer fund receiver address.
    #[must_use]
    pub fn receiver(mut self, receiver: short::Id) -> Self {
        self.receiver = receiver;
        self
    }

    /// Sets the transfer amount.
    #[must_use]
    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
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

    /// Issues the transfer transaction and returns the transaction Id.
    pub async fn issue(&self) -> Result<ids::Id> {
        let picked_http_rpc = self.inner.inner.pick_base_http_url();
        log::info!(
            "transferring {} AVAX from {} to {} via {}",
            self.amount,
            self.inner.inner.short_address,
            self.receiver,
            picked_http_rpc.1
        );

        // ref. https://github.com/ava-labs/avalanchego/blob/v1.7.9/wallet/chain/p/builder.go
        // ref. https://github.com/ava-labs/avalanchego/blob/v1.7.9/vms/platformvm/add_validator_tx.go#L263
        // ref. https://github.com/ava-labs/avalanchego/blob/v1.7.9/vms/platformvm/spend.go#L39 "stake"
        // ref. https://github.com/ava-labs/subnet-cli/blob/6bbe9f4aff353b812822af99c08133af35dbc6bd/client/p.go#L355 "AddValidator"
        // ref. https://github.com/ava-labs/subnet-cli/blob/6bbe9f4aff353b812822af99c08133af35dbc6bd/client/p.go#L614 "stake"
        // TODO: paginate next results
        let utxos = client_x::get_utxos(&picked_http_rpc.1, &self.inner.inner.x_address).await?;
        let utxos_result = utxos.result.unwrap();
        let utxos = utxos_result.utxos.unwrap();
        log::debug!(
            "fetched UTXOs for inputs: numFetched {:?}, endIndex {:?} and {} UTXOs",
            utxos_result.num_fetched,
            utxos_result.end_index,
            utxos.len()
        );

        let mut inputs: Vec<txs::transferable::Input> = Vec::new();
        let mut outputs: Vec<txs::transferable::Output> = vec![
            // receiver
            txs::transferable::Output {
                asset_id: self.inner.inner.avax_asset_id.clone(),
                transfer_output: Some(key::secp256k1::txs::transfer::Output {
                    amount: self.amount,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: 0,
                        threshold: 1,
                        addresses: vec![self.receiver.clone()],
                    },
                }),
                ..Default::default()
            },
        ];

        // ref. "avalanchego/wallet/chain/x"
        // "math.Add64(toBurn[assetID], out.Out.Amount())"
        let mut remaining_amount_to_burn = self.amount + self.inner.inner.tx_fee;

        // ref. "avalanchego/vms/avm#Service.SendMultiple"
        let now_unix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("unexpected None duration_since")
            .as_secs();

        for utxo in utxos.iter() {
            if utxo.asset_id != self.inner.inner.avax_asset_id {
                continue;
            }

            // consumed enough, no need to burn more
            if remaining_amount_to_burn == 0 {
                continue;
            }

            if let Some(out) = &utxo.transfer_output {
                let (input, _) = self.inner.inner.keychain.spend(out, now_unix).unwrap();

                inputs.push(txs::transferable::Input {
                    utxo_id: utxo.utxo_id.clone(),
                    asset_id: utxo.asset_id.clone(),
                    transfer_input: Some(input),
                    ..Default::default()
                });

                // burn any value that should be burned
                let amount_to_burn = cmp::min(
                    remaining_amount_to_burn, // amount we still need to burn
                    out.amount,               // amount available to burn
                );
                remaining_amount_to_burn -= amount_to_burn;

                let remaining_amount = out.amount - amount_to_burn;
                if remaining_amount > 0 {
                    // this input had extra value, so some must be returned
                    outputs.push(txs::transferable::Output {
                        asset_id: self.inner.inner.avax_asset_id.clone(),
                        transfer_output: Some(key::secp256k1::txs::transfer::Output {
                            amount: remaining_amount,
                            output_owners: key::secp256k1::txs::OutputOwners {
                                locktime: 0,
                                threshold: 1,
                                addresses: vec![self.inner.inner.short_address.clone()],
                            },
                        }),
                        ..Default::default()
                    })
                }
            }
        }
        inputs.sort();
        outputs.sort();

        // make sure it does not incur "tx has 1 credentials but 2 inputs. Should be same" error
        let mut signers: Vec<Vec<T>> = Vec::new();
        for _ in 0..inputs.len() {
            signers.push(vec![self.inner.inner.keychain.keys[0].clone()]);
        }
        if inputs.len() > 1 {
            log::debug!("signing for multiple inputs ({} inputs)", inputs.len());
        }

        log::debug!(
            "baseTx has {} inputs and {} outputs",
            inputs.len(),
            outputs.len()
        );
        let mut tx = avm::txs::Tx::new(txs::Tx {
            network_id: self.inner.inner.network_id,
            blockchain_id: self.inner.inner.blockchain_id_x.clone(),
            transferable_outputs: Some(outputs),
            transferable_inputs: Some(inputs.clone()),
            ..Default::default()
        });
        tx.sign(signers).await?;

        if self.dry_mode {
            return Ok(tx.base_tx.metadata.unwrap().id);
        }

        let tx_bytes_with_signatures = tx
            .base_tx
            .metadata
            .clone()
            .unwrap()
            .tx_bytes_with_signatures;
        let hex_tx = formatting::encode_hex_with_checksum(&tx_bytes_with_signatures);
        let resp = client_x::issue_tx(&picked_http_rpc.1, &hex_tx).await?;

        if resp.result.is_none() {
            return Err(Error::API {
                message: format!("failed to issue tx {:?} (no result)", resp.error),
                retryable: false,
            });
        }

        let tx_id = resp.result.unwrap().tx_id;
        log::info!("{} successfully issued", tx_id);

        if !self.check_acceptance {
            log::debug!("skipping checking acceptance...");
            return Ok(tx_id);
        }

        // enough time for txs processing
        log::info!("initial waiting {:?}", self.poll_initial_wait);
        sleep(self.poll_initial_wait).await;

        log::info!("polling to confirm base transaction");
        let (start, mut success) = (Instant::now(), false);
        loop {
            let elapsed = start.elapsed();
            if elapsed.gt(&self.poll_timeout) {
                break;
            }

            let resp = client_x::get_tx_status(&picked_http_rpc.1, &tx_id.to_string()).await?;

            let status = resp.result.unwrap().status;
            if status == Status::Accepted {
                log::info!("{} successfully accepted", tx_id);
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

        Ok(tx_id)
    }
}
