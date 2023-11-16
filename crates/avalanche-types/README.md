# avalanche-types

## Introduction

The `avalanche-types` crate implements and is the canonical representation of Avalanche primitive types in Rust. Avalanche types are separated by modules and are all under the `src` directory.

This crate also provides an SDK library for developing subnets in Rust. For the SDK functionality, see `src/subnet` which contains everything required to build a subnet VM in Rust.

The following VMs were built with the SDK:
* Simple Rust VM: [TimestampVM](https://github.com/ava-labs/timestampvm-rs)
* Complex Rust VM: [SpacesVM](https://github.com/ava-labs/spacesvm-rs)

## Getting Started

Examples can be found in [`examples`](./examples) and are a good first step to getting an understanding of general usage.

### Resources

- [How to Build a Simple Rust VM](https://docs.avax.network/subnets/create-a-simple-rust-vm) tutorial provides a basic example of using the Rust SDK.
- [TimestampVM Template](https://github.com/ava-labs/timestampvm-rs-template) allows you to quickly generate a [TimestampVM](https://github.com/ava-labs/timestampvm-rs) based project with [cargo generate](https://cargo-generate.github.io/cargo-generate/)

### Rust Version

This project uses the latest stable Rust toolchain.

## Getting Help

First please try find the answer to your question in the code documentation. If more clarification is required, try opening an [issue] with the question.

[issue]: https://github.com/ava-labs/avalanche-rs/issues/new

## Features

- Ids (e.g., [`src/ids`](./src/ids))
- Transaction types/serialization (e.g., [`src/platformvm/txs`](./src/platformvm/txs))
- Certificates (e.g., [`src/key/cert`](./src/key/cert))
- Keys and addresses (e.g., [`src/key/secp256k1`](./src/key/secp256k1))
- Peer-to-peer messages (e.g., [`src/message`](./src/message))
- RPC chain VM (e.g., [`src/subnet/rpc`](./src/subnet/rpc))
- Genesis generate helper (e.g., [`src/subnet_evm`](./src/subnet_evm))
- Protobuf generated stubs and helpers (e.g., [`src/proto`](./src/proto))
- AvalancheGo APIs (e.g., [`/src/avalanchego`](./src/avalanchego))

The basic types available in this crate are used in other Avalanche Rust projects (e.g., distributed load tester [`blizzard`](https://talks.gyuho.dev/distributed-load-generator-avalanche-2022.html), [`avalanche-ops`](https://github.com/ava-labs/avalanche-ops)).
