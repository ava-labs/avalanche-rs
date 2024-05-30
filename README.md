**This repository is no longer maintained. If you are interested in maintaining it, please reach out in the discussion tab.**

# avalanche-rs
![Github Actions](https://github.com/ava-labs/avalanche-rs/actions/workflows/e2e.yml/badge.svg)
[![Latest](https://img.shields.io/badge/avalanche-types?color=orange)](https://crates.io/crates/avalanche-types)
[![Ecosystem license](https://img.shields.io/badge/License-Ecosystem-blue.svg)](./LICENSE.md)

### **Disclosure:**

> :warning: avalanche-rs is alpha-level software and is not ready for production
> use. Do not use avalanche-rs to run production workloads. See the
> [license](./LICENSE) for more information regarding usage.

avalanche-rs is a collection of crates that provides all the necessary abstractions to develop Rust-based applications and VMs in the Avalanche ecosystem. It provides the canonical type definitions of all of the various Avalanche APIs, on par with those in [avalanchego](https://github.com/ava-labs/avalanchego), but for Rust developers. 

avalanche-rs is composed of several crates:
* [core](./core/) - Core networking components for a p2p Avalanche node.
* [avalanche-consensus](./crates/avalanche-consensus/) - A Rust implementation of the novel Avalanche consensus protocol.
* [avalanche-types](./crates/avalanche-types/) - Foundational types used in Avalanche, including those used by the JSON-RPC API and the EVM.
* [A Rust SDK](./crates/avalanche-types/src/subnet/) for developing Avalanche VMs.

For detailed developer documentation, check out the crate level documentation for [avalanche-types](https://docs.rs/crate/avalanche-types/latest) and [avalanche-consensus](https://docs.rs/crate/avalanche-consensus/latest).

## Goals of avalanche-rs

### Provide Interoperability with avalanchego

avalanche-rs provides core modules and APIs designed to build clients and other Avalanche tooling in Rust. Rust clients can interact with existing avalanchego clients.

### Ergonomic, modular APIs

avalanche-rs provides a wide set of modules to use as imports in other projects. Each module is small in scope and can be imported separately as needed.

### Enable Rust Developers to Build VMs
The Rust SDK in [subnet](./crates/avalanche-types/src/subnet/) provides tools to build Rust VMs on Avalanche.

## Releasing New Versions
To release a new version, first be sure to increment the version in the `Cargo.toml` file for one or both of `avalanche-types` and `avalanche-consensus` to the new version. It's not possible to republish an existing version of a crate on crates.io. The other crates in this project are not published and do not need to be updated. Be sure the version is semver-compatible.

Once the version is incremented, checkout the main branch and pull latest. Next create and push a tag with the name of the crate followed by the new crate version (for example, `avalanche-types-v0.1.0`). The CI automation will generate a draft release. Once that released is published CI will automatically push the latest versions of the crate in the tag to crates.io.

## License 
avalanche-rs is licensed under the [Ecosystem License](./LICENSE).

## Getting Help

First please try find the answer to your question in the code documentation. If more clarification is required, try opening an [issue] with the question.

[issue]: https://github.com/ava-labs/avalanche-rs/issues/new
