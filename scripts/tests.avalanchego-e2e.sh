#!/usr/bin/env bash
set -e

# ./scripts/tests.avalanchego-e2e.sh
# ./scripts/tests.avalanchego-e2e.sh ~/go/src/github.com/ava-labs/avalanchego/build/avalanchego
#
# to test AWS KMS
# E2E_FLAGS="--sign-with-kms-aws" ./scripts/tests.avalanchego-e2e.sh
#
if ! [[ "$0" =~ scripts/tests.avalanchego-e2e.sh ]]; then
  echo "must be run from repository root"
  exit 255
fi

AVALANCHEGO_PATH=$1
DEFAULT_SPEC_FLAGS=""
if [[ ! -z "${AVALANCHEGO_PATH}" ]]; then
  DEFAULT_SPEC_FLAGS="--network-runner-avalanchego-path=${AVALANCHEGO_PATH}"
fi

E2E_FLAGS=${E2E_FLAGS:-"--keys-to-generate 30"}

#################################
# download avalanche-network-runner
# https://github.com/ava-labs/avalanche-network-runner
# TODO: use "go install -v github.com/ava-labs/avalanche-network-runner/cmd/avalanche-network-runner@v${NETWORK_RUNNER_VERSION}"
GOOS=$(go env GOOS)
NETWORK_RUNNER_VERSION=1.7.7
DOWNLOAD_PATH=/tmp/avalanche-network-runner.tar.gz
DOWNLOAD_URL=https://github.com/ava-labs/avalanche-network-runner/releases/download/v${NETWORK_RUNNER_VERSION}/avalanche-network-runner_${NETWORK_RUNNER_VERSION}_linux_amd64.tar.gz
if [[ ${GOOS} == "darwin" ]]; then
  DOWNLOAD_URL=https://github.com/ava-labs/avalanche-network-runner/releases/download/v${NETWORK_RUNNER_VERSION}/avalanche-network-runner_${NETWORK_RUNNER_VERSION}_darwin_amd64.tar.gz
fi
echo ${DOWNLOAD_URL}

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
--port=":12342" \
--disable-grpc-gateway &
NETWORK_RUNNER_PID=${!}
sleep 5

#################################
echo "running e2e tests"
cargo build \
--release \
-p avalanche-e2e

./target/release/avalanche-e2e -h
./target/release/avalanche-e2e default-spec -h

# "1337" for network runner
./target/release/avalanche-e2e \
--spec-path /tmp/tests.avalanchego-e2e.yaml \
default-spec \
--network-id 1337 ${E2E_FLAGS} \
--network-runner-grpc-endpoint http://127.0.0.1:12342 ${DEFAULT_SPEC_FLAGS}

./target/release/avalanche-e2e \
--skip-prompt \
--spec-path /tmp/tests.avalanchego-e2e.yaml

#################################
# "e2e.test" already terminates the cluster for "test" mode
# just in case tests are aborted, manually terminate them again
echo "network-runner RPC server was running on NETWORK_RUNNER_PID ${NETWORK_RUNNER_PID} as test mode; terminating the process..."
pkill -P ${NETWORK_RUNNER_PID} || true
kill -2 ${NETWORK_RUNNER_PID} || true

echo "TEST SUCCESS"
