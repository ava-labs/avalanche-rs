// Copyright (C) 2019-2022, Ava Labs, Inc. All rights reserved.
// See the file LICENSE for licensing terms.

package server

import (
	"bytes"
	"compress/gzip"
	"context"
	"crypto/x509"
	"encoding/binary"
	"fmt"
	"io"
	"net"
	"time"

	"github.com/ava-labs/avalanche-rs/avalanchego-conformance/rpcpb"
	"github.com/ava-labs/avalanchego/ids"
	"github.com/ava-labs/avalanchego/message"
	"github.com/ava-labs/avalanchego/proto/pb/p2p"
	"github.com/ava-labs/avalanchego/utils/compression"
	"github.com/ava-labs/avalanchego/utils/ips"
	"github.com/ava-labs/avalanchego/utils/logging"
	"github.com/ava-labs/avalanchego/utils/wrappers"
	"github.com/prometheus/client_golang/prometheus"
	"go.uber.org/zap"
)

func (s *server) AcceptedFrontier(ctx context.Context, req *rpcpb.AcceptedFrontierRequest) (*rpcpb.AcceptedFrontierResponse, error) {
	zap.L().Debug("received AcceptedFrontier request")

	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compression.TypeNone, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	containersIDs := make([]ids.ID, 0, len(req.ContainerIds))
	for _, b := range req.ContainerIds {
		bb := [32]byte{}
		copy(bb[:], b)
		containersIDs = append(containersIDs, ids.ID(bb))
	}

	msg, err := mc.AcceptedFrontier(chainID, req.RequestId, containersIDs)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.AcceptedFrontierResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}

	return resp, nil
}

func (s *server) AcceptedStateSummary(ctx context.Context, req *rpcpb.AcceptedStateSummaryRequest) (*rpcpb.AcceptedStateSummaryResponse, error) {
	zap.L().Debug("received AcceptedStateSummary request")

	compressType := compression.TypeNone
	if req.GzipCompressed {
		compressType = compression.TypeGzip
	}
	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compressType, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	summaryIDs := make([]ids.ID, 0, len(req.SummaryIds))
	for _, b := range req.SummaryIds {
		bb := [32]byte{}
		copy(bb[:], b)
		summaryIDs = append(summaryIDs, ids.ID(bb))
	}

	msg, err := mc.AcceptedStateSummary(chainID, req.RequestId, summaryIDs)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.AcceptedStateSummaryResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !req.GzipCompressed && !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}
	if req.GzipCompressed {
		// gzip/flate2 in Rust/Go are compatible but outputs are different
		rd := new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(expected[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		expectedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}

		rd = new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(req.SerializedMsg[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		receivedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}
		if !bytes.Equal(expectedDecompressed, receivedDecompressed) {
			resp.Message = fmt.Sprintf("decompressed output expected [%x], got [%x]", expectedDecompressed, receivedDecompressed)
			resp.Success = false
		}
	}

	return resp, nil
}

func (s *server) Accepted(ctx context.Context, req *rpcpb.AcceptedRequest) (*rpcpb.AcceptedResponse, error) {
	zap.L().Debug("received Accepted request")

	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compression.TypeNone, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	containersIDs := make([]ids.ID, 0, len(req.ContainerIds))
	for _, b := range req.ContainerIds {
		bb := [32]byte{}
		copy(bb[:], b)
		containersIDs = append(containersIDs, ids.ID(bb))
	}

	msg, err := mc.Accepted(chainID, req.RequestId, containersIDs)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.AcceptedResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}

	return resp, nil
}

func (s *server) Ancestors(ctx context.Context, req *rpcpb.AncestorsRequest) (*rpcpb.AncestorsResponse, error) {
	zap.L().Debug("received Ancestors request")

	compressType := compression.TypeNone
	if req.GzipCompressed {
		compressType = compression.TypeGzip
	}
	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compressType, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	msg, err := mc.Ancestors(chainID, req.RequestId, req.Containers)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.AncestorsResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !req.GzipCompressed && !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}
	if req.GzipCompressed {
		// gzip/flate2 in Rust/Go are compatible but outputs are different
		rd := new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(expected[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		expectedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}

		rd = new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(req.SerializedMsg[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		receivedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}
		if !bytes.Equal(expectedDecompressed, receivedDecompressed) {
			resp.Message = fmt.Sprintf("decompressed output expected [%x], got [%x]", expectedDecompressed, receivedDecompressed)
			resp.Success = false
		}
	}

	return resp, nil
}

func (s *server) AppGossip(ctx context.Context, req *rpcpb.AppGossipRequest) (*rpcpb.AppGossipResponse, error) {
	zap.L().Debug("received AppGossip request")

	compressType := compression.TypeNone
	if req.GzipCompressed {
		compressType = compression.TypeGzip
	}
	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compressType, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	msg, err := mc.AppGossip(chainID, req.AppBytes)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.AppGossipResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !req.GzipCompressed && !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}
	if req.GzipCompressed {
		// gzip/flate2 in Rust/Go are compatible but outputs are different
		rd := new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(expected[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		expectedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}

		rd = new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(req.SerializedMsg[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		receivedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}
		if !bytes.Equal(expectedDecompressed, receivedDecompressed) {
			resp.Message = fmt.Sprintf("decompressed output expected [%x], got [%x]", expectedDecompressed, receivedDecompressed)
			resp.Success = false
		}
	}

	return resp, nil
}

func (s *server) AppRequest(ctx context.Context, req *rpcpb.AppRequestRequest) (*rpcpb.AppRequestResponse, error) {
	zap.L().Debug("received AppRequest request")

	compressType := compression.TypeNone
	if req.GzipCompressed {
		compressType = compression.TypeGzip
	}
	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compressType, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	msg, err := mc.AppRequest(chainID, req.RequestId, time.Duration(req.Deadline), req.AppBytes)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.AppRequestResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !req.GzipCompressed && !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}
	if req.GzipCompressed {
		// gzip/flate2 in Rust/Go are compatible but outputs are different
		rd := new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(expected[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		expectedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}

		rd = new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(req.SerializedMsg[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		receivedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}
		if !bytes.Equal(expectedDecompressed, receivedDecompressed) {
			resp.Message = fmt.Sprintf("decompressed output expected [%x], got [%x]", expectedDecompressed, receivedDecompressed)
			resp.Success = false
		}
	}

	return resp, nil
}

func (s *server) AppResponse(ctx context.Context, req *rpcpb.AppResponseRequest) (*rpcpb.AppResponseResponse, error) {
	zap.L().Debug("received AppResponse request")

	compressType := compression.TypeNone
	if req.GzipCompressed {
		compressType = compression.TypeGzip
	}
	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compressType, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	msg, err := mc.AppResponse(chainID, req.RequestId, req.AppBytes)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.AppResponseResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !req.GzipCompressed && !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}
	if req.GzipCompressed {
		// gzip/flate2 in Rust/Go are compatible but outputs are different
		rd := new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(expected[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		expectedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}

		rd = new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(req.SerializedMsg[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		receivedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}
		if !bytes.Equal(expectedDecompressed, receivedDecompressed) {
			resp.Message = fmt.Sprintf("decompressed output expected [%x], got [%x]", expectedDecompressed, receivedDecompressed)
			resp.Success = false
		}
	}

	return resp, nil
}

func (s *server) Chits(ctx context.Context, req *rpcpb.ChitsRequest) (*rpcpb.ChitsResponse, error) {
	zap.L().Debug("received Chits request")

	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compression.TypeNone, 10*time.Second)
	if err != nil {
		return nil, err
	}

	containersIDs := make([]ids.ID, 0, len(req.ContainerIds))
	for _, b := range req.ContainerIds {
		bb := [32]byte{}
		copy(bb[:], b)
		containersIDs = append(containersIDs, ids.ID(bb))
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	msg, err := mc.Chits(ids.ID(chainID), req.RequestId, containersIDs, nil)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.ChitsResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}

	return resp, nil
}

func (s *server) GetAcceptedFrontier(ctx context.Context, req *rpcpb.GetAcceptedFrontierRequest) (*rpcpb.GetAcceptedFrontierResponse, error) {
	zap.L().Debug("received GetAcceptedFrontier request")

	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compression.TypeNone, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	msg, err := mc.GetAcceptedFrontier(chainID, req.RequestId, time.Duration(req.Deadline), p2p.EngineType_ENGINE_TYPE_SNOWMAN)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.GetAcceptedFrontierResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}

	return resp, nil
}

func (s *server) GetAcceptedStateSummary(ctx context.Context, req *rpcpb.GetAcceptedStateSummaryRequest) (*rpcpb.GetAcceptedStateSummaryResponse, error) {
	zap.L().Debug("received GetAcceptedStateSummary request")

	compressType := compression.TypeNone
	if req.GzipCompressed {
		compressType = compression.TypeGzip
	}
	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compressType, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	msg, err := mc.GetAcceptedStateSummary(chainID, req.RequestId, time.Duration(req.Deadline), req.Heights)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.GetAcceptedStateSummaryResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !req.GzipCompressed && !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}
	if req.GzipCompressed {
		// gzip/flate2 in Rust/Go are compatible but outputs are different
		rd := new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(expected[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		expectedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}

		rd = new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(req.SerializedMsg[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		receivedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}
		if !bytes.Equal(expectedDecompressed, receivedDecompressed) {
			resp.Message = fmt.Sprintf("decompressed output expected [%x], got [%x]", expectedDecompressed, receivedDecompressed)
			resp.Success = false
		}
	}

	return resp, nil
}

func (s *server) GetAccepted(ctx context.Context, req *rpcpb.GetAcceptedRequest) (*rpcpb.GetAcceptedResponse, error) {
	zap.L().Debug("received GetAccepted request")

	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compression.TypeNone, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	containersIDs := make([]ids.ID, 0, len(req.ContainerIds))
	for _, b := range req.ContainerIds {
		bb := [32]byte{}
		copy(bb[:], b)
		containersIDs = append(containersIDs, ids.ID(bb))
	}

	msg, err := mc.GetAccepted(chainID, req.RequestId, time.Duration(req.Deadline), containersIDs, p2p.EngineType_ENGINE_TYPE_SNOWMAN)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.GetAcceptedResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}

	return resp, nil
}

func (s *server) GetAncestors(ctx context.Context, req *rpcpb.GetAncestorsRequest) (*rpcpb.GetAncestorsResponse, error) {
	zap.L().Debug("received GetAncestors request")

	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compression.TypeNone, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	containerID := [32]byte{}
	copy(containerID[:], req.ContainerId)

	msg, err := mc.GetAncestors(chainID, req.RequestId, time.Duration(req.Deadline), containerID, p2p.EngineType_ENGINE_TYPE_SNOWMAN)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.GetAncestorsResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}

	return resp, nil
}

func (s *server) GetStateSummaryFrontier(ctx context.Context, req *rpcpb.GetStateSummaryFrontierRequest) (*rpcpb.GetStateSummaryFrontierResponse, error) {
	zap.L().Debug("received GetStateSummaryFrontier request")

	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compression.TypeNone, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	msg, err := mc.GetStateSummaryFrontier(chainID, req.RequestId, time.Duration(req.Deadline))
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.GetStateSummaryFrontierResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}

	return resp, nil
}

func (s *server) Get(ctx context.Context, req *rpcpb.GetRequest) (*rpcpb.GetResponse, error) {
	zap.L().Debug("received Get request")

	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compression.TypeNone, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	containerID := [32]byte{}
	copy(containerID[:], req.ContainerId)

	msg, err := mc.Get(chainID, req.RequestId, time.Duration(req.Deadline), containerID, p2p.EngineType_ENGINE_TYPE_SNOWMAN)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.GetResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}

	return resp, nil
}

func (s *server) Peerlist(ctx context.Context, req *rpcpb.PeerlistRequest) (*rpcpb.PeerlistResponse, error) {
	zap.L().Debug("received Peerlist request")

	compressType := compression.TypeNone
	if req.GzipCompressed {
		compressType = compression.TypeGzip
	}
	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compressType, 10*time.Second)
	if err != nil {
		return nil, err
	}

	ipCerts := make([]ips.ClaimedIPPort, len(req.Peers))
	for i, p := range req.Peers {
		ipCerts[i] = ips.ClaimedIPPort{
			Cert: &x509.Certificate{Raw: p.Certificate},
			IPPort: ips.IPPort{
				IP:   p.IpAddr,
				Port: uint16(p.IpPort),
			},
			Timestamp: p.GetTimestamp(),
			Signature: p.Sig,
		}
	}

	msg, err := mc.PeerList(ipCerts, true)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.PeerlistResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !req.GzipCompressed && !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}
	if req.GzipCompressed {
		// gzip/flate2 in Rust/Go are compatible but outputs are different
		rd := new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(expected[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		expectedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}

		rd = new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(req.SerializedMsg[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		receivedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}
		if !bytes.Equal(expectedDecompressed, receivedDecompressed) {
			resp.Message = fmt.Sprintf("decompressed output expected [%x], got [%x]", expectedDecompressed, receivedDecompressed)
			resp.Success = false
		}
	}

	return resp, nil
}

func (s *server) Ping(ctx context.Context, req *rpcpb.PingRequest) (*rpcpb.PingResponse, error) {
	zap.L().Debug("received Ping request")

	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compression.TypeNone, 10*time.Second)
	if err != nil {
		return nil, err
	}
	msg, err := mc.Ping()
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.PingResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}

	return resp, nil
}

func (s *server) Pong(ctx context.Context, req *rpcpb.PongRequest) (*rpcpb.PongResponse, error) {
	zap.L().Debug("received Pong request")

	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compression.TypeNone, 10*time.Second)
	if err != nil {
		return nil, err
	}
	msg, err := mc.Pong(req.UptimePct, nil)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.PongResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}

	return resp, nil
}

func (s *server) PullQuery(ctx context.Context, req *rpcpb.PullQueryRequest) (*rpcpb.PullQueryResponse, error) {
	zap.L().Debug("received PullQuery request")

	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compression.TypeNone, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	containerID := [32]byte{}
	copy(containerID[:], req.ContainerId)

	msg, err := mc.PullQuery(ids.ID(chainID), req.RequestId, time.Duration(req.Deadline), ids.ID(containerID), p2p.EngineType_ENGINE_TYPE_SNOWMAN)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.PullQueryResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}

	return resp, nil
}

func (s *server) PushQuery(ctx context.Context, req *rpcpb.PushQueryRequest) (*rpcpb.PushQueryResponse, error) {
	zap.L().Debug("received PushQuery request")

	compressType := compression.TypeNone
	if req.GzipCompressed {
		compressType = compression.TypeGzip
	}
	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compressType, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	msg, err := mc.PushQuery(ids.ID(chainID), req.RequestId, time.Duration(req.Deadline), req.ContainerBytes, p2p.EngineType_ENGINE_TYPE_SNOWMAN)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.PushQueryResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !req.GzipCompressed && !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}
	if req.GzipCompressed {
		// gzip/flate2 in Rust/Go are compatible but outputs are different
		rd := new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(expected[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		expectedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}

		rd = new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(req.SerializedMsg[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		receivedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}
		if !bytes.Equal(expectedDecompressed, receivedDecompressed) {
			resp.Message = fmt.Sprintf("decompressed output expected [%x], got [%x]", expectedDecompressed, receivedDecompressed)
			resp.Success = false
		}
	}

	return resp, nil
}

func (s *server) Put(ctx context.Context, req *rpcpb.PutRequest) (*rpcpb.PutResponse, error) {
	zap.L().Debug("received Put request")

	compressType := compression.TypeNone
	if req.GzipCompressed {
		compressType = compression.TypeGzip
	}
	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compressType, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	msg, err := mc.Put(ids.ID(chainID), req.RequestId, req.ContainerBytes, p2p.EngineType_ENGINE_TYPE_SNOWMAN)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.PutResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !req.GzipCompressed && !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}
	if req.GzipCompressed {
		// gzip/flate2 in Rust/Go are compatible but outputs are different
		rd := new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(expected[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		expectedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}

		rd = new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(req.SerializedMsg[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		receivedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}
		if !bytes.Equal(expectedDecompressed, receivedDecompressed) {
			resp.Message = fmt.Sprintf("decompressed output expected [%x], got [%x]", expectedDecompressed, receivedDecompressed)
			resp.Success = false
		}
	}

	return resp, nil
}

func (s *server) StateSummaryFrontier(ctx context.Context, req *rpcpb.StateSummaryFrontierRequest) (*rpcpb.StateSummaryFrontierResponse, error) {
	zap.L().Debug("received StateSummaryFrontier request")

	compressType := compression.TypeNone
	if req.GzipCompressed {
		compressType = compression.TypeGzip
	}
	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compressType, 10*time.Second)
	if err != nil {
		return nil, err
	}

	chainID := [32]byte{}
	copy(chainID[:], req.ChainId)

	msg, err := mc.StateSummaryFrontier(ids.ID(chainID), req.RequestId, req.Summary)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.StateSummaryFrontierResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !req.GzipCompressed && !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}
	if req.GzipCompressed {
		// gzip/flate2 in Rust/Go are compatible but outputs are different
		rd := new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(expected[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		expectedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}

		rd = new(gzip.Reader)
		// +2; 1 for type ID, 1 for compressible boolean
		if err = rd.Reset(bytes.NewReader(req.SerializedMsg[wrappers.IntLen+2:])); err != nil {
			return nil, err
		}
		receivedDecompressed, err := io.ReadAll(rd)
		if err != nil {
			return nil, err
		}
		if !bytes.Equal(expectedDecompressed, receivedDecompressed) {
			resp.Message = fmt.Sprintf("decompressed output expected [%x], got [%x]", expectedDecompressed, receivedDecompressed)
			resp.Success = false
		}
	}

	return resp, nil
}

func (s *server) Version(ctx context.Context, req *rpcpb.VersionRequest) (*rpcpb.VersionResponse, error) {
	zap.L().Debug("received Version request")

	mc, err := message.NewCreator(logging.NoLog{}, prometheus.NewRegistry(), "", compression.TypeNone, 10*time.Second)
	if err != nil {
		return nil, err
	}
	ip := ips.IPPort{
		IP:   net.IP(req.IpAddr),
		Port: uint16(req.IpPort),
	}
	trackedSubnets := make([]ids.ID, 0, len(req.TrackedSubnets))
	for _, b := range req.TrackedSubnets {
		bb := [32]byte{}
		copy(bb[:], b)
		trackedSubnets = append(trackedSubnets, ids.ID(bb))
	}
	msg, err := mc.Version(
		req.NetworkId,
		req.MyTime,
		ip,
		req.MyVersion,
		req.MyVersionTime,
		req.Sig,
		trackedSubnets,
	)
	if err != nil {
		return nil, err
	}

	// ref. "network/peer.writeMessages"
	msgBytes := msg.Bytes()
	msgLen := uint32(len(msgBytes))
	msgLenBytes := [wrappers.IntLen]byte{}
	binary.BigEndian.PutUint32(msgLenBytes[:], msgLen)
	expected := append(msgLenBytes[:], msgBytes...)

	resp := &rpcpb.VersionResponse{
		ExpectedSerializedMsg: expected,
		Success:               true,
	}
	if !bytes.Equal(req.SerializedMsg, expected) {
		resp.Message = fmt.Sprintf("expected 0x%x", expected)
		resp.Success = false
	}

	return resp, nil
}
