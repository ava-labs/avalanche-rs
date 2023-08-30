// Copyright (C) 2019-2022, Ava Labs, Inc. All rights reserved.
// See the file LICENSE for licensing terms.

package server

import (
	"context"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/ava-labs/avalanche-rs/avalanchego-conformance/pkg/logutil"
	"github.com/ava-labs/avalanche-rs/avalanchego-conformance/server"
	"github.com/spf13/cobra"
	"go.uber.org/zap"
)

func init() {
	cobra.EnablePrefixMatching = true
}

var (
	logLevel    string
	port        uint16
	gwPort      uint16
	dialTimeout time.Duration
)

func NewCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "server [options]",
		Short: "Start a network runner server.",
		RunE:  serverFunc,
	}

	cmd.PersistentFlags().StringVar(&logLevel, "log-level", logutil.DefaultLogLevel, "log level")
	cmd.PersistentFlags().Uint16Var(&port, "port", 9090, "server port")
	cmd.PersistentFlags().Uint16Var(&gwPort, "grpc-gateway-port", 9091, "grpc-gateway server port")
	cmd.PersistentFlags().DurationVar(&dialTimeout, "dial-timeout", 10*time.Second, "server dial timeout")

	return cmd
}

func serverFunc(cmd *cobra.Command, args []string) (err error) {
	lcfg := logutil.GetDefaultZapLoggerConfig()
	lcfg.Level = zap.NewAtomicLevelAt(logutil.ConvertToZapLevel(logLevel))
	logger, err := lcfg.Build()
	if err != nil {
		log.Fatalf("failed to build global logger, %v", err)
	}
	_ = zap.ReplaceGlobals(logger)

	s, err := server.New(server.Config{
		Port:        port,
		GwPort:      gwPort,
		DialTimeout: dialTimeout,
	})
	if err != nil {
		return err
	}

	rootCtx, rootCancel := context.WithCancel(context.Background())
	errc := make(chan error)
	go func() {
		errc <- s.Run(rootCtx)
	}()

	sigc := make(chan os.Signal, 1)
	signal.Notify(sigc, syscall.SIGINT, syscall.SIGTERM)
	select {
	case sig := <-sigc:
		zap.L().Warn("signal received; closing server", zap.String("signal", sig.String()))
		rootCancel()
		zap.L().Warn("closed server", zap.Error(<-errc))
	case err = <-errc:
		zap.L().Warn("server closed", zap.Error(err))
		rootCancel()
	}
	return err
}
