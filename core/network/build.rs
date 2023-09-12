/// ref. <https://github.com/hyperium/tonic/tree/master/tonic-build>
fn main() {
    tonic_build::configure()
        .out_dir("./src/p2p")
        .build_server(true)
        .build_client(true)
        .compile(&["./src/p2p/gossip/sdk.proto"], &["./src/p2p/gossip/"])
        .unwrap();

}
