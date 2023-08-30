use std::time::SystemTime;

use crate::{
    avm,
    choices::status::Status,
    errors::{Error, Result},
    formatting, ids,
    jsonrpc::client::x as client_x,
    key, txs,
};
use tokio::time::{sleep, Duration, Instant};

/// Represents X-chain "Import" transaction.
/// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.4/wallet/chain/x/builder.go> "NewImportTx".
#[derive(Clone, Debug)]
pub struct Tx<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
{
    pub inner: crate::wallet::x::X<T>,

    /// Import source blockchain id.
    pub source_blockchain_id: ids::Id,

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
            source_blockchain_id: ids::Id::empty(),
            check_acceptance: false,
            poll_initial_wait: Duration::from_millis(500),
            poll_interval: Duration::from_millis(700),
            poll_timeout: Duration::from_secs(300),
            dry_mode: false,
        }
    }

    /// Sets the source blockchain Id.
    #[must_use]
    pub fn source_blockchain_id(mut self, blockchain_id: ids::Id) -> Self {
        self.source_blockchain_id = blockchain_id;
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

    /// Issues the import transaction and returns the transaction Id.
    pub async fn issue(&self) -> Result<ids::Id> {
        let picked_http_rpc = self.inner.inner.pick_base_http_url();
        log::info!(
            "importing from {} via {}",
            self.source_blockchain_id,
            picked_http_rpc.1
        );

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

        // ref. "avalanchego/vms/avm#Service.SendMultiple"
        let now_unix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("unexpected None duration_since")
            .as_secs();

        let mut import_amount = 0u64;
        let mut import_inputs: Vec<txs::transferable::Input> = Vec::new();
        let mut signers: Vec<Vec<T>> = Vec::new();

        for utxo in utxos.iter() {
            if utxo.asset_id != self.inner.inner.avax_asset_id {
                continue;
            }

            if let Some(out) = &utxo.transfer_output {
                let res = self.inner.inner.keychain.spend(out, now_unix);
                if res.is_none() {
                    // cannot spend the output, move onto next
                    continue;
                }
                let (transfer_input, in_signers) = res.unwrap();

                // TODO: check overflow
                import_amount += transfer_input.amount;

                // add input to the consumed inputs
                import_inputs.push(txs::transferable::Input {
                    utxo_id: utxo.utxo_id.clone(),
                    asset_id: utxo.asset_id,
                    transfer_input: Some(transfer_input),
                    ..txs::transferable::Input::default()
                });
                signers.push(in_signers);
            }
        }

        if import_inputs.is_empty() {
            return Err(Error::Other {
                message: "no spendable funds were found".to_string(),
                retryable: false,
            });
        }

        // TODO: check import amount with tx fee
        log::info!(
            "importing total {} AVAX with tx fee {}",
            import_amount,
            self.inner.inner.tx_fee
        );
        import_amount -= self.inner.inner.tx_fee;

        let outputs: Vec<txs::transferable::Output> = vec![
            // receiver
            txs::transferable::Output {
                asset_id: self.inner.inner.avax_asset_id.clone(),
                transfer_output: Some(key::secp256k1::txs::transfer::Output {
                    amount: import_amount,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: 0,
                        threshold: 1,
                        addresses: vec![self.inner.inner.short_address.clone()],
                    },
                }),
                ..Default::default()
            },
        ];

        log::debug!(
            "baseTx has {} inputs and {} outputs",
            import_inputs.len(),
            outputs.len()
        );
        let mut tx = avm::txs::import::Tx {
            base_tx: txs::Tx {
                network_id: self.inner.inner.network_id,
                blockchain_id: self.inner.inner.blockchain_id_p.clone(),
                transferable_outputs: Some(outputs),
                ..Default::default()
            },
            source_chain_id: self.source_blockchain_id.clone(),
            source_chain_transferable_inputs: Some(import_inputs),
            ..Default::default()
        };
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
                message: format!("failed to issue import tx {:?} (no result)", resp.error),
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
