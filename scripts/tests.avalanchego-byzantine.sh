#!/usr/bin/env bash
set -e

# ./scripts/tests.avalanchego-byzantine.sh
# ./scripts/tests.avalanchego-byzantine.sh /Users/leegyuho/avalanchego/build/avalanchego
if ! [[ "$0" =~ scripts/tests.avalanchego-byzantine.sh ]]; then
  echo "must be run from repository root"
  exit 255
fi

NETWORK_RUNNER_AVALANCHEGO_PATH=${1:-""}

echo "Running with:"
echo NETWORK_RUNNER_AVALANCHEGO_PATH: "${NETWORK_RUNNER_AVALANCHEGO_PATH}"

#################################
# download avalanche-network-runner
# https://github.com/ava-labs/avalanche-network-runner
# TODO: use "go install -v github.com/ava-labs/avalanche-network-runner/cmd/avalanche-network-runner@v${NETWORK_RUNNER_VERSION}"
GOOS=$(go env GOOS)
NETWORK_RUNNER_VERSION=1.7.1
DOWNLOAD_PATH=/tmp/avalanche-network-runner.tar.gz
DOWNLOAD_URL=https://github.com/ava-labs/avalanche-network-runner/releases/download/v${NETWORK_RUNNER_VERSION}/avalanche-network-runner_${NETWORK_RUNNER_VERSION}_linux_amd64.tar.gz
if [[ ${GOOS} == "darwin" ]]; then
  DOWNLOAD_URL=https://github.com/ava-labs/avalanche-network-runner/releases/download/v${NETWORK_RUNNER_VERSION}/avalanche-network-runner_${NETWORK_RUNNER_VERSION}_darwin_amd64.tar.gz
fi

rm -f ${DOWNLOAD_PATH}
rm -f /tmp/avalanche-network-runner

echo "downloading avalanche-network-runner ${NETWORK_RUNNER_VERSION} at ${DOWNLOAD_URL}"
curl -L ${DOWNLOAD_URL} -o ${DOWNLOAD_PATH}

echo "extracting downloaded avalanche-network-runner"
tar xzvf ${DOWNLOAD_PATH} -C /tmp
/tmp/avalanche-network-runner -h

#################################
# run "avalanche-network-runner" server
echo "launch avalanche-network-runner in the background"
/tmp/avalanche-network-runner \
server \
--log-level debug \
--port=":13342" \
--disable-grpc-gateway &
NETWORK_RUNNER_PID=${!}
sleep 5

#################################
# for testing local
# AVALANCHEGO_PATH=/Users/leegyuho/avalanchego/build/avalanchego
#
# do not run in parallel, to run in sequence
echo "running byzantine tests"
NETWORK_RUNNER_GRPC_ENDPOINT=http://127.0.0.1:13342 \
RUST_LOG=debug \
cargo test --all-features --package avalanchego-byzantine -- --show-output --nocapture

#################################
# "e2e.test" already terminates the cluster for "test" mode
# just in case tests are aborted, manually terminate them again
echo "network-runner RPC server was running on NETWORK_RUNNER_PID ${NETWORK_RUNNER_PID} as test mode; terminating the process..."
pkill -P ${NETWORK_RUNNER_PID} || true
kill -2 ${NETWORK_RUNNER_PID} || true

echo "TEST SUCCESS"
