// Copyright (C) 2019-2022, Ava Labs, Inc. All rights reserved.
// See the file LICENSE for licensing terms.

package server

import (
	"bytes"
	"context"
	"encoding/hex"
	"fmt"
	"reflect"
	"strings"

	"github.com/ava-labs/avalanche-rs/avalanchego-conformance/rpcpb"
	"github.com/ava-labs/avalanchego/ids"
	"github.com/ava-labs/avalanchego/utils/cb58"
	"github.com/ava-labs/avalanchego/utils/constants"
	"github.com/ava-labs/avalanchego/utils/crypto/bls"
	"github.com/ava-labs/avalanchego/utils/crypto/secp256k1"
	"github.com/ava-labs/avalanchego/utils/formatting/address"
	"github.com/ava-labs/avalanchego/utils/hashing"
	eth_crypto "github.com/ethereum/go-ethereum/crypto"
	"go.uber.org/zap"
)

func (s *server) CertificateToNodeId(ctx context.Context, req *rpcpb.CertificateToNodeIdRequest) (*rpcpb.CertificateToNodeIdResponse, error) {
	zap.L().Debug("received CertificateToNodeId request", zap.Int("cert-size", len(req.Certificate)))

	nodeID, err := ids.ToShortID(hashing.PubkeyBytesToAddress(req.Certificate))
	if err != nil {
		return nil, err
	}

	resp := &rpcpb.CertificateToNodeIdResponse{ExpectedNodeId: nodeID[:], Success: true}
	if !bytes.Equal(nodeID[:], req.NodeId) {
		resp.Message = fmt.Sprintf("expected node ID %s", nodeID.String())
		resp.Success = false
	}

	return resp, nil
}

func (s *server) Secp256K1RecoverHashPublicKey(ctx context.Context, req *rpcpb.Secp256K1RecoverHashPublicKeyRequest) (*rpcpb.Secp256K1RecoverHashPublicKeyResponse, error) {
	zap.L().Debug("received Secp256K1RecoverHashPublicKey request")

	resp := &rpcpb.Secp256K1RecoverHashPublicKeyResponse{Success: true}
	pubkey, err := s.secpFactory.RecoverHashPublicKey(req.Message, req.Signature)
	if err != nil {
		resp.Message = fmt.Sprintf("failed RecoverHashPublicKey %v", err)
		resp.Success = false
		return resp, nil
	}

	resp.ExpectedPublicKeyShortIdCb58 = pubkey.Address().String()
	if pubkey.Address().String() != req.PublicKeyShortIdCb58 {
		resp.Message = fmt.Sprintf("expected recovered public key in short id + cb58 %s, but instead got %s", pubkey.Address().String(), req.PublicKeyShortIdCb58)
		resp.Success = false
	}

	return resp, nil
}

func (s *server) Secp256K1Info(ctx context.Context, req *rpcpb.Secp256K1InfoRequest) (*rpcpb.Secp256K1InfoResponse, error) {
	zap.L().Debug("received Secp256K1Info request")

	// based on the received cb58-encoded key, create its own key info using avalanchego
	privKeyInfo := &rpcpb.Secp256K1Info{KeyType: "hot", ChainAddresses: make(map[uint32]*rpcpb.ChainAddresses)}
	privKey, err := decodePrivateKey(req.Secp256K1Info.PrivateKeyCb58)
	if err != nil {
		return nil, err
	}
	privKeyInfo.PrivateKeyCb58, err = encodePrivateKey(privKey)
	if err != nil {
		return nil, err
	}
	privKeyInfo.PrivateKeyHex = hex.EncodeToString(privKey.Bytes())

	for networkID := range req.Secp256K1Info.ChainAddresses {
		xAddr, err := encodeAddr(privKey, "X", constants.GetHRP(uint32(networkID)))
		if err != nil {
			return nil, err
		}
		pAddr, err := encodeAddr(privKey, "P", constants.GetHRP(uint32(networkID)))
		if err != nil {
			return nil, err
		}
		privKeyInfo.ChainAddresses[networkID] = &rpcpb.ChainAddresses{
			X: xAddr,
			P: pAddr,
		}
	}
	privKeyInfo.ShortAddress = encodeShortAddr(privKey)
	privKeyInfo.EthAddress = encodeEthAddr(privKey)

	resp := &rpcpb.Secp256K1InfoResponse{
		ExpectedSecp256K1Info: privKeyInfo,
		Success:               true,
	}
	if req.Secp256K1Info.PrivateKeyCb58 != privKeyInfo.PrivateKeyCb58 {
		resp.Message += "req.Secp256K1Info.PrivateKeyCb58 != ExpectedSecp256K1Info.PrivateKeyCb58"
		resp.Success = false
	}
	if strings.TrimPrefix(req.Secp256K1Info.PrivateKeyHex, "0x") != strings.TrimPrefix(privKeyInfo.PrivateKeyHex, "0x") {
		if resp.Message != "" {
			resp.Message += "; "
		}
		resp.Message += "req.Secp256K1Info.PrivateKeyHex != ExpectedSecp256K1Info.PrivateKeyHex"
		resp.Success = false
	}
	if !reflect.DeepEqual(req.Secp256K1Info.ChainAddresses, privKeyInfo.ChainAddresses) {
		if resp.Message != "" {
			resp.Message += "; "
		}
		resp.Message += "req.Secp256K1Info.ChainAddresses != ExpectedSecp256K1Info.ChainAddresses"
		resp.Success = false
	}
	if req.Secp256K1Info.ShortAddress != privKeyInfo.ShortAddress {
		if resp.Message != "" {
			resp.Message += "; "
		}
		resp.Message += "req.Secp256K1Info.ShortAddress != ExpectedSecp256K1Info.ShortAddress"
		resp.Success = false
	}

	if req.Secp256K1Info.EthAddress != privKeyInfo.EthAddress {
		if resp.Message != "" {
			resp.Message += "; "
		}
		resp.Message += fmt.Sprintf("req.Secp256K1Info.EthAddress %q != ExpectedSecp256K1Info.EthAddress %q", req.Secp256K1Info.EthAddress, privKeyInfo.EthAddress)
		resp.Success = false
	}

	if resp.Success {
		resp.Message = "SUCCESS"
	}
	return resp, nil
}

const privKeyEncPfx = "PrivateKey-"

func encodePrivateKey(pk *secp256k1.PrivateKey) (string, error) {
	privKeyRaw := pk.Bytes()
	enc, err := cb58.Encode(privKeyRaw)
	if err != nil {
		return "", err
	}
	return privKeyEncPfx + enc, nil
}

func decodePrivateKey(enc string) (*secp256k1.PrivateKey, error) {
	rawPk := strings.Replace(enc, privKeyEncPfx, "", 1)

	// ref. "formatting.Decode(formatting.CB58"
	skBytes, err := cb58.Decode(rawPk)
	if err != nil {
		return nil, err
	}

	keyFactory := new(secp256k1.Factory)
	return keyFactory.ToPrivateKey(skBytes)
}

func encodeAddr(pk *secp256k1.PrivateKey, chainIDAlias string, hrp string) (string, error) {
	pubBytes := pk.PublicKey().Address().Bytes()
	return address.Format(chainIDAlias, hrp, pubBytes)
}

func encodeShortAddr(pk *secp256k1.PrivateKey) string {
	return pk.PublicKey().Address().String()
}

func encodeEthAddr(pk *secp256k1.PrivateKey) string {
	ethAddr := eth_crypto.PubkeyToAddress(pk.ToECDSA().PublicKey)
	return ethAddr.String()
}

func (s *server) BlsSignature(ctx context.Context, req *rpcpb.BlsSignatureRequest) (*rpcpb.BlsSignatureResponse, error) {
	zap.L().Debug("received BlsSignature request")

	sk, err := bls.SecretKeyFromBytes(req.PrivateKey)
	if err != nil {
		return nil, err
	}

	pubkey, err := bls.PublicKeyFromBytes(req.PublicKey)
	if err != nil {
		return nil, err
	}

	resp := &rpcpb.BlsSignatureResponse{
		Success: true,
	}

	zap.L().Info("verifying Signature")
	sig := bls.Sign(sk, req.Message)
	if !bls.Verify(pubkey, sig, req.Message) {
		if resp.Message != "" {
			resp.Message += ", "
		}
		resp.Message += "bls.Verify failed from derived signature"
		resp.Success = false
	}

	zap.L().Info("verifying Signature by loading")
	loadedSig, err := bls.SignatureFromBytes(req.Signature)
	if err != nil {
		return nil, err
	}
	if !bls.Verify(pubkey, loadedSig, req.Message) {
		if resp.Message != "" {
			resp.Message += ", "
		}
		resp.Message += "bls.Verify failed from loaded signature"
		resp.Success = false
	}

	zap.L().Info("verifying SignatureProofOfPossession")
	sigPoP := bls.SignProofOfPossession(sk, req.Message)
	if !bls.VerifyProofOfPossession(pubkey, sigPoP, req.Message) {
		if resp.Message != "" {
			resp.Message += ", "
		}
		resp.Message += "bls.Verify failed from derived proof-of-possession signature"
		resp.Success = false
	}

	zap.L().Info("verifying SignatureProofOfPossession by loading")
	loadedSigPoP, err := bls.SignatureFromBytes(req.SignatureProofOfPossession)
	if err != nil {
		return nil, err
	}
	if !bls.VerifyProofOfPossession(pubkey, loadedSigPoP, req.Message) {
		if resp.Message != "" {
			resp.Message += ", "
		}
		resp.Message += "bls.Verify failed from loaded proof-of-possession signature"
		resp.Success = false
	}

	if resp.Success {
		resp.Message = "SUCCESS"
	}
	return resp, nil
}
