
```bash
./scripts/build.release.sh

./target/release/avalanche-e2e -h
./target/release/avalanche-e2e default-spec -h

./target/release/avalanche-e2e \
--spec-path /tmp/tests.avalanchego-e2e.yaml \
default-spec \
--keys-to-generate 5 \
--network-runner-grpc-endpoint http://127.0.0.1:12342

./target/release/avalanche-e2e \
--skip-prompt \
--spec-path /tmp/tests.avalanchego-e2e.yaml
```
