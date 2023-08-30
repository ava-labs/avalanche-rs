use std::{
    io::{Error, ErrorKind, Result},
    net::SocketAddr,
};

use crate::{
    proto::{
        grpcutil,
        pb::{
            self,
            vm::vm_server::{Vm, VmServer},
        },
        PROTOCOL_VERSION,
    },
    subnet::rpc::utils,
};
use jsonrpc_core::futures::FutureExt;
use tokio::sync::broadcast::Receiver;
use tonic::transport::server::NamedService;
use tonic_health::server::health_reporter;

/// ref. <https://github.com/ava-labs/avalanchego/blob/v1.9.5/version/constants.go#L15-L17>
struct HandshakeConfig {
    protocol_version: &'static str,
}

impl HandshakeConfig {
    pub fn new() -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
        }
    }
}

struct Plugin;

impl NamedService for Plugin {
    const NAME: &'static str = "plugin";
}

/// serve starts a gRPC server which serves the Vm service and generates the handshake message for plugin support.
/// Reflection is enabled by default.
pub async fn serve<V>(vm: V, stop_ch: Receiver<()>) -> Result<()>
where
    V: Vm,
{
    // TODO: Add support for abstract unix sockets once supported by tonic.
    // ref. https://github.com/hyperium/tonic/issues/966
    // avalanchego currently only supports plugins listening on IP address.
    let addr = utils::new_socket_addr();

    serve_with_address(vm, addr, stop_ch).await
}

pub async fn serve_with_address<V>(vm: V, addr: SocketAddr, mut stop_ch: Receiver<()>) -> Result<()>
where
    V: Vm,
{
    // "go-plugin requires the gRPC Health Checking Service to be registered on your server"
    // ref. https://github.com/hashicorp/go-plugin/blob/master/docs/guide-plugin-write-non-go.md
    // ref. https://github.com/hyperium/tonic/blob/v0.7.1/examples/src/health/server.rs
    let (mut health_reporter, health_svc) = health_reporter();
    health_reporter.set_serving::<Plugin>().await;

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

    log::info!("plugin listening on address {:?}", addr);

    // handshake message must be printed to stdout
    // ref. https://github.com/hashicorp/go-plugin/blob/master/docs/guide-plugin-write-non-go.md#4-output-handshake-information
    // TODO: remove this once go-plugin is deprecated in avalanchego
    let handshake_config = HandshakeConfig::new();
    let handshake_msg = format!("1|{}|tcp|{}|grpc|", handshake_config.protocol_version, addr);
    println!("{}", handshake_msg);

    grpcutil::default_server()
        .add_service(health_svc)
        .add_service(reflection_service)
        .add_service(VmServer::new(vm))
        .serve_with_shutdown(addr, stop_ch.recv().map(|_| ()))
        .await
        .map_err(|e| Error::new(ErrorKind::Other, format!("grpc server failed: {:?}", e)))?;
    log::info!("grpc server shutdown complete: {}", addr);

    Ok(())
}
