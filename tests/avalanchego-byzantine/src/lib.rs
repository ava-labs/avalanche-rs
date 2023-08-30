#[cfg(test)]
mod tests;

pub fn get_network_runner_grpc_endpoint() -> (String, bool) {
    match std::env::var("NETWORK_RUNNER_GRPC_ENDPOINT") {
        Ok(s) => (s, true),
        _ => (String::new(), false),
    }
}

pub fn get_network_runner_avalanchego_path() -> (String, bool) {
    match std::env::var("NETWORK_RUNNER_AVALANCHEGO_PATH") {
        Ok(s) => (s, true),
        _ => (String::new(), false),
    }
}
