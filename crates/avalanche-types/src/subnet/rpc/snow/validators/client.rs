use std::{
    collections::BTreeMap,
    io::{Error, ErrorKind, Result},
};

use bytes::Bytes;
use tonic::transport::Channel;

use super::*;
use crate::{
    ids,
    proto::{
        google::protobuf::Empty,
        validatorstate::{validator_state_client, GetSubnetIdRequest, GetValidatorSetRequest},
    },
};

#[derive(Clone, Debug)]
pub struct ValidatorStateClient {
    inner: validator_state_client::ValidatorStateClient<Channel>,
}

impl ValidatorStateClient {
    pub fn new(client_conn: Channel) -> Self {
        Self {
            inner: validator_state_client::ValidatorStateClient::new(client_conn),
        }
    }
}

#[tonic::async_trait]
impl super::State for ValidatorStateClient {
    async fn get_minimum_height(&self) -> Result<u64> {
        let mut client = self.inner.clone();
        let resp = client
            .get_minimum_height(Empty {})
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("get_minimum_height failed: {e}")))?
            .into_inner();

        Ok(resp.height)
    }

    async fn get_current_height(&self) -> Result<u64> {
        let mut client = self.inner.clone();
        let resp = client
            .get_current_height(Empty {})
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("get_current_height failed: {e}")))?
            .into_inner();

        Ok(resp.height)
    }

    async fn get_subnet_id(&self, chain_id: crate::ids::Id) -> Result<ids::Id> {
        let mut client = self.inner.clone();
        let resp = client
            .get_subnet_id(GetSubnetIdRequest {
                chain_id: Bytes::from(chain_id.to_vec()),
            })
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("get_subnet_id failed: {e}")))?
            .into_inner();

        Ok(ids::Id::from_slice(&resp.subnet_id))
    }

    async fn get_validator_set(
        &self,
        height: u64,
        subnet_id: crate::ids::Id,
    ) -> std::io::Result<BTreeMap<ids::node::Id, GetValidatorOutput>> {
        let mut client = self.inner.clone();
        let resp = client
            .get_validator_set(GetValidatorSetRequest {
                height,
                subnet_id: Bytes::from(subnet_id.to_vec()),
            })
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("get_validator_set failed: {e}")))?
            .into_inner();

        let mut validators: BTreeMap<ids::node::Id, GetValidatorOutput> = BTreeMap::new();

        for validator in resp.validators.iter() {
            let node_id = ids::node::Id::from_slice(&validator.node_id);

            let mut public_key: Option<Key> = None;
            if !resp.validators.is_empty() {
                public_key = Some(Key::from_bytes(&validator.public_key)?);
            }
            validators.insert(
                node_id,
                GetValidatorOutput {
                    node_id,
                    public_key,
                    weight: validator.weight,
                },
            );
        }

        Ok(validators)
    }
}
