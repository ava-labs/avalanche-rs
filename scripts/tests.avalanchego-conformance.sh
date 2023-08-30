#!/usr/bin/env bash
set -e

if ! [[ "$0" =~ scripts/tests.avalanchego-conformance.sh ]]; then
  echo "must be run from repository root"
  exit 255
fi

#################################
pushd avalanchego-conformance
go install -v ./cmd/avalanchego-conformance
popd

#################################
# run "avalanchego-conformance" server
echo "launch avalanchego-conformance in the background"
avalanchego-conformance \
server \
--log-level debug \
--port=22342 \
--grpc-gateway-port=22343 &
AVALANCHEGO_CONFORMANCE_PID=${!}
echo "avalanchego-conformance server is running on PID ${AVALANCHEGO_CONFORMANCE_PID}"

#################################
echo "running conformance tests"
AVALANCHEGO_CONFORMANCE_SERVER_RPC_ENDPOINT=http://127.0.0.1:22342 \
RUST_LOG=debug \
cargo test --all-features --package avalanchego-conformance -- --show-output --nocapture

#################################
echo "SUCCESS conformance tests"
kill -2 ${AVALANCHEGO_CONFORMANCE_PID} || true

echo "TEST SUCCESS"
