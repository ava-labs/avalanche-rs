// Copyright (C) 2019-2022, Ava Labs, Inc. All rights reserved.
// See the file LICENSE for licensing terms.

package main

import (
	"fmt"
	"os"

	"github.com/ava-labs/avalanche-rs/avalanchego-conformance/cmd/avalanchego-conformance/server"
	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:        "avalanchego-conformance",
	Short:      "avalanchego-conformance commands",
	SuggestFor: []string{"avalanche-conformance"},
}

func init() {
	cobra.EnablePrefixMatching = true
}

func init() {
	rootCmd.AddCommand(
		server.NewCommand(),
	)
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "avalanchego-conformance failed %v\n", err)
		os.Exit(1)
	}
	os.Exit(0)
}
