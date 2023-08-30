// Copyright (C) 2019-2022, Ava Labs, Inc. All rights reserved.
// See the file LICENSE for licensing terms.

package server

import (
	"bytes"
	"context"
	"fmt"

	"github.com/ava-labs/avalanche-rs/avalanchego-conformance/rpcpb"
	"github.com/ava-labs/avalanchego/ids"
	"github.com/ava-labs/avalanchego/snow/engine/avalanche/vertex"
	"go.uber.org/zap"
)

func (s *server) BuildVertex(ctx context.Context, req *rpcpb.BuildVertexRequest) (*rpcpb.BuildVertexResponse, error) {
	zap.L().Info("received BuildVertex request")

	chainID, err := ids.ToID(req.ChainId)
	if err != nil {
		return nil, err
	}
	parentIDs := make([]ids.ID, 0, len(req.ParentIds))
	for _, b := range req.ParentIds {
		parentID, err := ids.ToID(b)
		if err != nil {
			return nil, err
		}
		parentIDs = append(parentIDs, parentID)
	}

	vtx, err := vertex.Build(chainID, req.Height, parentIDs, req.Txs)
	if err != nil {
		return nil, err
	}
	expectedVtxBytes := vtx.Bytes()

	resp := &rpcpb.BuildVertexResponse{
		ExpectedBytes: expectedVtxBytes,
		Success:       true,
	}
	if !bytes.Equal(req.VtxBytes, expectedVtxBytes) {
		resp.Message = fmt.Sprintf("expected 0x%x", expectedVtxBytes)
		resp.Success = false
	}

	return resp, nil
}
