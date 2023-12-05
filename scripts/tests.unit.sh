#!/usr/bin/env bash
set -xue

if ! [[ "$0" =~ scripts/tests.unit.sh ]]; then
  echo "must be run from repository root"
  exit 255
fi

# uses nextest for faster test runs and better output
# https://github.com/nextest-rs/nextest/tree/main
# local use: cargo install nextest

RUST_LOG=debug cargo test \
--all-features \
-p avalanche-types \
-p avalanche-consensus

echo "ALL SUCCESS!"
