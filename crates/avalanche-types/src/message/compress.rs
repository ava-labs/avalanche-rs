use std::io::{self, Cursor, Read};

use flate2::{
    bufread::{GzDecoder, GzEncoder},
    Compression,
};

/// Compress the input bytes.

pub fn pack_gzip<S>(d: S) -> io::Result<Vec<u8>>
where
    S: AsRef<[u8]>,
{
    // ref. "golang/compress/flag.DefaultCompression" is -1 which is level 6
    // "Compression::default()" returns 6
    let mut gz = GzEncoder::new(Cursor::new(d), Compression::new(6));
    let mut encoded = Vec::new();
    gz.read_to_end(&mut encoded)?;
    Ok(encoded)
}

/// Decompress the input bytes.

pub fn unpack_gzip<S>(d: S) -> io::Result<Vec<u8>>
where
    S: AsRef<[u8]>,
{
    let mut gz = GzDecoder::new(Cursor::new(d));
    let mut decoded = Vec::new();
    gz.read_to_end(&mut decoded)?;
    Ok(decoded)
}
