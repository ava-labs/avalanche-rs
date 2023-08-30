// Copyright (C) 2019-2022, Ava Labs, Inc. All rights reserved.
// See the file LICENSE for licensing terms.

// Package client implements client.
package client

import (
	"context"
	"sync"
	"time"

	"github.com/ava-labs/avalanche-rs/avalanchego-conformance/pkg/color"
	"github.com/ava-labs/avalanche-rs/avalanchego-conformance/pkg/logutil"
	"github.com/ava-labs/avalanche-rs/avalanchego-conformance/rpcpb"
	"go.uber.org/zap"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

type Config struct {
	LogLevel    string
	Endpoint    string
	DialTimeout time.Duration
}

type Client interface {
	PingService(ctx context.Context) (*rpcpb.PingServiceResponse, error)
	Close() error
}

type client struct {
	cfg Config

	conn *grpc.ClientConn

	pingc rpcpb.PingServiceClient

	closed    chan struct{}
	closeOnce sync.Once
}

func New(cfg Config) (Client, error) {
	lcfg := logutil.GetDefaultZapLoggerConfig()
	lcfg.Level = zap.NewAtomicLevelAt(logutil.ConvertToZapLevel(cfg.LogLevel))
	logger, err := lcfg.Build()
	if err != nil {
		return nil, err
	}
	_ = zap.ReplaceGlobals(logger)

	color.Outf("{{blue}}dialing endpoint %q{{/}}\n", cfg.Endpoint)
	ctx, cancel := context.WithTimeout(context.Background(), cfg.DialTimeout)
	conn, err := grpc.DialContext(
		ctx,
		cfg.Endpoint,
		grpc.WithBlock(),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	cancel()
	if err != nil {
		return nil, err
	}

	return &client{
		cfg:    cfg,
		conn:   conn,
		pingc:  rpcpb.NewPingServiceClient(conn),
		closed: make(chan struct{}),
	}, nil
}

func (c *client) PingService(ctx context.Context) (*rpcpb.PingServiceResponse, error) {
	zap.L().Info("ping service")

	// ref. https://grpc-ecosystem.github.io/grpc-gateway/docs/tutorials/adding_annotations/
	// curl -X POST -k http://localhost:8081/v1/ping-service -d ''
	return c.pingc.PingService(ctx, &rpcpb.PingServiceRequest{})
}

func (c *client) Close() error {
	c.closeOnce.Do(func() {
		close(c.closed)
	})
	return c.conn.Close()
}
