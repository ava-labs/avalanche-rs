use std::net::IpAddr;

use avalanche_types::message::peerlist::{self, ClaimedIpPort};
use avalanchego_conformance_sdk::{Client, Peer as RpcPeer, PeerlistRequest};

#[tokio::test]
async fn peerlist() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let claimed_ip_ports = vec![
        ClaimedIpPort {
            certificate: random_manager::secure_bytes(50).unwrap(),
            ip_addr: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            ip_port: 8080,
            time: 7,
            sig: random_manager::secure_bytes(20).unwrap(),
        },
        ClaimedIpPort {
            certificate: random_manager::secure_bytes(50).unwrap(),
            ip_addr: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            ip_port: 8081,
            time: 7,
            sig: random_manager::secure_bytes(20).unwrap(),
        },
    ];
    let msg = peerlist::Message::default().claimed_ip_ports(claimed_ip_ports.clone());
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let mut rpc_peers: Vec<RpcPeer> = Vec::new();
    for p in claimed_ip_ports.iter() {
        let ip_bytes = match p.ip_addr {
            IpAddr::V4(v) => {
                // "avalanchego" encodes IPv4 address as it is
                // (not compatible with IPv6, e.g., prepends 2 "0xFF"s as in Rust)
                let octets = v.octets();
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, octets[0], octets[1], octets[2], octets[3],
                ]
            }
            IpAddr::V6(v) => v.octets(),
        };
        rpc_peers.push(RpcPeer {
            certificate: p.certificate.clone(),
            ip_addr: ip_bytes.to_vec(),
            ip_port: p.ip_port as u32,
            timestamp: p.time,
            sig: p.sig.clone(),
        });
    }
    let resp = cli
        .peerlist(PeerlistRequest {
            peers: rpc_peers,
            gzip_compressed: false,
            serialized_msg,
        })
        .await
        .expect("failed peerlist");
    assert!(resp.success);
}

#[tokio::test]
async fn peerlist_gzip_compress() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (ep, is_set) = crate::get_endpoint();
    assert!(is_set);
    let cli = Client::new(&ep).await;

    let claimed_ip_ports = vec![
        ClaimedIpPort {
            certificate: random_manager::secure_bytes(50).unwrap(),
            ip_addr: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            ip_port: 8080,
            time: 7,
            sig: random_manager::secure_bytes(20).unwrap(),
        },
        ClaimedIpPort {
            certificate: random_manager::secure_bytes(50).unwrap(),
            ip_addr: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            ip_port: 8081,
            time: 7,
            sig: random_manager::secure_bytes(20).unwrap(),
        },
    ];
    let msg = peerlist::Message::default()
        .claimed_ip_ports(claimed_ip_ports.clone())
        .gzip_compress(true);
    let serialized_msg = msg.serialize().expect("failed serialize");

    log::info!("sending message ({} bytes)", serialized_msg.len());

    let mut rpc_peers: Vec<RpcPeer> = Vec::new();
    for p in claimed_ip_ports.iter() {
        let ip_bytes = match p.ip_addr {
            IpAddr::V4(v) => {
                // "avalanchego" encodes IPv4 address as it is
                // (not compatible with IPv6, e.g., prepends 2 "0xFF"s as in Rust)
                let octets = v.octets();
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, octets[0], octets[1], octets[2], octets[3],
                ]
            }
            IpAddr::V6(v) => v.octets(),
        };
        rpc_peers.push(RpcPeer {
            certificate: p.certificate.clone(),
            ip_addr: ip_bytes.to_vec(),
            ip_port: p.ip_port as u32,
            timestamp: p.time,
            sig: p.sig.clone(),
        });
    }
    let resp = cli
        .peerlist(PeerlistRequest {
            peers: rpc_peers,
            gzip_compressed: true,
            serialized_msg,
        })
        .await
        .expect("failed peerlist");
    assert!(resp.success);
}
