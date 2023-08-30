use std::io::{self, Error, ErrorKind};

use crate::{message, proto::pb::p2p};
use prost::Message as ProstMessage;

#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub msg: p2p::Ping,
    pub gzip_compress: bool,
}

impl Default for Message {
    fn default() -> Self {
        Self::default()
    }
}

impl Message {
    pub fn default() -> Self {
        Message {
            msg: p2p::Ping {},
            gzip_compress: false,
        }
    }

    #[must_use]
    pub fn gzip_compress(mut self, gzip_compress: bool) -> Self {
        self.gzip_compress = gzip_compress;
        self
    }

    pub fn serialize(&self) -> io::Result<Vec<u8>> {
        let msg = p2p::Message {
            message: Some(p2p::message::Message::Ping(self.msg.clone())),
        };
        let encoded = ProstMessage::encode_to_vec(&msg);
        if !self.gzip_compress {
            return Ok(encoded);
        }

        let uncompressed_len = encoded.len();
        let compressed = message::compress::pack_gzip(&encoded)?;
        let msg = p2p::Message {
            message: Some(p2p::message::Message::CompressedGzip(
                prost::bytes::Bytes::from(compressed),
            )),
        };

        let compressed_len = msg.encoded_len();
        if uncompressed_len > compressed_len {
            log::debug!(
                "ping compression saved {} bytes",
                uncompressed_len - compressed_len
            );
        } else {
            log::debug!(
                "ping compression added {} byte(s)",
                compressed_len - uncompressed_len
            );
        }

        Ok(ProstMessage::encode_to_vec(&msg))
    }

    pub fn deserialize(d: impl AsRef<[u8]>) -> io::Result<Self> {
        let buf = bytes::Bytes::from(d.as_ref().to_vec());
        let p2p_msg: p2p::Message = ProstMessage::decode(buf).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("failed prost::Message::decode '{}'", e),
            )
        })?;

        match p2p_msg.message.unwrap() {
            // was not compressed
            p2p::message::Message::Ping(msg) => Ok(Message {
                msg,
                gzip_compress: false,
            }),

            // was compressed, so need decompress first
            p2p::message::Message::CompressedGzip(msg) => {
                let decompressed = message::compress::unpack_gzip(msg.as_ref())?;
                let decompressed_msg: p2p::Message =
                    ProstMessage::decode(prost::bytes::Bytes::from(decompressed)).map_err(|e| {
                        Error::new(
                            ErrorKind::InvalidData,
                            format!("failed prost::Message::decode '{}'", e),
                        )
                    })?;
                match decompressed_msg.message.unwrap() {
                    p2p::message::Message::Ping(msg) => Ok(Message {
                        msg,
                        gzip_compress: false,
                    }),
                    _ => Err(Error::new(
                        ErrorKind::InvalidInput,
                        "unknown message type after decompress",
                    )),
                }
            }

            // unknown message enum
            _ => Err(Error::new(ErrorKind::InvalidInput, "unknown message type")),
        }
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- message::ping::test_message --exact --show-output
#[test]
fn test_message() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let msg1_with_no_compression = Message::default();

    let data1 = msg1_with_no_compression.serialize().unwrap();
    let msg1_with_no_compression_deserialized = Message::deserialize(&data1).unwrap();
    assert_eq!(
        msg1_with_no_compression,
        msg1_with_no_compression_deserialized
    );

    let msg2_with_compression = msg1_with_no_compression.clone().gzip_compress(true);
    assert_ne!(msg1_with_no_compression, msg2_with_compression);

    let data2 = msg2_with_compression.serialize().unwrap();
    let msg2_with_compression_deserialized = Message::deserialize(&data2).unwrap();
    assert_eq!(msg1_with_no_compression, msg2_with_compression_deserialized);
}
