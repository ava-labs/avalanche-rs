use std::io::{Error, ErrorKind, Read, Result};
use std::str::FromStr;

use crate::proto::pb::warp::{
        signer_client,
        SignRequest,
        SignResponse,
    };
use prost::bytes::Bytes;
use tonic::transport::Channel;
use crate::ids::Id;

#[derive(Clone)]
pub struct WarpSignerClient {
    inner: signer_client::SignerClient<Channel>,
}

impl WarpSignerClient {
    pub fn new(client_conn: Channel) -> Self {
        Self {
            inner: signer_client::SignerClient::new(client_conn)
                .max_decoding_message_size(usize::MAX)
                .max_encoding_message_size(usize::MAX),
        }
    }
}

#[tonic::async_trait]
impl super::WarpSignerClient_ for WarpSignerClient {
    async fn sign(&self,
                  network_id: u32,
                  source_chain_id: &str,
                  payload: &[u8]) -> Result<SignResponse> {
        let mut client = self.inner.clone();
        let res = client
            .sign(SignRequest {
                network_id,
                source_chain_id: Bytes::from(Id::from_str(source_chain_id).unwrap().to_vec()),
                payload: Bytes::from(payload.to_vec()),
            })
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("sign failed: {:?}", e),
                )
            })?;
        Ok(res.into_inner())
    }
}
