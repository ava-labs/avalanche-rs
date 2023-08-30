#!/usr/bin/env bash
set -xue

if ! [[ "$0" =~ scripts/build.release.sh ]]; then
  echo "must be run from repository root"
  exit 255
fi

# "--bin" can be specified multiple times for each directory in "bin/*" or workspaces
# cargo build \
# --release \
# --bin avalanche-node-rust \
# --bin avalanche-cli-rust \
# --bin avalanche-e2e

# ./target/release/avalanche-node-rust --help
# ./target/release/avalanche-cli-rust --help
# ./target/release/avalanche-e2e --help

cargo build \
--release \
-p avalanche-types \
-p avalanche-consensus 
