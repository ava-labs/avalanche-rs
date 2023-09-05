# Avalanche-rs Proto

Now Serving: **Protocol Version 28**

Protobuf files are hosted at
[https://buf.build/ava-labs/avalanche](https://buf.build/ava-labs/avalanche) and
can be used as dependencies in other projects.

Protobuf linting and generation for this project is managed by
[buf](https://github.com/bufbuild/buf).

Please find installation instructions on
[https://docs.buf.build/installation/](https://docs.buf.build/installation/) or
use `Dockerfile.buf` provided in the `proto/` directory of AvalancheGo.

Introduction to `buf`
[https://docs.buf.build/tour/introduction](https://docs.buf.build/tour/introduction)

To update the protocol version update the `PROTOCOL_VERSION` environment variable
in `scripts/protobuf_codegen.sh` and `mod.rs` then run the script.

