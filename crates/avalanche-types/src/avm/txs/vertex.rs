//! Vertex types used in the Avalanche X-chain.
use crate::{errors::Result, ids, packer::Packer, txs::raw};

/// Vertex represents a set of transactions for Avalanche X-chain.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/avalanche/vertex#Build>
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Vertex {
    pub codec_version: u16,
    pub chain_id: ids::Id,
    pub height: u64,
    pub epoch: u32,
    pub parent_ids: Vec<ids::Id>,
    pub txs: Vec<Vec<u8>>,
}

impl Packer {
    /// Encodes vertex fields with codec version and packer.
    ///
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/avalanche/vertex#Build>
    pub fn pack_vertex(&self, vtx: &mut Vertex) -> Result<()> {
        // sort "parent_ids"
        // ref. "ids.SortIDs"
        vtx.parent_ids.sort();

        // sort "txs" by SHA256 hashes
        // ref. "SortHashOf"
        vtx.txs.sort_by(|a, b| {
            (raw::Data::from_slice(a.as_ref())).cmp(&raw::Data::from_slice(b.as_ref()))
        });

        self.pack_u16(vtx.codec_version)?;
        self.pack_bytes(vtx.chain_id.as_ref())?;
        self.pack_u64(vtx.height)?;
        self.pack_u32(vtx.epoch)?;

        self.pack_u32(vtx.parent_ids.len() as u32)?;
        for id in vtx.parent_ids.iter() {
            self.pack_bytes(id.as_ref())?;
        }

        self.pack_u32(vtx.txs.len() as u32)?;
        for tx in vtx.txs.iter() {
            self.pack_bytes_with_header(tx.as_ref())?;
        }

        Ok(())
    }

    /// Unpacks the vertex.
    ///
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/avalanche/vertex#Build>
    pub fn unpack_vertex(&self) -> Result<Vertex> {
        let codec_version = self.unpack_u16()?;

        let chain_id = self.unpack_bytes(ids::LEN)?;
        let chain_id = ids::Id::from_slice(chain_id.as_ref());

        let height = self.unpack_u64()?;
        let epoch = self.unpack_u32()?;

        let parent_ids_size = self.unpack_u32()?;
        let mut parent_ids: Vec<ids::Id> = Vec::new();
        for _ in 0..parent_ids_size {
            let parent_id = self.unpack_bytes(ids::LEN)?;
            let parent_id = ids::Id::from_slice(parent_id.as_ref());
            parent_ids.push(parent_id);
        }

        let txs_size = self.unpack_u32()?;
        let mut txs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..txs_size {
            let tx_size = self.unpack_u32()?;
            let tx = self.unpack_bytes(tx_size as usize)?;
            txs.push(tx);
        }

        Ok(Vertex {
            codec_version,
            chain_id,
            height,
            epoch,
            parent_ids,
            txs,
        })
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- avm::txs::vertex::test_pack_and_unpack --exact --show-output
#[test]
fn test_pack_and_unpack() {
    use bytes::BytesMut;

    let mut vtx = Vertex {
        codec_version: 0_u16,
        chain_id: ids::Id::from_slice(&<Vec<u8>>::from([
            0x3d, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
            0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
            0xc3, 0x2b, 0xff, 0x1d, 0x6d, 0xec, 0x47, 0x2b, 0x25, 0xcf, //
            0x59, 0xa7,
        ])),
        height: 1234567_u64,
        epoch: 0,

        // to be sorted
        parent_ids: vec![
            ids::Id::from_slice(&<Vec<u8>>::from([
                0x03, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
                0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
                0xc3, 0x2b, 0xff, 0x1d, 0x6d, 0xec, 0x47, 0x2b, 0x25, 0xcf, //
                0x59, 0xa7,
            ])),
            ids::Id::from_slice(&<Vec<u8>>::from([
                0x02, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
                0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
                0xc3, 0x2b, 0xff, 0x1d, 0x6d, 0xec, 0x47, 0x2b, 0x25, 0xcf, //
                0x59, 0xa7,
            ])),
            ids::Id::from_slice(&<Vec<u8>>::from([
                0x01, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
                0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
                0xc3, 0x2b, 0xff, 0x1d, 0x6d, 0xec, 0x47, 0x2b, 0x25, 0xcf, //
                0x59, 0xa7,
            ])),
        ],

        // to be sorted
        txs: vec![
            <Vec<u8>>::from([0x01]),
            <Vec<u8>>::from([0x02]),
            <Vec<u8>>::from([0x03]),
        ],
    };

    let packer = Packer::with_max_size(1024);
    packer.pack_vertex(&mut vtx).unwrap();

    let vtx_sorted = Vertex {
        codec_version: 0_u16,
        chain_id: ids::Id::from_slice(&<Vec<u8>>::from([
            0x3d, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
            0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
            0xc3, 0x2b, 0xff, 0x1d, 0x6d, 0xec, 0x47, 0x2b, 0x25, 0xcf, //
            0x59, 0xa7,
        ])),
        height: 1234567_u64,
        epoch: 0,

        // sorted
        parent_ids: vec![
            ids::Id::from_slice(&<Vec<u8>>::from([
                0x01, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
                0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
                0xc3, 0x2b, 0xff, 0x1d, 0x6d, 0xec, 0x47, 0x2b, 0x25, 0xcf, //
                0x59, 0xa7,
            ])),
            ids::Id::from_slice(&<Vec<u8>>::from([
                0x02, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
                0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
                0xc3, 0x2b, 0xff, 0x1d, 0x6d, 0xec, 0x47, 0x2b, 0x25, 0xcf, //
                0x59, 0xa7,
            ])),
            ids::Id::from_slice(&<Vec<u8>>::from([
                0x03, 0x0a, 0xd1, 0x2b, 0x8e, 0xe8, 0x92, 0x8e, 0xdf, 0x24, //
                0x8c, 0xa9, 0x1c, 0xa5, 0x56, 0x00, 0xfb, 0x38, 0x3f, 0x07, //
                0xc3, 0x2b, 0xff, 0x1d, 0x6d, 0xec, 0x47, 0x2b, 0x25, 0xcf, //
                0x59, 0xa7,
            ])),
        ],

        // sorted
        txs: vec![
            <Vec<u8>>::from([0x03]),
            <Vec<u8>>::from([0x01]),
            <Vec<u8>>::from([0x02]),
        ],
    };
    assert!(vtx == vtx_sorted);

    let b = packer.take_bytes();
    let b = BytesMut::from(&b[..]);

    let packer = Packer::with_max_size(0);
    packer.set_bytes(&b);

    let vtx_unpacked = packer.unpack_vertex().unwrap();
    assert!(vtx == vtx_unpacked);
}
