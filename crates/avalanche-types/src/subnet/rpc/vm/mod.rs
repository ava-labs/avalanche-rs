//! RPC Chain VM implementation.
pub mod server;

use std::{
    env,
    io::{Error, ErrorKind, Result},
    net::SocketAddr,
    time::Duration,
};

use crate::{
    proto::{
        pb::{
            self,
            vm::vm_server::{Vm as VmImpl, VmServer},
        },
        PROTOCOL_VERSION,
    },
    subnet::rpc::{runtime, utils},
};
use jsonrpc_core::futures::FutureExt;
use tokio::sync::broadcast::Receiver;
use tonic::transport::server::NamedService;
use tonic_health::server::health_reporter;

use super::runtime::Initializer;

/// Health Service for the RPC Chain VM Server.
struct HealthServer;

impl NamedService for HealthServer {
    const NAME: &'static str = "vm server";
}

/// The address of the Runtime server is expected to be passed via ENV `runtime::ENGINE_ADDR_KEY`.
/// This address is used by the Runtime client to send Initialize RPC to server.
///
// Serve starts the RPC Chain VM server and performs a handshake with the VM runtime service.
pub async fn serve<V>(vm: V, stop_ch: Receiver<()>) -> Result<()>
where
    V: VmImpl,
{
    // TODO: Add support for abstract unix sockets once supported by tonic.
    // ref. https://github.com/hyperium/tonic/issues/966
    // avalanchego currently only supports plugins listening on IP address.
    let vm_server_addr = utils::new_socket_addr();

    let runtime_server_addr = env::var(runtime::ENGINE_ADDR_KEY).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!(
                "required environment not found: {}: {e}",
                runtime::ENGINE_ADDR_KEY
            ),
        )
    })?;

    let client_conn = utils::grpc::default_client(&runtime_server_addr)?
        .connect_timeout(Duration::from_secs(runtime::DEFAULT_DIAL_TIMEOUT))
        .connect()
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!(
                    "failed to create runtime client conn from {}: {e}",
                    &runtime_server_addr
                ),
            )
        })?;

    let client = runtime::client::Client::new(client_conn);
    client
        .initialize(PROTOCOL_VERSION, &vm_server_addr.to_string())
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed to initialize runtime: {e}"),
            )
        })?;

    serve_with_address(vm, vm_server_addr, stop_ch).await
}

pub async fn serve_with_address<V>(vm: V, addr: SocketAddr, mut stop_ch: Receiver<()>) -> Result<()>
where
    V: VmImpl,
{
    let (mut health_reporter, health_svc) = health_reporter();
    health_reporter.set_serving::<HealthServer>().await;

    // ref. https://github.com/hyperium/tonic/blob/v0.7.2/examples/src/reflection/server.rs
    // ref. https://docs.rs/prost-types/latest/prost_types/struct.FileDescriptorSet.html
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(pb::rpcdb::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(pb::vm::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(pb::google::protobuf::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(pb::io::prometheus::client::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build()
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed to create gRPC reflection service: {:?}", e),
            )
        })?;

    utils::grpc::default_server()
        .add_service(health_svc)
        .add_service(reflection_service)
        .add_service(VmServer::new(vm))
        .serve_with_shutdown(addr, stop_ch.recv().map(|_| ()))
        .await
        .map_err(|e| Error::new(ErrorKind::Other, format!("grpc server failed: {:?}", e)))?;
    log::info!("grpc server shutdown complete: {}", addr);

    Ok(())
}
