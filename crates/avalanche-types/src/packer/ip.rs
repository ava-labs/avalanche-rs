use std::{
    convert::TryInto,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use crate::{
    errors::Result,
    packer::{self, Packer},
};

/// All IPs (either IPv4 or IPv6) are represented as a 16-byte (IPv6) array.
/// ref. "go/net/IP"
pub const IP_ADDR_LEN: usize = 16;

/// number of bytes per IP + port
pub const IP_LEN: usize = IP_ADDR_LEN + packer::U16_LEN;

impl Packer {
    /// Writes the "IP" value at the offset in 16-byte representation and increments the offset afterwards.
    /// ref. "avalanchego/utils/wrappers.Packer.PackIP"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.PackIP>
    /// ref. <https://doc.rust-lang.org/std/net/enum.IpAddr.html>
    /// ref. <https://doc.rust-lang.org/std/net/struct.Ipv4Addr.html>
    /// ref. <https://doc.rust-lang.org/std/net/struct.Ipv6Addr.html>
    pub fn pack_ip(&self, ip_addr: IpAddr, port: u16) -> Result<()> {
        let ip_bytes = match ip_addr {
            IpAddr::V4(v) => {
                // "avalanchego" encodes IPv4 address as it is
                // (not compatible with IPv6, e.g., prepends 2 "0xFF"s as in Rust)
                let octets = v.octets();
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, octets[0], octets[1], octets[2], octets[3],
                ]
            }
            IpAddr::V6(v) => v.octets(),
        };
        self.pack_bytes(&ip_bytes)?;
        self.pack_u16(port)
    }

    /// Unpacks the "IP" in the "offset" position,
    /// and advances the cursor and offset.
    /// ref. "avalanchego/utils/wrappers.Packer.UnpackIP"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.UnpackIP>
    pub fn unpack_ip(&self) -> Result<(IpAddr, u16)> {
        let ip = self.unpack_bytes(IP_ADDR_LEN)?;
        let ip_array: [u8; IP_ADDR_LEN] = fix_vector_size(ip);

        let ip = {
            // check if it were IPv4 or 6
            if all_zeroes(&ip_array[..12]) && ip_array[12] > 0 {
                IpAddr::V4(Ipv4Addr::new(
                    ip_array[12],
                    ip_array[13],
                    ip_array[14],
                    ip_array[15],
                ))
            } else {
                let ip_u128 = u128::from_be_bytes(ip_array);
                let ipv6 = Ipv6Addr::from(ip_u128);
                IpAddr::V6(ipv6)
            }
        };
        let port = self.unpack_u16()?;

        Ok((ip, port))
    }

    /// Writes the list of "IP" values at the offset in 16-byte representation
    /// and increments the offset afterwards.
    /// ref. "avalanchego/utils/wrappers.Packer.PackIPs"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.PackIPs>
    pub fn pack_ips(&self, ips: &[(IpAddr, u16)]) -> Result<()> {
        let n = ips.len();
        self.pack_u32(n as u32)?;
        for ip in ips.iter() {
            self.pack_ip(ip.0, ip.1)?;
        }
        Ok(())
    }

    /// Unpacks the list of "IP"s in the "offset" position,
    /// and advances the cursor and offset.
    /// ref. "avalanchego/utils/wrappers.Packer.UnpackIPs"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.UnpackIPs>
    pub fn unpack_ips(&self) -> Result<Vec<(IpAddr, u16)>> {
        let n = self.unpack_u32()?;
        let mut rs: Vec<(IpAddr, u16)> = Vec::new();
        for _ in 0..n {
            let b = self.unpack_ip()?;
            rs.push(b);
        }
        Ok(rs)
    }
}

fn fix_vector_size<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("expected vec length {} but {}", N, v.len()))
}

/// ref. <https://doc.rust-lang.org/std/primitive.slice.html#method.align_to>
fn all_zeroes(d: &[u8]) -> bool {
    let (prefix, aligned, suffix) = unsafe { d.align_to::<u128>() };
    prefix.iter().all(|&x| x == 0)
        && suffix.iter().all(|&x| x == 0)
        && aligned.iter().all(|&x| x == 0)
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::ip::test_pack_and_unpack --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackIPCert"
#[test]
fn test_pack_and_unpack() {
    use bytes::BytesMut;
    use std::cell::Cell;

    let packer = Packer {
        max_size: IP_LEN,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };

    packer
        .pack_ip(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
        .unwrap();
    assert_eq!(packer.bytes_len(), IP_LEN);

    // beyond max size
    assert!(packer
        .pack_ip(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
        .is_err());

    let b = packer.take_bytes();
    let expected: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1, 31, 144];
    assert_eq!(&b[..], &expected[..]);

    // test unpack
    let b = BytesMut::from(&expected[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    let b = packer.unpack_ip().unwrap();
    assert_eq!(packer.get_offset(), IP_LEN);

    assert_eq!(b.0, IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(b.1, 8080);

    assert!(packer.unpack_ip().is_err());

    let packer = Packer::new_with_header(4 + IP_LEN, 0);
    packer
        .pack_ip(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
        .unwrap();
    let expected: Vec<u8> = vec![
        0x00, 0x00, 0x00, 18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1, 31, 144,
    ];
    assert_eq!(&packer.take_bytes()[..], &expected[..]);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::ip::test_packs_and_unpacks --exact --show-output
#[test]
fn test_packs_and_unpacks() {
    use bytes::BytesMut;
    use std::cell::Cell;

    let packer = Packer {
        max_size: packer::U32_LEN + IP_LEN * 3,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };

    packer
        .pack_ips(&[
            (IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
            (IpAddr::V6(Ipv6Addr::LOCALHOST), 8081),
            (IpAddr::V6(Ipv6Addr::new(1, 1, 1, 1, 1, 1, 1, 1)), 80),
        ])
        .unwrap();

    let b = packer.take_bytes();
    let expected: Vec<u8> = vec![
        0, 0, 0, 3, // length of IPs
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1, 31,
        144, // IpAddr::V4(Ipv4Addr::LOCALHOST), 8080
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 31,
        145, // IpAddr::V6(Ipv6Addr::LOCALHOST), 8081
        0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
        80, // IpAddr::V6(Ipv6Addr::new(1, 1, 1, 1, 1, 1, 1, 1)), 80
    ];
    assert_eq!(&b[..], &expected[..]);

    // test unpack
    let b = BytesMut::from(&expected[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    let b = packer.unpack_ips().unwrap();
    assert_eq!(packer.get_offset(), packer::U32_LEN + IP_LEN * 3);

    assert_eq!(b[0].0, IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(b[0].1, 8080);

    assert_eq!(b[1].0, IpAddr::V6(Ipv6Addr::LOCALHOST));
    assert_eq!(b[1].1, 8081);

    assert_eq!(b[2].0, IpAddr::V6(Ipv6Addr::new(1, 1, 1, 1, 1, 1, 1, 1)));
    assert_eq!(b[2].1, 80);

    let packer = Packer::new_with_header(1024, 0);
    packer
        .pack_ips(&[
            (IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
            (IpAddr::V6(Ipv6Addr::LOCALHOST), 8081),
            (IpAddr::V6(Ipv6Addr::new(1, 1, 1, 1, 1, 1, 1, 1)), 80),
        ])
        .unwrap();
    let expected: Vec<u8> = vec![
        0x00, 0x00, 0x00, 58, // length of message
        0, 0, 0, 3, // length of IPs
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1, 31,
        144, // IpAddr::V4(Ipv4Addr::LOCALHOST), 8080
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 31,
        145, // IpAddr::V6(Ipv6Addr::LOCALHOST), 8081
        0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
        80, // IpAddr::V6(Ipv6Addr::new(1, 1, 1, 1, 1, 1, 1, 1)), 80
    ];
    assert_eq!(&packer.take_bytes()[..], &expected[..]);
}
