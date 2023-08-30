use std::io::{self, Error, ErrorKind};

use crate::{ids, message, proto::pb::p2p};
use prost::Message as ProstMessage;

#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub msg: p2p::Version,
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
            msg: p2p::Version {
                network_id: 0,
                my_time: 0,
                ip_addr: prost::bytes::Bytes::new(),
                ip_port: 0,
                my_version: String::new(),
                my_version_time: 0,
                sig: prost::bytes::Bytes::new(),
                tracked_subnets: Vec::new(),
            },
            gzip_compress: false,
        }
    }

    #[must_use]
    pub fn network_id(mut self, network_id: u32) -> Self {
        self.msg.network_id = network_id;
        self
    }

    #[must_use]
    pub fn my_time(mut self, my_time: u64) -> Self {
        self.msg.my_time = my_time; // local time in unix second.
        self
    }

    #[must_use]
    pub fn ip_addr(mut self, ip_addr: std::net::IpAddr) -> Self {
        self.msg.ip_addr = prost::bytes::Bytes::from(super::ip_addr_to_bytes(ip_addr));
        self
    }

    #[must_use]
    pub fn ip_port(mut self, ip_port: u32) -> Self {
        self.msg.ip_port = ip_port;
        self
    }

    #[must_use]
    pub fn my_version(mut self, my_version: String) -> Self {
        self.msg.my_version = my_version;
        self
    }

    #[must_use]
    pub fn my_version_time(mut self, my_version_time: u64) -> Self {
        self.msg.my_version_time = my_version_time;
        self
    }

    #[must_use]
    pub fn sig(mut self, sig: Vec<u8>) -> Self {
        self.msg.sig = prost::bytes::Bytes::from(sig);
        self
    }

    #[must_use]
    pub fn tracked_subnets(mut self, tracked_subnets: Vec<ids::Id>) -> Self {
        let mut tracked_subnet_bytes: Vec<prost::bytes::Bytes> =
            Vec::with_capacity(tracked_subnets.len());
        for id in tracked_subnets.iter() {
            tracked_subnet_bytes.push(prost::bytes::Bytes::from(id.to_vec()));
        }
        self.msg.tracked_subnets = tracked_subnet_bytes;
        self
    }

    #[must_use]
    pub fn gzip_compress(mut self, gzip_compress: bool) -> Self {
        self.gzip_compress = gzip_compress;
        self
    }

    pub fn serialize(&self) -> io::Result<Vec<u8>> {
        let msg = p2p::Message {
            message: Some(p2p::message::Message::Version(self.msg.clone())),
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
                "version compression saved {} bytes",
                uncompressed_len - compressed_len
            );
        } else {
            log::debug!(
                "version compression added {} byte(s)",
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
            p2p::message::Message::Version(msg) => Ok(Message {
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
                    p2p::message::Message::Version(msg) => Ok(Message {
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

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- message::version::test_message --exact --show-output
#[test]
fn test_message() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let msg1_with_no_compression = Message::default()
        .network_id(100000)
        .my_time(77777777)
        .ip_addr(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
        .ip_port(8080)
        .my_version(String::from("v1.2.3"))
        .my_version_time(1234567)
        .sig(random_manager::secure_bytes(65).unwrap())
        .tracked_subnets(vec![
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
            ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ]);

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
