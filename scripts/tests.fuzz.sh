#!/usr/bin/env bash
set -xue

if ! [[ "$0" =~ scripts/tests.fuzz.sh ]]; then
  echo "must be run from repository root"
  exit 255
fi

# ref. https://github.com/rust-fuzz/cargo-fuzz
# ref. https://rust-fuzz.github.io/book/cargo-fuzz/setup.html
rustup default nightly

pushd crates/avalanche-types
cargo fuzz run ids
popd

rustup default stable

cargo clean

echo "ALL SUCCESS!"
