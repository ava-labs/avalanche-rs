use avalanche_types::{avm::txs::vertex::Vertex, ids, packer::Packer};
use avalanchego_conformance_sdk::{BuildVertexRequest, Client};
use log::info;

#[tokio::test]
async fn build_vertex() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let mut vtx = Vertex {
        codec_version: 0,
        chain_id: ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        height: random_manager::u64(),
        epoch: 0,
        parent_ids: vec![
            ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
            ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
            ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
            ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
            ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ],
        txs: vec![
            random_manager::secure_bytes(32).unwrap(),
            random_manager::secure_bytes(32).unwrap(),
            random_manager::secure_bytes(32).unwrap(),
            random_manager::secure_bytes(32).unwrap(),
            random_manager::secure_bytes(32).unwrap(),
        ],
    };

    let mut parent_ids_copied: Vec<Vec<u8>> = Vec::new();
    for id in vtx.parent_ids.iter() {
        parent_ids_copied.push(id.as_ref().to_vec().clone());
    }
    let mut txs_copied: Vec<Vec<u8>> = Vec::new();
    for tx in vtx.txs.iter() {
        txs_copied.push(tx.clone());
    }

    let mut req = BuildVertexRequest {
        codec_version: 0,
        chain_id: vtx.chain_id.clone().as_ref().to_vec(),
        height: vtx.height,
        epoch: 0,
        parent_ids: parent_ids_copied,
        txs: txs_copied,
        vtx_bytes: Vec::new(),
    };
    let packer = Packer::new(1024, 0);
    packer.pack_vertex(&mut vtx).unwrap();
    let b = packer.take_bytes();

    req.vtx_bytes = b.as_ref().to_vec().clone();
    info!("built vertex ({} bytes)", req.vtx_bytes.len());

    let resp = cli.build_vertex(req).await.expect("failed build_vertex");
    assert!(resp.success);
}
