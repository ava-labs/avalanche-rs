# avalanche-conformance 

avalanche-conformance is a tool used to ensure code is compatible with avalanchego. The tool performs byte level 
checking for the avalanchego codec as well as key generations. The code is run against a live avalanchego node. 

avalanche-conformance is a gRPC server that accepts client connections on a configurable port. A client library is also
provided. 

## Usage 

```bash
go install -v ./cmd/avalanchego-conformance
avalanchego-conformance server \
--port 9090 \
--grpc-gateway-port 9091
```

Use a client-side tool like `grpc-curl` or `curl` to send the server requests.

The following gRPC messages are implemented by the gRPC server:

Keys 
* CertificateToNodeId
* Secp256K1RecoverHashPublicKey
* Secp256K1Info
* BlsSignature

Node Messages 
* AcceptedFrontier
* AcceptedStateSummary
* Accepted
* Ancestors
* AppGossip
* AppRequest
* AppResponse
* Chits
* GetAcceptedFrontier
* GetAcceptedStateSummary
* GetAccepted
* GetAncestors
* GetStateSummaryFrontier
* Get
* Peerlist
* Ping
* Pong
* PullQuery
* PushQuery
* Put
* StateSummaryFrontier
* Version

Vertex Messages
* BuildVertex

Server Messages
* PingService