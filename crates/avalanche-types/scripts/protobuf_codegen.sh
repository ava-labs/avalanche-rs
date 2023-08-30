#!/usr/bin/env bash

# protocol version is the version of the gRPC proto definitions
# as defined by the avalanchego rpcchainvm.
# ref. https://github.com/ava-labs/avalanchego/blob/v1.9.11/version/constants.go#L15-L17
PROTOCOL_VERSION='26'

if ! [[ "$0" =~ scripts/protobuf_codegen.sh ]]; then
  echo "must be run from repository root"
  exit 255
fi

# ref. https://docs.buf.build/installation
BUF_VERSION='1.19.0'
if [[ $(buf --version | cut -f2 -d' ') != "${BUF_VERSION}" ]]; then
  echo "could not find buf ${BUF_VERSION}, is it installed + in PATH?"
  exit 255
fi

# protoc plugin "protoc-gen-prost" is required
#
# e.g.,
# cargo install protoc-gen-prost --version 0.2.2
# ref. https://crates.io/crates/protoc-gen-prost
PROTOC_GEN_PROST_VERSION=0.2.2
if [[ $(protoc-gen-prost --version | cut -f2 -d' ') != "${PROTOC_GEN_PROST_VERSION}" ]]; then
  echo "could not find protoc-gen-prost version ${PROTOC_GEN_PROST_VERSION} is it installed + in PATH?"
  exit 255
fi

# protoc plugin "protoc-gen-tonic" is required
#
# e.g.,
# cargo install protoc-gen-tonic --version 0.2.2
# ref. https://crates.io/crates/protoc-gen-tonic
PROTOC_GEN_TONIC_VERSION=0.2.2
if [[ $(protoc-gen-tonic --version | cut -f2 -d' ') != "${PROTOC_GEN_TONIC_VERSION}" ]]; then
  echo "could not find protoc-gen-tonic version ${PROTOC_GEN_TONIC_VERSION} is it installed + in PATH?"
  exit 255
fi

# protoc plugin "protoc-gen-prost-crate" is required
#
# e.g.,
# cargo install protoc-gen-prost-crate --version 0.3.1 
# ref. https://crates.io/crates/protoc-gen-prost-crate
PROTOC_GEN_PROST_CRATE_VERSION=0.3.1
if [[ $(protoc-gen-prost-crate --version | cut -f2 -d' ') != "${PROTOC_GEN_PROST_CRATE_VERSION}" ]]; then
  echo "could not find protoc-gen-prost-crate version ${PROTOC_GEN_PROST_CRATE_VERSION} is it installed + in PATH?"
  exit 255
fi

pushd ./src/proto || return

# cleanup previous protos
rm -rf ./protos/avalanche

# pull source from buf registry
echo "Pulling proto source for protocol version: ${PROTOCOL_VERSION}..."
buf export buf.build/ava-labs/avalanche:v"${PROTOCOL_VERSION}" -o ./protos/avalanche

echo "Re-generating proto stubs..."
buf generate

if [[ $? -ne 0 ]];  then
    echo "ERROR: buf generate proto stubs failed"
    exit 1
fi
