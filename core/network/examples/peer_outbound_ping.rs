use std::{
    array,
    env::args,
    io,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    time::{Duration, SystemTime},
};

use avalanche_types::{ids::Id, message};
use network::{cert_manager, peer::outbound};

/// cargo run --example peer_outbound_ping -- [PEER IP] [STAKING PORT]
/// cargo run --example peer_outbound_ping -- 34.222.2.60 9651
/// cargo run --example peer_outbound_ping -- 35.167.53.168 9651
/// cargo run --example peer_outbound_ping -- 52.43.89.51 9651
/// cargo run --example peer_outbound_ping -- 35.89.110.175 9651
/// NOTE: just pick random node/IP from https://github.com/ava-labs/avalanchego/blob/master/genesis/bootstrappers.json
fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let peer_ip = args().nth(1).expect("no peer IP given");
    let peer_ip = IpAddr::from_str(&peer_ip).expect("invalid peer IP");

    let port = args().nth(2).expect("no port given");
    let port: u16 = port.parse().unwrap();

    let addr = SocketAddr::new(peer_ip, port);

    let key_path = random_manager::tmp_path(10, Some(".key")).unwrap();
    let cert_path = random_manager::tmp_path(10, Some(".cert")).unwrap();
    cert_manager::x509::generate_and_write_pem(None, &key_path, &cert_path)?;

    let connector = outbound::Connector::new_from_pem(&key_path, &cert_path)?;
    let mut stream = connector.connect(addr, Duration::from_secs(10))?;
    log::info!("peer certificate:\n\n{}", stream.peer_certificate_pem);

    log::info!("sending version...");
    let now = SystemTime::now();
    let now_unix = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("unexpected None duration_since")
        .as_secs();
    let tracked_subnets: [Id; 5] =
        array::from_fn(|_| Id::from_slice(&random_manager::secure_bytes(32).unwrap()));

    let msg = message::version::Message::default()
        .network_id(1000000)
        .my_time(now_unix)
        .ip_addr(addr.ip())
        .ip_port(0)
        .my_version("avalanche/1.2.3".to_string())
        .sig(random_manager::secure_bytes(64).unwrap())
        .tracked_subnets(tracked_subnets);
    let msg = msg.serialize().expect("failed serialize");
    stream.write(&msg)?;

    log::info!("sending ping...");
    let ping_msg = message::ping::Message::default();
    let ping_msg_bytes = ping_msg.serialize()?;
    stream.write(&ping_msg_bytes)?;
    stream.close()?;

    Ok(())
}
