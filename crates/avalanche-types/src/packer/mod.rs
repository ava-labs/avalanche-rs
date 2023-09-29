//! Low-level byte-packing utilities.
pub mod ip;

use std::{cell::Cell, u16};

use crate::errors::{Error, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub const MAX_STR_LEN: u16 = u16::MAX - 1;

/// number of bytes per byte
/// 8-bit unsigned integer, so the length is 1-byte
pub const BYTE_LEN: usize = 1;
pub const BYTE_SENTINEL: u8 = 0;

/// number of bytes per short
/// 16-bit unsigned integer, so the length is 2-byte
pub const U16_LEN: usize = 2;
pub const U16_SENTINEL: u16 = 0;

/// number of bytes per int
/// 32-bit unsigned integer, so the length is 4-byte
pub const U32_LEN: usize = 4;
pub const U32_SENTINEL: u32 = 0;

/// number of bytes per long
/// 64-bit unsigned integer, so the length is 8-byte
pub const U64_LEN: usize = 8;
pub const U64_SENTINEL: u64 = 0;

/// number of bytes per bool
pub const BOOL_LEN: usize = 1;
pub const BOOL_SENTINEL: bool = false;

/// Packer packs and unpacks the underlying bytes array.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer>
/// ref. <https://doc.rust-lang.org/1.7.0/book/mutability.html>
/// ref. <https://doc.rust-lang.org/std/cell/struct.Cell.html>
pub struct Packer {
    /// largest allowed size of expanding the byte array
    max_size: usize,
    /// current byte array
    bytes: Cell<BytesMut>,
    /// If true, then the first 4-bytes in "take_bytes"
    /// encodes the length of the message.
    /// The returned bytes length will be 4-byte + message.
    header: bool,
    /// offset that is being written to in the byte array
    offset: Cell<usize>,
}

impl Packer {
    pub fn new(max_size: usize, initial_cap: usize) -> Self {
        let bytes = Cell::new(BytesMut::with_capacity(initial_cap));
        Self {
            max_size,
            bytes,
            header: false,
            offset: Cell::new(0),
        }
    }

    /// Creates a new Packer with 32-bit message length header.
    pub fn new_with_header(max_size: usize, initial_cap: usize) -> Self {
        let mut b = BytesMut::with_capacity(initial_cap);
        b.put_slice(&[0x00, 0x00, 0x00, 0x00]);
        let bytes = Cell::new(b);
        let offset = Cell::new(4);
        Self {
            max_size,
            bytes,
            header: true,
            offset,
        }
    }

    /// Create a new packer from the existing bytes.
    /// Resets the offset to the end of the existing bytes.
    pub fn load_bytes_for_pack(max_size: usize, b: &[u8]) -> Self {
        Self {
            max_size,
            bytes: Cell::new(BytesMut::from(b)),
            header: false,
            offset: Cell::new(b.len()),
        }
    }

    /// Create a new packer from the existing bytes.
    /// Resets the offset to the beginning of the existing bytes.
    pub fn load_bytes_for_unpack(max_size: usize, b: &[u8]) -> Self {
        Self {
            max_size,
            bytes: Cell::new(BytesMut::from(b)),
            header: false,
            offset: Cell::new(0),
        }
    }

    /// Returns the current bytes array as an immutable bytes array.
    /// If the packer header is set to "true", the first 4-byte represents
    /// the message length in the big-endian order. The returned bytes length
    /// will be 4-byte + message.
    ///
    /// Be cautious! Once bytes are taken out, the "bytes" field is set to default (empty).
    /// To continue to write to bytes, remember to put it back with "set_bytes"
    /// because "bytes.take" leaves the field as "Default::default()".
    /// TODO: make sure this does shallow copy!
    pub fn take_bytes(&self) -> Bytes {
        let mut b = self.bytes.take();
        let n = b.len();
        if self.header {
            assert!(n >= 4);
            let msg_length = (n - 4) as u32;

            let header = msg_length.to_be_bytes();
            assert!(header.len() == 4);
            b[0] = header[0];
            b[1] = header[1];
            b[2] = header[2];
            b[3] = header[3];
        }
        b.copy_to_bytes(n)
    }

    /// Sets the current bytes array as an immutable bytes array.
    /// Useful to reuse packer after calling "take_bytes", which
    /// makes the "bytes" field default (empty).
    pub fn set_bytes(&self, b: &[u8]) {
        self.bytes.set(BytesMut::from(b));
    }

    /// Updates the "offset" field.
    fn set_offset(&self, offset: usize) {
        self.offset.set(offset)
    }

    /// Returns the "offset" value.
    pub fn get_offset(&self) -> usize {
        // "usize" implements "Copy" so just use "get" on "Cell"
        // ref. https://doc.rust-lang.org/std/cell/struct.Cell.html#impl-1
        self.offset.get()
    }

    /// Returns the current length of the bytes array.
    pub fn bytes_len(&self) -> usize {
        // "BytesMut" does not implement "Copy" so take/update/set it back
        // ref. https://doc.rust-lang.org/std/cell/struct.Cell.html#impl-1
        let b = self.bytes.take();
        let n = b.len();
        self.bytes.set(b);
        n
    }

    /// Returns the current capacity of the bytes array.
    pub fn bytes_cap(&self) -> usize {
        // "BytesMut" does not implement "Copy" so take/update/set it back
        // ref. https://doc.rust-lang.org/std/cell/struct.Cell.html#impl-1
        let b = self.bytes.take();
        let n = b.capacity();
        self.bytes.set(b);
        n
    }

    /// Truncates the bytes array while retaining the underlying capacity.
    fn truncate_bytes_with_length(&self, len: usize) {
        // "BytesMut" does not implement "Copy" so take/update/set it back
        // remember to put it back -- "take" leaves the field as "Default::default()"
        // ref. https://doc.rust-lang.org/std/cell/struct.Cell.html#impl-1
        let mut b = self.bytes.take();
        b.truncate(len);
        self.bytes.set(b);
    }

    /// Reserves the bytes array while retaining the underlying length.
    fn reserve_bytes_with_length(&self, len: usize) {
        // "BytesMut" does not implement "Copy" so take/update/set it back
        // remember to put it back -- "take" leaves the field as "Default::default()"
        // ref. https://doc.rust-lang.org/std/cell/struct.Cell.html#impl-1
        let mut b = self.bytes.take();
        b.reserve(len);
        self.bytes.set(b);
    }

    /// Ensures the remaining capacity of the bytes array
    /// so it can write "n" bytes to the array.
    /// ref. "avalanchego/utils/wrappers.Packer.Expand"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.Expand>
    pub fn expand(&self, n: usize) -> Result<()> {
        // total number of bytes that must be remained in the bytes array
        let needed_size = self.get_offset() + n;

        // already has sufficient length
        // thus no need to check max_size
        if needed_size <= self.bytes_len() {
            return Ok(());
        }

        // byte slice would cause it to grow too large (out of bounds)
        if needed_size > self.max_size {
            return Err(Error::Other {
                message: format!(
                    "needed_size {} exceeds max_size {}",
                    needed_size, self.max_size
                ),
                retryable: false,
            });
        }

        // has sufficient capacity to lengthen it without mem alloc
        let bytes_cap = self.bytes_cap();
        if needed_size <= bytes_cap {
            self.truncate_bytes_with_length(needed_size);
            return Ok(());
        }

        // "avalanchego/utils/wrappers.Packer.Expand" is different in that
        // it uses "resize" to fill in the array with zero values.
        // As long as we maintain the "offset", it does not change the underlying
        // packing algorithm, thus compatible.
        self.reserve_bytes_with_length(needed_size);
        Ok(())
    }

    /// Returns an error if the packer has insufficient length for the input size.
    /// ref. "avalanchego/utils/wrappers.Packer.CheckSpace"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.CheckSpace>
    fn check_remaining_unpack(&self, bytes_to_read: usize) -> Result<()> {
        let needed_size = self.get_offset() + bytes_to_read;
        let bytes_n = self.bytes_len();
        if needed_size > bytes_n {
            return Err(Error::Other {
                message:  format!(
                    "bad length to read; offset + bytes ({}) to read exceeds current total bytes size {}",
                    needed_size,
                    bytes_n
                ), // ref. "errBadLength"
                retryable: false,
            });
        };
        Ok(())
    }

    /// Writes the "u8" value at the offset and increments the offset afterwards.
    /// ref. "avalanchego/utils/wrappers.Packer.PackByte"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.PackByte>
    pub fn pack_byte(&self, v: u8) -> Result<()> {
        self.expand(BYTE_LEN)?;

        let offset = self.get_offset();
        let mut b = self.bytes.take();

        // assume "offset" is not updated by the other "unpack*"
        // thus no need to keep internal cursor in sync with "offset"
        // unsafe { b.advance_mut(offset) };

        // writes an unsigned 8-bit integer
        b.put_u8(v);

        // remember to put it back -- "take" leaves the field as "Default::default()"
        self.bytes.set(b);

        // "put_u8" already advances the current position by BYTE_LEN
        // thus no need for "unsafe { b.advance_mut(offset + BYTE_LEN) };"
        // ref. https://docs.rs/bytes/latest/bytes/buf/trait.BufMut.html#method.put_u8
        self.set_offset(offset + BYTE_LEN);
        Ok(())
    }

    /// Unpacks the byte in the "offset" position,
    /// and advances the cursor and offset.
    /// ref. "avalanchego/utils/wrappers.Packer.UnpackByte"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.UnpackByte>
    pub fn unpack_byte(&self) -> Result<u8> {
        self.check_remaining_unpack(BYTE_LEN)?;

        let offset = self.get_offset();
        let b = self.bytes.take();

        let p = &b[offset];
        let v = *p;

        // remember to put it back -- "take" leaves the field as "Default::default()"
        self.bytes.set(b);

        self.set_offset(offset + BYTE_LEN);
        Ok(v)
    }

    /// Writes the "u16" value at the offset and increments the offset afterwards.
    /// ref. "avalanchego/utils/wrappers.Packer.PackShort"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.PackShort>
    pub fn pack_u16(&self, v: u16) -> Result<()> {
        self.expand(U16_LEN)?;

        let offset = self.get_offset();
        let mut b = self.bytes.take();

        // assume "offset" is not updated by the other "unpack*"
        // thus no need to keep internal cursor in sync with "offset"
        // unsafe { b.advance_mut(offset) };

        // writes an unsigned 16 bit integer in big-endian byte order
        // ref. "binary.BigEndian.PutUint16"
        b.put_u16(v);

        // remember to put it back -- "take" leaves the field as "Default::default()"
        self.bytes.set(b);

        // "put_u16" already advances the current position by U16_LEN
        // thus no need for "unsafe { b.advance_mut(offset + U16_LEN) };"
        // ref. https://docs.rs/bytes/latest/bytes/buf/trait.BufMut.html#method.put_u16
        self.set_offset(offset + U16_LEN);
        Ok(())
    }

    /// Unpacks the u16 from the "offset" position,
    /// and advances the cursor and offset.
    /// ref. "avalanchego/utils/wrappers.Packer.UnpackShort"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.UnpackShort>
    pub fn unpack_u16(&self) -> Result<u16> {
        self.check_remaining_unpack(U16_LEN)?;

        let offset = self.get_offset();
        let b = self.bytes.take();

        let pos = &b[offset..offset + U16_LEN];

        // ref. "binary.BigEndian.Uint16"
        // ref. https://doc.rust-lang.org/std/primitive.u16.html#method.from_be_bytes
        let v = u16::from_be_bytes([pos[0], pos[1]]);

        // remember to put it back -- "take" leaves the field as "Default::default()"
        self.bytes.set(b);

        self.set_offset(offset + U16_LEN);
        Ok(v)
    }

    /// Writes the "u32" value at the offset and increments the offset afterwards.
    /// This is also used for encoding the type IDs from codec.
    /// ref. "avalanchego/utils/wrappers.Packer.PackInt"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.PackInt>
    pub fn pack_u32(&self, v: u32) -> Result<()> {
        self.expand(U32_LEN)?;

        let offset = self.get_offset();
        let mut b = self.bytes.take();

        // assume "offset" is not updated by the other "unpack*"
        // thus no need to keep internal cursor in sync with "offset"
        // unsafe { b.advance_mut(offset) };

        // writes an unsigned 32 bit integer in big-endian byte order
        // ref. "binary.BigEndian.PutUint32"
        b.put_u32(v);

        // remember to put it back -- "take" leaves the field as "Default::default()"
        self.bytes.set(b);

        // "put_u32" already advances the current position by U32_LEN
        // thus no need for "unsafe { b.advance_mut(offset + U32_LEN) };"
        // ref. https://docs.rs/bytes/latest/bytes/buf/trait.BufMut.html#method.put_u32
        self.set_offset(offset + U32_LEN);
        Ok(())
    }

    /// Unpacks the u32 from the "offset" position,
    /// and advances the cursor and offset.
    /// ref. "avalanchego/utils/wrappers.Packer.UnpackInt"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.UnpackInt>
    pub fn unpack_u32(&self) -> Result<u32> {
        self.check_remaining_unpack(U32_LEN)?;

        let offset = self.get_offset();
        let b = self.bytes.take();

        let pos = &b[offset..offset + U32_LEN];

        // ref. "binary.BigEndian.Uint32"
        // ref. https://doc.rust-lang.org/std/primitive.u32.html#method.from_be_bytes
        let v = u32::from_be_bytes([pos[0], pos[1], pos[2], pos[3]]);

        // remember to put it back -- "take" leaves the field as "Default::default()"
        self.bytes.set(b);

        self.set_offset(offset + U32_LEN);
        Ok(v)
    }

    /// Writes the "u64" value at the offset and increments the offset afterwards.
    /// ref. "avalanchego/utils/wrappers.Packer.PackLong"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.PackLong>
    pub fn pack_u64(&self, v: u64) -> Result<()> {
        self.expand(U64_LEN)?;

        let offset = self.get_offset();
        let mut b = self.bytes.take();

        // assume "offset" is not updated by the other "unpack*"
        // thus no need to keep internal cursor in sync with "offset"
        // unsafe { b.advance_mut(offset) };

        // writes an unsigned 64 bit integer in big-endian byte order
        // ref. "binary.BigEndian.PutUint64"
        b.put_u64(v);

        // remember to put it back -- "take" leaves the field as "Default::default()"
        self.bytes.set(b);

        // "put_u64" already advances the current position by U64_LEN
        // thus no need for "unsafe { b.advance_mut(offset + U64_LEN) };"
        // ref. https://docs.rs/bytes/latest/bytes/buf/trait.BufMut.html#method.put_u64
        self.set_offset(offset + U64_LEN);
        Ok(())
    }

    /// Unpacks the u64 from the "offset" position,
    /// and advances the cursor and offset.
    /// ref. "avalanchego/utils/wrappers.Packer.UnpackLong"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.UnpackLong>
    pub fn unpack_u64(&self) -> Result<u64> {
        self.check_remaining_unpack(U64_LEN)?;

        let offset = self.get_offset();
        let b = self.bytes.take();

        let pos = &b[offset..offset + U64_LEN];

        // ref. "binary.BigEndian.Uint64"
        // ref. https://doc.rust-lang.org/std/primitive.u64.html#method.from_be_bytes
        let v = u64::from_be_bytes([
            pos[0], pos[1], pos[2], pos[3], pos[4], pos[5], pos[6], pos[7],
        ]);

        // remember to put it back -- "take" leaves the field as "Default::default()"
        self.bytes.set(b);

        self.set_offset(offset + U64_LEN);
        Ok(v)
    }

    /// Writes the "bool" value at the offset and increments the offset afterwards.
    /// ref. "avalanchego/utils/wrappers.Packer.PackBool"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.PackBool>
    pub fn pack_bool(&self, v: bool) -> Result<()> {
        if v {
            self.pack_byte(1)
        } else {
            self.pack_byte(0)
        }
    }

    /// Unpacks the bool in the "offset" position,
    /// and advances the cursor and offset.
    /// ref. "avalanchego/utils/wrappers.Packer.UnpackBool"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.UnpackBool>
    pub fn unpack_bool(&self) -> Result<bool> {
        let b = self.unpack_byte()?;
        match b {
            0 => Ok(false),
            1 => Ok(true),
            _ => {
                Err(Error::Other {
                    message: "unexpected value when unpacking bool".to_string(), // ref. "errBadBool"
                    retryable: false,
                })
            }
        }
    }

    /// Writes the "u8" fixed-size array from the offset and increments the offset as much.
    /// ref. "avalanchego/utils/wrappers.Packer.PackFixedBytes"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.PackFixedBytes>
    pub fn pack_bytes(&self, v: &[u8]) -> Result<()> {
        let n = v.len();
        self.expand(n)?;

        let offset = self.get_offset();
        let mut b = self.bytes.take();

        // assume "offset" is not updated by the other "unpack*"
        // thus no need to keep internal cursor in sync with "offset"
        // unsafe { b.advance_mut(offset) };

        // writes bytes from the offset
        // ref. "copy(p.Bytes[p.Offset:], bytes)"
        b.put_slice(v);

        // remember to put it back -- "take" leaves the field as "Default::default()"
        self.bytes.set(b);

        // "put_slice" already advances the current position by "n"
        // thus no need for "unsafe { b.advance_mut(offset + n) };"
        // ref. https://docs.rs/bytes/latest/bytes/buf/trait.BufMut.html#method.put_u64
        self.set_offset(offset + n);
        Ok(())
    }

    /// Unpacks the "u8" fixed-size array from the "offset" position,
    /// and advances the cursor and offset.
    /// ref. "avalanchego/utils/wrappers.Packer.UnpackFixedBytes"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.UnpackFixedBytes>
    pub fn unpack_bytes(&self, n: usize) -> Result<Vec<u8>> {
        self.check_remaining_unpack(n)?;

        let offset = self.get_offset();
        let b = self.bytes.take();

        let pos = &b[offset..offset + n];
        let v = Vec::from(pos);

        // remember to put it back -- "take" leaves the field as "Default::default()"
        self.bytes.set(b);

        self.set_offset(offset + n);
        Ok(v)
    }

    /// Writes the "u8" slice from the offset and increments the offset as much.
    /// The first 4-byte is used for encoding length header.
    /// ref. "avalanchego/utils/wrappers.Packer.PackBytes"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.PackBytes>
    pub fn pack_bytes_with_header(&self, v: &[u8]) -> Result<()> {
        self.pack_u32(v.len() as u32)?;
        self.pack_bytes(v)
    }

    /// Unpacks the "u8" slice from the "offset" position,
    /// and advances the cursor and offset.
    /// ref. "avalanchego/utils/wrappers.Packer.UnpackBytes"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.UnpackBytes>
    pub fn unpack_bytes_with_header(&self) -> Result<Vec<u8>> {
        let n = self.unpack_u32()?;
        self.unpack_bytes(n as usize)
    }

    /// Writes the two-dimensional "u8" slice from the offset and increments the offset as much.
    /// ref. "avalanchego/utils/wrappers.Packer.PackFixedByteSlices"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.PackFixedByteSlices>
    pub fn pack_2d_bytes(&self, v: Vec<Vec<u8>>) -> Result<()> {
        self.pack_u32(v.len() as u32)?;
        for vv in v.iter() {
            self.pack_bytes(vv)?;
        }
        Ok(())
    }

    /// Unpacks the two-dimensional "u8" slice from the "offset" position,
    /// and advances the cursor and offset.
    /// ref. "avalanchego/utils/wrappers.Packer.UnpackFixedByteSlices"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.UnpackFixedByteSlices>
    pub fn unpack_2d_bytes(&self, n: usize) -> Result<Vec<Vec<u8>>> {
        let total = self.unpack_u32()?;
        let mut rs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..total {
            let b = self.unpack_bytes(n)?;
            rs.push(b);
        }
        Ok(rs)
    }

    /// Writes the two-dimensional "u8" slice from the offset and increments the offset as much.
    /// ref. "avalanchego/utils/wrappers.Packer.Pack2DByteSlice"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.Pack2DByteSlice>
    pub fn pack_2d_bytes_with_header(&self, v: Vec<Vec<u8>>) -> Result<()> {
        self.pack_u32(v.len() as u32)?;
        for vv in v.iter() {
            self.pack_bytes_with_header(vv)?;
        }
        Ok(())
    }

    /// Unpacks the two-dimensional "u8" slice from the "offset" position,
    /// and advances the cursor and offset.
    /// ref. "avalanchego/utils/wrappers.Packer.Unpack2DByteSlice"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.Unpack2DByteSlice>
    pub fn unpack_2d_bytes_with_header(&self) -> Result<Vec<Vec<u8>>> {
        let total = self.unpack_u32()?;
        let mut rs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..total {
            let b = self.unpack_bytes_with_header()?;
            rs.push(b);
        }
        Ok(rs)
    }

    /// Writes str from the offset and increments the offset as much.
    /// ref. "avalanchego/utils/wrappers.Packer.PackStr"
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.PackStr>
    pub fn pack_str(&self, v: &str) -> Result<()> {
        let n = v.len() as u16;
        if n > MAX_STR_LEN {
            return Err(Error::Other {
                message: format!("str {} > max_size {}", n, MAX_STR_LEN),
                retryable: false,
            });
        }
        self.pack_u16(n)?;
        self.pack_bytes(v.as_bytes())
    }

    /// Unpacks str from the offset.
    /// ref. "avalanchego/utils/wrappers.Packer.UnpackStr"
    ///
    /// TODO: Go "UnpackStr" does deep-copy of bytes to "string" cast
    /// Can we bypass deep-copy by passing around bytes?
    /// ref. <https://github.com/golang/go/issues/25484>
    ///
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/wrappers#Packer.UnpackStr>
    pub fn unpack_str(&self) -> Result<String> {
        let n = self.unpack_u16()?;
        let d = self.unpack_bytes(n as usize)?;
        let s = match String::from_utf8(d) {
            Ok(v) => v,
            Err(e) => {
                return Err(Error::Other {
                    message: format!("failed String::from_utf8 {}", e),
                    retryable: false,
                });
            }
        };
        Ok(s)
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_expand --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerExpand"
#[test]
fn test_expand() {
    let s = [0x01];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(2),
    };
    assert!(packer.expand(1).is_err());

    let s = [0x01, 0x02, 0x03];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    packer.expand(1).unwrap();
    assert_eq!(packer.bytes_len(), 3);

    // 256 KiB
    let packer = Packer::new(256 * 1024, 128);
    packer.expand(10000).unwrap();
    assert_eq!(packer.bytes_len(), 0);
    assert_eq!(packer.bytes_cap(), 10000);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_packer_from_bytes --exact --show-output
#[test]
fn test_packer_from_bytes() {
    let s: Vec<u8> = vec![0x01, 0x02, 0x03];
    let packer = Packer::load_bytes_for_pack(10000, &s);
    packer.pack_byte(0x10).unwrap();
    assert_eq!(packer.bytes_len(), 4);
    assert_eq!(packer.get_offset(), 4);

    let b = packer.take_bytes();
    assert_eq!(&b[..], b"\x01\x02\x03\x10");
    let expected = [0x01, 0x02, 0x03, 0x10];
    assert_eq!(&b[..], &expected[..]);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_pack_byte --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerPackByte"
#[test]
fn test_pack_byte() {
    let packer = Packer::new(1, 0);
    packer.pack_byte(0x01).unwrap();
    assert_eq!(packer.bytes_len(), 1);
    assert_eq!(packer.get_offset(), 1);

    assert!(packer.pack_byte(0x02).is_err());
    assert_eq!(packer.bytes_len(), 1);
    assert_eq!(packer.get_offset(), 1);

    let b = packer.take_bytes();
    assert_eq!(&b[..], b"\x01");
    let expected = [0x01];
    assert_eq!(&b[..], &expected[..]);
    assert_eq!(packer.bytes_len(), 0);
    assert_eq!(packer.get_offset(), 1);

    packer.set_bytes(&b);
    assert_eq!(packer.bytes_len(), 1);

    let packer = Packer::new_with_header(5, 0);
    packer.pack_byte(0x01).unwrap();
    let expected = [0x00, 0x00, 0x00, 0x01, 0x01];
    assert_eq!(&packer.take_bytes()[..], &expected[..]);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_unpack_byte --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerUnpackByte"
#[test]
fn test_unpack_byte() {
    let s = [0x01];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    let b = packer.unpack_byte().unwrap();
    assert_eq!(b, 1);
    assert_eq!(packer.get_offset(), 1);

    assert!(packer.unpack_byte().is_err());
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_pack_u16 --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerPackShort"
#[test]
fn test_pack_u16() {
    let packer = Packer {
        max_size: U16_LEN,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };
    packer.pack_u16(0x0102).unwrap();
    assert_eq!(packer.bytes_len(), U16_LEN);

    let b = packer.take_bytes();
    assert_eq!(&b[..], b"\x01\x02");
    let expected = [0x01, 0x02];
    assert_eq!(&b[..], &expected[..]);

    let packer = Packer::new_with_header(4 + U16_LEN, 0);
    packer.pack_u16(0x0102).unwrap();
    let expected = [0x00, 0x00, 0x00, 0x02, 0x01, 0x02];
    assert_eq!(&packer.take_bytes()[..], &expected[..]);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_unpack_u16 --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerUnpackShort"
#[test]
fn test_unpack_u16() {
    let s: Vec<u8> = vec![0x01, 0x02];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    let b = packer.unpack_u16().unwrap();
    assert_eq!(b, 0x0102);
    assert_eq!(packer.get_offset(), U16_LEN);

    assert!(packer.unpack_u16().is_err());
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_pack_u16_short --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPacker"
#[test]
fn test_pack_u16_short() {
    let packer = Packer {
        max_size: 3,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };

    packer.pack_u16(17).unwrap();
    assert_eq!(packer.bytes_len(), 2);
    assert!(packer.pack_u16(1).is_err());

    let b = packer.take_bytes();
    let expected = [0x00, 17];
    assert_eq!(&b[..], &expected[..]);

    let s: Vec<u8> = vec![0x00, 17];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    let b = packer.unpack_u16().unwrap();
    assert_eq!(b, 17);
    assert_eq!(packer.get_offset(), U16_LEN);

    let packer = Packer::new_with_header(4 + U16_LEN, 0);
    packer.pack_u16(17).unwrap();
    let expected = [0x00, 0x00, 0x00, 0x02, 0x00, 17];
    assert_eq!(&packer.take_bytes()[..], &expected[..]);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_pack_u32 --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerPackInt"
#[test]
fn test_pack_u32() {
    let packer = Packer {
        max_size: U32_LEN,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };
    packer.pack_u32(0x01020304).unwrap();
    assert_eq!(packer.bytes_len(), U32_LEN);
    assert!(packer.pack_u32(0x05060708).is_err());

    let b = packer.take_bytes();
    assert_eq!(&b[..], b"\x01\x02\x03\x04");
    let expected = [0x01, 0x02, 0x03, 0x04];
    assert_eq!(&b[..], &expected[..]);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_unpack_u32 --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerUnpackInt"
#[test]
fn test_unpack_u32() {
    let s: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    assert_eq!(packer.unpack_u32().unwrap(), 0x01020304);
    assert_eq!(packer.get_offset(), U32_LEN);
    assert!(packer.unpack_u32().is_err());
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_pack_u64 --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerPackLong"
#[test]
fn test_pack_u64() {
    let packer = Packer {
        max_size: U64_LEN,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };
    packer.pack_u64(0x0102030405060708).unwrap();
    assert_eq!(packer.bytes_len(), U64_LEN);

    // beyond max size
    assert!(packer.pack_u64(0x090a0b0c0d0e0f00).is_err());

    let b = packer.take_bytes();
    assert_eq!(&b[..], b"\x01\x02\x03\x04\x05\x06\x07\x08");
    let expected = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    assert_eq!(&b[..], &expected[..]);

    let packer = Packer::new_with_header(4 + U64_LEN, 0);
    packer.pack_u64(0x0102030405060708).unwrap();
    let expected = [
        0x00, 0x00, 0x00, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ];
    assert_eq!(&packer.take_bytes()[..], &expected[..]);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_unpack_u64 --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerUnpackLong"
#[test]
fn test_unpack_u64() {
    let s: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    assert_eq!(packer.unpack_u64().unwrap(), 0x0102030405060708);
    assert_eq!(packer.get_offset(), U64_LEN);
    assert!(packer.unpack_u64().is_err());
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_pack_bool --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackBool"
/// ref. "avalanchego/utils/wrappers.TestPackerPackBool"
#[test]
fn test_pack_bool() {
    let packer = Packer {
        max_size: 3,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };
    packer.pack_bool(false).unwrap();
    packer.pack_bool(true).unwrap();
    packer.pack_bool(false).unwrap();
    assert_eq!(packer.bytes_len(), 3);

    assert!(packer.pack_bool(true).is_err());

    let b = packer.take_bytes();
    assert_eq!(&b[..], b"\x00\x01\x00");
    let expected = [0x00, 0x01, 0x00];
    assert_eq!(&b[..], &expected[..]);

    let b = BytesMut::from(&expected[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    assert!(!packer.unpack_bool().unwrap());
    assert!(packer.unpack_bool().unwrap());
    assert!(!packer.unpack_bool().unwrap());

    let packer = Packer::new_with_header(4 + 3, 0);
    packer.pack_bool(false).unwrap();
    packer.pack_bool(true).unwrap();
    packer.pack_bool(false).unwrap();
    let expected = [0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x00];
    assert_eq!(&packer.take_bytes()[..], &expected[..]);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_unpack_bool --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackBool"
/// ref. "avalanchego/utils/wrappers.TestPackerUnpackBool"
#[test]
fn test_unpack_bool() {
    let s = [0x01];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    assert!(packer.unpack_bool().unwrap());
    assert_eq!(packer.get_offset(), BOOL_LEN);
    assert!(packer.unpack_bool().is_err());
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_pack_bytes --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerPackFixedBytes"
#[test]
fn test_pack_bytes() {
    let packer = Packer {
        max_size: 8,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };

    let s = "Avax";
    packer.pack_bytes(s.as_bytes()).unwrap();
    assert_eq!(packer.bytes_len(), 4);

    packer.pack_bytes(s.as_bytes()).unwrap();
    assert_eq!(packer.bytes_len(), 8);

    // beyond max size
    assert!(packer.pack_bytes(s.as_bytes()).is_err());

    let b = packer.take_bytes();
    assert_eq!(&b[..], b"AvaxAvax");
    let expected = [65, 118, 97, 120, 65, 118, 97, 120];
    assert_eq!(&b[..], &expected[..]);

    let packer = Packer::new_with_header(4 + 8, 0);
    packer.pack_bytes(s.as_bytes()).unwrap();
    packer.pack_bytes(s.as_bytes()).unwrap();
    let expected = [0x00, 0x00, 0x00, 0x08, 65, 118, 97, 120, 65, 118, 97, 120];
    assert_eq!(&packer.take_bytes()[..], &expected[..]);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_unpack_bytes --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerUnpackFixedBytes"
#[test]
fn test_unpack_bytes() {
    let s: Vec<u8> = vec![65, 118, 97, 120];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    let b = packer.unpack_bytes(4).unwrap();
    assert_eq!(&b[..], b"Avax");
    assert_eq!(packer.get_offset(), 4);
    assert!(packer.unpack_bytes(4).is_err());
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_pack_bytes_with_header --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerPackBytes"
#[test]
fn test_pack_bytes_with_header() {
    let packer = Packer {
        max_size: 8,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };

    let s = "Avax";
    packer.pack_bytes_with_header(s.as_bytes()).unwrap();
    assert_eq!(packer.bytes_len(), 8);

    // beyond max size
    assert!(packer.pack_bytes_with_header(s.as_bytes()).is_err());

    let b = packer.take_bytes();
    assert_eq!(&b[..], b"\x00\x00\x00\x04Avax");
    let expected = [0x00, 0x00, 0x00, 0x04, 65, 118, 97, 120];
    assert_eq!(&b[..], &expected[..]);

    let packer = Packer::new_with_header(4 + 8, 0);
    packer.pack_bytes_with_header(s.as_bytes()).unwrap();
    let expected = [
        0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x04, 65, 118, 97, 120,
    ];
    assert_eq!(&packer.take_bytes()[..], &expected[..]);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_unpack_bytes_with_header --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerUnpackBytes"
#[test]
fn test_unpack_bytes_with_header() {
    let s: Vec<u8> = vec![0x00, 0x00, 0x00, 0x04, 65, 118, 97, 120];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    let b = packer.unpack_bytes_with_header().unwrap();
    assert_eq!(&b[..], b"Avax");
    assert_eq!(packer.get_offset(), 8);
    assert!(packer.unpack_bytes_with_header().is_err());
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_pack_2d_bytes --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerPackFixedByteSlices"
#[test]
fn test_pack_2d_bytes() {
    let packer = Packer {
        max_size: 12,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };

    // first 4-byte is for length
    let s1 = "Avax";
    let s2 = "Evax";
    packer
        .pack_2d_bytes(vec![Vec::from(s1.as_bytes()), Vec::from(s2.as_bytes())])
        .unwrap();
    assert_eq!(packer.bytes_len(), 12);

    // beyond max size
    assert!(packer
        .pack_2d_bytes(vec![Vec::from(s1.as_bytes()), Vec::from(s2.as_bytes()),])
        .is_err());

    let b = packer.take_bytes();
    assert_eq!(&b[..], b"\x00\x00\x00\x02AvaxEvax");
    let expected = [0x00, 0x00, 0x00, 0x02, 65, 118, 97, 120, 69, 118, 97, 120];
    assert_eq!(&b[..], &expected[..]);

    let packer = Packer::new_with_header(4 + 12, 0);
    packer
        .pack_2d_bytes(vec![Vec::from(s1.as_bytes()), Vec::from(s2.as_bytes())])
        .unwrap();
    let expected = [
        0x00, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x02, 65, 118, 97, 120, 69, 118, 97, 120,
    ];
    assert_eq!(&packer.take_bytes()[..], &expected[..]);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_unpack_2d_bytes --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerUnpackFixedByteSlices"
#[test]
fn test_unpack_2d_bytes() {
    let s: Vec<u8> = vec![0x00, 0x00, 0x00, 0x02, 65, 118, 97, 120, 69, 118, 97, 120];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    let b = packer.unpack_2d_bytes(4).unwrap();
    assert_eq!(
        &b[..],
        vec![Vec::from("Avax".as_bytes()), Vec::from("Evax".as_bytes()),]
    );
    assert_eq!(packer.get_offset(), 12);
    assert!(packer.unpack_2d_bytes(4).is_err());
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_pack_2d_bytes_with_header --exact --show-output
#[test]
fn test_pack_2d_bytes_with_header() {
    let packer = Packer {
        max_size: 1024,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };

    // first 4-byte is for length
    // two more 4-bytes for each length
    let s1 = "Avax";
    let s2 = "Evax";
    packer
        .pack_2d_bytes_with_header(vec![Vec::from(s1.as_bytes()), Vec::from(s2.as_bytes())])
        .unwrap();
    assert_eq!(packer.bytes_len(), 20); // 4*3 + 4*2

    let b = packer.take_bytes();
    assert_eq!(
        &b[..],
        b"\x00\x00\x00\x02\x00\x00\x00\x04Avax\x00\x00\x00\x04Evax"
    );
    let expected = [
        0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x04, 65, 118, 97, 120, 0x00, 0x00, 0x00, 0x04,
        69, 118, 97, 120,
    ];
    assert_eq!(&b[..], &expected[..]);

    let packer = Packer::new_with_header(4 + 20, 0);
    packer
        .pack_2d_bytes_with_header(vec![Vec::from(s1.as_bytes()), Vec::from(s2.as_bytes())])
        .unwrap();
    let expected = [
        0x00, 0x00, 0x00, 20, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x04, 65, 118, 97, 120,
        0x00, 0x00, 0x00, 0x04, 69, 118, 97, 120,
    ];
    assert_eq!(&packer.take_bytes()[..], &expected[..]);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_unpack_2d_bytes_with_header --exact --show-output
#[test]
fn test_unpack_2d_bytes_with_header() {
    let s: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x04, 65, 118, 97, 120, 0x00, 0x00, 0x00, 0x04,
        69, 118, 97, 120,
    ];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 0,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    let b = packer.unpack_2d_bytes_with_header().unwrap();
    assert_eq!(
        &b[..],
        vec![Vec::from("Avax".as_bytes()), Vec::from("Evax".as_bytes()),]
    );
    assert_eq!(packer.get_offset(), 20);
    assert!(packer.unpack_2d_bytes_with_header().is_err());
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_pack_2d_bytes_with_header_123 --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPacker2DByteSlice"
#[test]
fn test_pack_2d_bytes_with_header_123() {
    // case 1; empty
    let packer = Packer {
        max_size: 1024,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };
    packer.pack_2d_bytes_with_header(vec![]).unwrap();
    assert_eq!(packer.bytes_len(), 4);
    assert!(packer.unpack_2d_bytes_with_header().is_err());

    // case 2; only one dimension
    let packer = Packer {
        max_size: 1024,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };
    packer
        .pack_2d_bytes_with_header(vec![vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]])
        .unwrap();
    assert_eq!(packer.bytes_len(), 4 + 4 + 10);

    let b = packer.take_bytes();
    let expected = [0, 0, 0, 1, 0, 0, 0, 10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(&b[..], &expected[..]);

    let s: Vec<u8> = vec![0, 0, 0, 1, 0, 0, 0, 10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 1024,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    let b = packer.unpack_2d_bytes_with_header().unwrap();
    assert_eq!(&b[..], vec![vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]]);
    assert_eq!(packer.get_offset(), 4 + 4 + 10);

    // case 3; two dimensions
    let packer = Packer {
        max_size: 1024,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };
    packer
        .pack_2d_bytes_with_header(vec![
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            vec![11, 12, 3, 4, 5, 6, 7, 8, 9, 10],
        ])
        .unwrap();
    assert_eq!(packer.bytes_len(), 4 + 4 + 10 + 4 + 10);

    let b = packer.take_bytes();
    let expected = [
        0, 0, 0, 2, 0, 0, 0, 10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 0, 0, 10, 11, 12, 3, 4, 5, 6, 7,
        8, 9, 10,
    ];
    assert_eq!(&b[..], &expected[..]);

    let s: Vec<u8> = vec![
        0, 0, 0, 2, 0, 0, 0, 10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 0, 0, 10, 11, 12, 3, 4, 5, 6, 7,
        8, 9, 10,
    ];
    let b = BytesMut::from(&s[..]);
    let packer = Packer {
        max_size: 1024,
        bytes: Cell::new(b),
        header: false,
        offset: Cell::new(0),
    };
    let b = packer.unpack_2d_bytes_with_header().unwrap();
    assert_eq!(
        &b[..],
        vec![
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            vec![11, 12, 3, 4, 5, 6, 7, 8, 9, 10],
        ]
    );
    assert_eq!(packer.get_offset(), 4 + 4 + 10 + 4 + 10);
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- packer::test_pack_str --exact --show-output
/// ref. "avalanchego/utils/wrappers.TestPackerString"
#[test]
fn test_pack_str() {
    let packer = Packer {
        max_size: 6,
        bytes: Cell::new(BytesMut::with_capacity(0)),
        header: false,
        offset: Cell::new(0),
    };

    let s = "Avax";
    packer.pack_str(s).unwrap();
    assert_eq!(packer.bytes_len(), 2 + 4);

    // beyond max size
    assert!(packer.pack_str(s).is_err());

    let b = packer.take_bytes();
    assert_eq!(&b[..], b"\x00\x04Avax");
    let expected = [0x00, 0x04, 65, 118, 97, 120];
    assert_eq!(&b[..], &expected[..]);

    let packer = Packer::new_with_header(4 + 6, 0);
    packer.pack_str(s).unwrap();
    let expected = [0x00, 0x00, 0x00, 0x06, 0x00, 0x04, 65, 118, 97, 120];
    assert_eq!(&packer.take_bytes()[..], &expected[..]);
}
