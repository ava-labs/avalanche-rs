#!/usr/bin/env bash

# This script is used to generate the protobuf stubs for the avalanche-types
# crate.

# protocol version is the version of the gRPC proto definitions
# as defined by the avalanchego rpcchainvm.
# ref. https://github.com/ava-labs/avalanchego/blob/v1.9.11/version/constants.go#L15-L17
PROTOCOL_VERSION='29'

if ! [[ "$0" =~ scripts/protobuf_codegen.sh ]]; then
  echo "must be run from repository root"
  exit 255
fi

# ref. https://docs.buf.build/installation
BUF_VERSION='1.29.0'
if [[ $(buf --version | cut -f2 -d' ') != "${BUF_VERSION}" ]]; then
  echo "could not find buf ${BUF_VERSION}, is it installed + in PATH?"
  exit 255
fi

# protoc-gen-prost and protoc-gen-tonic are now community modules hosted by buf
# and not required by this script.
#
# ref. https://buf.build/community/neoeinstein-tonic
# ref. https://buf.build/community/neoeinstein-prost

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
