/// ref. <https://github.com/hyperium/tonic/tree/master/tonic-build>
fn main() {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &[
                "../avalanchego-conformance/rpcpb/key.proto",
                "../avalanchego-conformance/rpcpb/message.proto",
                "../avalanchego-conformance/rpcpb/packer.proto",
                "../avalanchego-conformance/rpcpb/ping.proto",
            ],
            &["../avalanchego-conformance/rpcpb"],
        )
        .unwrap();
}
