#[cfg(test)]
mod tests;

pub fn get_endpoint() -> (String, bool) {
    match std::env::var("AVALANCHEGO_CONFORMANCE_SERVER_RPC_ENDPOINT") {
        Ok(s) => (s, true),
        _ => (String::new(), false),
    }
}
