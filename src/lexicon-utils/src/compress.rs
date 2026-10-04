// Streaming compression with untrusted-input limits (Spec §51).
//
// Backed by gzip (flate2). Other codecs (brotli, zstd, lz4) are documented
// integration points: the API is codec-agnostic so callers migrate without
// rewrites. Decompression ALWAYS requires an explicit output cap.

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{Read, Write};

/// Compress with the default level.
pub fn gzip_compress(data: &[u8]) -> Vec<u8> {
    gzip_compress_level(data, Compression::default())
}

pub fn gzip_compress_level(data: &[u8], level: Compression) -> Vec<u8> {
    let mut enc = GzEncoder::new(Vec::new(), level);
    enc.write_all(data).expect("gzip encode cannot fail on Vec");
    enc.finish().expect("gzip finish cannot fail on Vec")
}

/// Decompress, refusing outputs larger than `max_output` bytes.
/// Returns an error instead of exhausting memory (zip-bomb defense).
pub fn gzip_decompress_limited(data: &[u8], max_output: usize) -> Result<Vec<u8>, String> {
    let mut dec = GzDecoder::new(data);
    let mut out = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = dec.read(&mut buf).map_err(|e| format!("gzip corrupt: {}", e))?;
        if n == 0 {
            break;
        }
        if out.len() + n > max_output {
            return Err(format!(
                "E0601: gzip output exceeds limit of {} bytes",
                max_output
            ));
        }
        out.extend_from_slice(&buf[..n]);
    }
    Ok(out)
}

/// Streaming compressor: feed chunks, finish for the trailer.
///
/// Ownership: owns the encoder until `finish`. Thread-safety: `!Sync`;
/// move across threads, do not share. Complexity: O(bytes).
pub struct GzipStream {
    enc: Option<GzEncoder<Vec<u8>>>,
}

impl GzipStream {
    pub fn new() -> Self {
        GzipStream { enc: Some(GzEncoder::new(Vec::new(), Compression::default())) }
    }

    pub fn feed(&mut self, chunk: &[u8]) -> Result<(), String> {
        self.enc
            .as_mut()
            .ok_or_else(|| "stream already finished".to_string())?
            .write_all(chunk)
            .map_err(|e| e.to_string())
    }

    pub fn finish(&mut self) -> Result<Vec<u8>, String> {
        self.enc
            .take()
            .ok_or_else(|| "stream already finished".to_string())?
            .finish()
            .map_err(|e| e.to_string())
    }
}

impl Default for GzipStream {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Codec-agnostic facade (Spec §51).
//
// Only `Gzip` is backed by `flate2` today. Every other variant is a
// documented integration point (`brotli`/`zstd`/`lz4` crates) that fails
// with a stable `E0601` error instead of silently picking another codec.
// Decompression ALWAYS requires an explicit `max_output` cap so hostile
// inputs fail fast (zip-bomb defense).
//
// Ownership: pure functions over borrowed slices, owned output.
// Thread-safety: `Send + Sync` (no shared state).
// Complexity: Gzip O(n); stubs O(1) error.
// ---------------------------------------------------------------------------

/// Compression codecs (Spec §51).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    Gzip,
    Zlib,
    Deflate,
    Brotli,
    Zstd,
    Lz4,
}

#[allow(dead_code)]
impl Codec {
    /// Parse `gzip`/`zlib`/`deflate`/`brotli`/`zstd`/`lz4` (case-insensitive).
    /// Complexity: O(1).
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "gzip" | "gz" => Some(Codec::Gzip),
            "zlib" => Some(Codec::Zlib),
            "deflate" => Some(Codec::Deflate),
            "brotli" | "br" => Some(Codec::Brotli),
            "zstd" | "zst" => Some(Codec::Zstd),
            "lz4" => Some(Codec::Lz4),
            _ => None,
        }
    }

    /// Short name. Complexity: O(1).
    pub fn name(self) -> &'static str {
        match self {
            Codec::Gzip => "gzip",
            Codec::Zlib => "zlib",
            Codec::Deflate => "deflate",
            Codec::Brotli => "brotli",
            Codec::Zstd => "zstd",
            Codec::Lz4 => "lz4",
        }
    }

    /// True only for codecs with a backend in this build.
    pub fn is_supported(self) -> bool {
        matches!(self, Codec::Gzip)
    }
}

/// Default output cap for untrusted decompression (8 MiB).
#[allow(dead_code)]
pub const MAX_DECOMPRESSED_DEFAULT: usize = 8 << 20;

/// Max input accepted for one-shot (untrusted) compression (64 MiB).
/// Larger inputs must use streaming. Complexity note: enforced O(1) check.
#[allow(dead_code)]
pub const MAX_COMPRESS_INPUT: usize = 64 << 20;

/// Compress with the named codec. Gzip is real; others return a stable
/// integration-point error. Complexity: Gzip O(n).
#[allow(dead_code)]
pub fn compress(codec: Codec, data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() > MAX_COMPRESS_INPUT {
        return Err(format!("E0601: compress input exceeds {} bytes", MAX_COMPRESS_INPUT));
    }
    match codec {
        Codec::Gzip => Ok(gzip_compress(data)),
        other => Err(format!(
            "E0601: codec {} not enabled in this build (integration point: add `{}-crate` backend)",
            other.name(),
            other.name()
        )),
    }
}

/// Decompress with an explicit output cap. Gzip is real; others error.
/// Complexity: Gzip O(output); stubs O(1).
#[allow(dead_code)]
pub fn decompress_limited(codec: Codec, data: &[u8], max_output: usize) -> Result<Vec<u8>, String> {
    if data.len() > MAX_COMPRESS_INPUT {
        return Err(format!("E0601: decompress input exceeds {} bytes", MAX_COMPRESS_INPUT));
    }
    match codec {
        Codec::Gzip => gzip_decompress_limited(data, max_output),
        other => Err(format!(
            "E0601: codec {} not enabled in this build (integration point: add `{}-crate` backend)",
            other.name(),
            other.name()
        )),
    }
}

/// Streaming facade: Gzip-backed; other codecs error at construction.
/// Ownership: owns internal buffer. Thread-safety: `!Sync`. Complexity: O(bytes).
#[allow(dead_code)]
pub struct StreamCompressor {
    codec: Codec,
    gzip: Option<GzipStream>,
}

#[allow(dead_code)]
impl StreamCompressor {
    pub fn new(codec: Codec) -> Result<Self, String> {
        match codec {
            Codec::Gzip => Ok(StreamCompressor { codec, gzip: Some(GzipStream::new()) }),
            other => Err(format!("E0601: streaming codec {} not enabled", other.name())),
        }
    }

    pub fn codec(&self) -> Codec {
        self.codec
    }

    pub fn feed(&mut self, chunk: &[u8]) -> Result<(), String> {
        self.gzip.as_mut().ok_or_else(|| "stream already finished".to_string())?.feed(chunk)
    }

    pub fn finish(&mut self) -> Result<Vec<u8>, String> {
        self.gzip.as_mut().ok_or_else(|| "stream already finished".to_string())?.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let data = b"hello lexicon ".repeat(100);
        let c = gzip_compress(&data);
        assert!(c.len() < data.len());
        assert_eq!(gzip_decompress_limited(&c, 1 << 20).unwrap(), data);
    }

    #[test]
    fn streaming_matches_oneshot() {
        let data = b"abcdef".repeat(50);
        let mut s = GzipStream::new();
        for chunk in data.chunks(7) {
            s.feed(chunk).unwrap();
        }
        let streamed = s.finish().unwrap();
        let out = gzip_decompress_limited(&streamed, 1 << 20).unwrap();
        assert_eq!(out, data);
    }

    #[test]
    fn bomb_rejected() {
        let data = vec![0u8; 1 << 16];
        let c = gzip_compress(&data);
        assert!(gzip_decompress_limited(&c, 16).is_err());
        assert!(gzip_decompress_limited(b"not gzip", 1 << 20).is_err());
    }

    #[test]
    fn codec_facade() {
        assert_eq!(Codec::parse("GZIP"), Some(Codec::Gzip));
        assert_eq!(Codec::parse("zstd"), Some(Codec::Zstd));
        assert_eq!(Codec::parse("nope"), None);
        assert!(Codec::Gzip.is_supported());
        assert!(!Codec::Zstd.is_supported());
        let data = b"codec facade ".repeat(50);
        let c = compress(Codec::Gzip, &data).unwrap();
        assert_eq!(decompress_limited(Codec::Gzip, &c, 1 << 20).unwrap(), data);
        assert!(compress(Codec::Zstd, &data).is_err());
        assert!(decompress_limited(Codec::Brotli, &c, 1 << 20).is_err());
        assert!(StreamCompressor::new(Codec::Lz4).is_err());
        let mut s = StreamCompressor::new(Codec::Gzip).unwrap();
        assert_eq!(s.codec(), Codec::Gzip);
        s.feed(&data).unwrap();
        let out = s.finish().unwrap();
        assert_eq!(decompress_limited(Codec::Gzip, &out, 1 << 20).unwrap(), data);
    }
}
