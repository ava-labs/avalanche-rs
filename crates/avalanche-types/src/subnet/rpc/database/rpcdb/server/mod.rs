//! RPC Database Server

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use crate::{
    proto::pb::{
        self,
        google::protobuf::Empty,
        rpcdb::{
            CloseRequest, CloseResponse, CompactRequest, CompactResponse, DeleteRequest,
            DeleteResponse, GetRequest, GetResponse, HasRequest, HasResponse, HealthCheckResponse,
            IteratorErrorRequest, IteratorErrorResponse, IteratorNextRequest, IteratorNextResponse,
            IteratorReleaseRequest, IteratorReleaseResponse, NewIteratorWithStartAndPrefixRequest,
            NewIteratorWithStartAndPrefixResponse, PutRequest, PutResponse, WriteBatchRequest,
            WriteBatchResponse,
        },
    },
    subnet::rpc::database::{
        iterator::BoxedIterator, rpcdb::error_to_error_code, BoxedDatabase, MAX_BATCH_SIZE,
    },
};

use prost::bytes::Bytes;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use zerocopy::AsBytes;

/// Serves a [`crate::subnet::rpc::database::Database`] over over RPC.
pub struct Server {
    inner: Arc<RwLock<BoxedDatabase>>,
    iterators: Arc<RwLock<HashMap<u64, BoxedIterator>>>,
    next_iterator_id: AtomicU64,
}

impl Server {
    pub fn new(db: BoxedDatabase) -> impl pb::rpcdb::database_server::Database {
        Self {
            inner: Arc::new(RwLock::new(db)),
            iterators: Arc::new(RwLock::new(HashMap::new())),
            next_iterator_id: AtomicU64::new(0),
        }
    }
}
#[tonic::async_trait]
impl pb::rpcdb::database_server::Database for Server {
    async fn has(&self, request: Request<HasRequest>) -> Result<Response<HasResponse>, Status> {
        let req = request.into_inner();
        let db = self.inner.read().await;

        match db.has(req.key.as_bytes()).await {
            Ok(has) => Ok(Response::new(HasResponse {
                has,
                err: pb::rpcdb::Error::Unspecified.into(),
            })),
            Err(e) => Ok(Response::new(HasResponse {
                has: false,
                err: error_to_error_code(&e.to_string()).unwrap(),
            })),
        }
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let req = request.into_inner();
        let db = self.inner.read().await;

        match db.get(req.key.as_bytes()).await {
            Ok(resp) => Ok(Response::new(GetResponse {
                value: Bytes::from(resp),
                err: pb::rpcdb::Error::Unspecified.into(),
            })),
            Err(e) => Ok(Response::new(GetResponse {
                value: Bytes::from(""),
                err: error_to_error_code(&e.to_string()).unwrap(),
            })),
        }
    }

    async fn put(&self, request: Request<PutRequest>) -> Result<Response<PutResponse>, Status> {
        let req = request.into_inner();
        let mut db = self.inner.write().await;

        match db.put(req.key.as_bytes(), req.value.as_bytes()).await {
            Ok(_) => Ok(Response::new(PutResponse {
                err: pb::rpcdb::Error::Unspecified.into(),
            })),
            Err(e) => Ok(Response::new(PutResponse {
                err: error_to_error_code(&e.to_string()).unwrap(),
            })),
        }
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let req = request.into_inner();
        let mut db = self.inner.write().await;

        match db.delete(req.key.as_bytes()).await {
            Ok(_) => Ok(Response::new(DeleteResponse {
                err: pb::rpcdb::Error::Unspecified.into(),
            })),
            Err(e) => Ok(Response::new(DeleteResponse {
                err: error_to_error_code(&e.to_string()).unwrap(),
            })),
        }
    }

    async fn compact(
        &self,
        _request: Request<CompactRequest>,
    ) -> Result<Response<CompactResponse>, Status> {
        Err(Status::unimplemented("compact"))
    }

    async fn close(
        &self,
        _request: Request<CloseRequest>,
    ) -> Result<Response<CloseResponse>, Status> {
        let db = self.inner.read().await;

        match db.close().await {
            Ok(_) => Ok(Response::new(CloseResponse {
                err: pb::rpcdb::Error::Unspecified.into(),
            })),
            Err(e) => Ok(Response::new(CloseResponse {
                err: error_to_error_code(&e.to_string()).unwrap(),
            })),
        }
    }

    async fn health_check(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let db = self.inner.read().await;

        match db.health_check().await {
            Ok(health) => match serde_json::to_string(&health) {
                Ok(details) => Ok(Response::new(HealthCheckResponse {
                    details: Bytes::from(details),
                })),
                Err(e) => Err(tonic::Status::unknown(e.to_string())),
            },
            Err(e) => Err(tonic::Status::unknown(e.to_string())),
        }
    }

    async fn write_batch(
        &self,
        request: Request<WriteBatchRequest>,
    ) -> Result<Response<WriteBatchResponse>, Status> {
        let req = request.into_inner();
        let db = self.inner.read().await;

        let mut batch = db.new_batch().await?;
        for put in req.puts.iter() {
            let resp = batch.put(&put.key, &put.value).await;
            if let Err(e) = resp {
                return Ok(Response::new(WriteBatchResponse {
                    err: error_to_error_code(&e.to_string()).unwrap(),
                }));
            }
        }

        for del in req.deletes.iter() {
            let resp = batch.delete(&del.key).await;
            if let Err(e) = resp {
                return Ok(Response::new(WriteBatchResponse {
                    err: error_to_error_code(&e.to_string()).unwrap(),
                }));
            }
        }

        match batch.write().await {
            Ok(_) => {
                return Ok(Response::new(WriteBatchResponse {
                    err: pb::rpcdb::Error::Unspecified.into(),
                }))
            }
            Err(e) => {
                return Ok(Response::new(WriteBatchResponse {
                    err: error_to_error_code(&e.to_string()).unwrap(),
                }))
            }
        }
    }

    async fn new_iterator_with_start_and_prefix(
        &self,
        req: Request<NewIteratorWithStartAndPrefixRequest>,
    ) -> Result<Response<NewIteratorWithStartAndPrefixResponse>, Status> {
        let req = req.into_inner();
        let db = self.inner.read().await;
        let it = db
            .new_iterator_with_start_and_prefix(&req.start, &req.prefix)
            .await?;

        let mut iterators = self.iterators.write().await;
        let id = self.next_iterator_id.fetch_add(1, Ordering::SeqCst);
        iterators.insert(id, it);

        Ok(Response::new(NewIteratorWithStartAndPrefixResponse { id }))
    }

    async fn iterator_next(
        &self,
        request: Request<IteratorNextRequest>,
    ) -> Result<Response<IteratorNextResponse>, Status> {
        let req = request.into_inner();

        let mut iterators = self.iterators.write().await;
        if let Some(it) = iterators.get_mut(&req.id) {
            let mut size: usize = 0;
            let mut data: Vec<PutRequest> = Vec::new();

            while (size < MAX_BATCH_SIZE) && it.next().await? {
                let key = it.key().await?.to_owned();
                let value = it.value().await?.to_owned();
                size += key.len() + value.len();

                data.push(PutRequest {
                    key: Bytes::from(key),
                    value: Bytes::from(value),
                });
            }

            return Ok(Response::new(IteratorNextResponse { data }));
        }

        Err(tonic::Status::unknown("unknown iterator"))
    }

    async fn iterator_error(
        &self,
        request: Request<IteratorErrorRequest>,
    ) -> Result<Response<IteratorErrorResponse>, Status> {
        let req = request.into_inner();

        let mut iterators = self.iterators.write().await;
        if let Some(it) = iterators.get_mut(&req.id) {
            match it.error().await {
                Ok(_) => {
                    return Ok(Response::new(IteratorErrorResponse {
                        err: pb::rpcdb::Error::Unspecified.into(),
                    }))
                }
                Err(e) => {
                    return Ok(Response::new(IteratorErrorResponse {
                        err: error_to_error_code(&e.to_string()).unwrap(),
                    }))
                }
            }
        }

        Err(tonic::Status::unknown("unknown iterator"))
    }

    async fn iterator_release(
        &self,
        request: Request<IteratorReleaseRequest>,
    ) -> Result<Response<IteratorReleaseResponse>, Status> {
        let req = request.into_inner();

        let mut iterators = self.iterators.write().await;
        if let Some(it) = iterators.get_mut(&req.id) {
            match it.error().await {
                Ok(_) => {
                    let _ = it.release().await;
                    return Ok(Response::new(IteratorReleaseResponse {
                        err: pb::rpcdb::Error::Unspecified.into(),
                    }));
                }
                Err(e) => {
                    return Ok(Response::new(IteratorReleaseResponse {
                        err: error_to_error_code(&e.to_string()).unwrap(),
                    }))
                }
            }
        }

        Err(tonic::Status::unknown("unknown iterator"))
    }
}
