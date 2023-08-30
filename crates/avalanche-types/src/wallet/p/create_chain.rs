use crate::{
    errors::{Error, Result},
    formatting, ids,
    jsonrpc::client::p as client_p,
    key, platformvm, txs,
};
use tokio::time::{sleep, Duration, Instant};

/// Represents P-chain "CreateChain" transaction.
/// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.4/wallet/chain/p/builder.go#L459-L498> "NewCreateChainTx"
/// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.4/vms/platformvm/txs/builder/builder.go#L345> "NewCreateChainTx"
#[derive(Clone, Debug)]
pub struct Tx<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone,
{
    pub inner: crate::wallet::p::P<T>,

    pub subnet_id: ids::Id,
    pub genesis_data: Vec<u8>,
    pub vm_id: ids::Id,
    pub chain_name: String,

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
        Self {
            inner: p.clone(),
            subnet_id: ids::Id::empty(),
            genesis_data: Vec::new(),
            vm_id: ids::Id::empty(),
            chain_name: String::new(),
            check_acceptance: false,
            poll_initial_wait: Duration::from_millis(1500),
            poll_interval: Duration::from_secs(1),
            poll_timeout: Duration::from_secs(300),
            dry_mode: false,
        }
    }

    /// Sets the subnet Id.
    #[must_use]
    pub fn subnet_id(mut self, subnet_id: ids::Id) -> Self {
        self.subnet_id = subnet_id;
        self
    }

    /// Sets the genesis.
    #[must_use]
    pub fn genesis_data(mut self, genesis_data: Vec<u8>) -> Self {
        self.genesis_data = genesis_data;
        self
    }

    /// Sets the Vm Id.
    #[must_use]
    pub fn vm_id(mut self, vm_id: ids::Id) -> Self {
        self.vm_id = vm_id;
        self
    }

    /// Sets the chain name.
    #[must_use]
    pub fn chain_name(mut self, chain_name: String) -> Self {
        self.chain_name = chain_name;
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

    /// Issues the create chain transaction and returns the transaction Id.
    pub async fn issue(&self) -> Result<ids::Id> {
        let picked_http_rpc = self.inner.inner.pick_base_http_url();
        log::info!(
            "creating a new chain for subnet {}, vm id {}, chain name {}, via {}",
            self.subnet_id,
            self.vm_id,
            self.chain_name,
            picked_http_rpc.1
        );

        let (ins, unstaked_outs, _, signers) = self
            .inner
            .spend(0, self.inner.inner.create_blockchain_tx_fee)
            .await?;
        let (subnet_auth, subnet_signers) = self.inner.authorize(self.subnet_id).await?;

        let mut tx = platformvm::txs::create_chain::Tx {
            base_tx: txs::Tx {
                network_id: self.inner.inner.network_id,
                blockchain_id: self.inner.inner.blockchain_id_p,
                transferable_outputs: Some(unstaked_outs),
                transferable_inputs: Some(ins),
                ..Default::default()
            },
            subnet_id: self.subnet_id,
            chain_name: self.chain_name.clone(),
            vm_id: self.vm_id,
            genesis_data: self.genesis_data.clone(),
            subnet_auth,
            ..Default::default()
        };
        tx.sign(vec![signers, subnet_signers].concat()).await?;

        if self.dry_mode {
            return Ok(tx.base_tx.metadata.unwrap().id);
        }

        let tx_bytes_with_signatures = tx.base_tx.metadata.unwrap().tx_bytes_with_signatures;
        let hex_tx = formatting::encode_hex_with_checksum(&tx_bytes_with_signatures);
        let resp = client_p::issue_tx(&picked_http_rpc.1, &hex_tx).await?;

        if let Some(e) = resp.error {
            return Err(Error::API {
                message: format!("failed to issue create chain transaction {:?}", e),
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

        log::info!("polling to confirm create chain transaction");
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

        Ok(tx_id)
    }
}
