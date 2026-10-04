use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Maximum frame payload accepted from untrusted input (8 MiB).
    pub max_bytes: usize,
    /// Maximum nesting depth of structures (maps/arrays/objects).
    pub max_depth: u32,
    /// Maximum container element count.
    pub max_array_len: usize,
}

impl Limits {
    pub fn new(max_bytes: usize, max_depth: u32, max_array_len: usize) -> Self {
        Self {
            max_bytes,
            max_depth,
            max_array_len,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerError {
    UnknownField(String),
    MissingField(String),
    Null(String),
    Overflow(String),
    Recursion(String),
    VersionMismatch(String),
    Utf8(String),
    Truncated(String),
}

impl std::fmt::Display for SerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerError::UnknownField(m) => write!(f, "unknown field: {m}"),
            SerError::MissingField(m) => write!(f, "missing field: {m}"),
            SerError::Null(m) => write!(f, "null value: {m}"),
            SerError::Overflow(m) => write!(f, "overflow / limit exceeded: {m}"),
            SerError::Recursion(m) => write!(f, "recursion limit exceeded: {m}"),
            SerError::VersionMismatch(m) => write!(f, "version mismatch: {m}"),
            SerError::Utf8(m) => write!(f, "invalid utf-8: {m}"),
            SerError::Truncated(m) => write!(f, "truncated input: {m}"),
        }
    }
}

impl std::error::Error for SerError {}

impl From<serde_json::Error> for SerError {
    fn from(e: serde_json::Error) -> Self {
        let msg = e.to_string();
        let lower = msg.to_lowercase();
        if lower.contains("unknown field") {
            SerError::UnknownField(msg)
        } else if lower.contains("missing field") {
            SerError::MissingField(msg)
        } else if lower.contains("null") {
            SerError::Null(msg)
        } else if lower.contains("recursion") || lower.contains("depth") {
            SerError::Recursion(msg)
        } else if lower.contains("eof") || lower.contains("expected") {
            SerError::Truncated(msg)
        } else {
            SerError::Truncated(msg)
        }
    }
}

impl From<std::str::Utf8Error> for SerError {
    fn from(e: std::str::Utf8Error) -> Self {
        SerError::Utf8(e.to_string())
    }
}

/// Maximum frame payload accepted from untrusted input (8 MiB).
pub const MAX_FRAME: usize = 8 << 20;
/// Maximum MessagePack nesting depth.
pub const MAX_DEPTH: usize = 32;
/// Maximum container element count.
pub const MAX_ELEMENTS: usize = 1 << 20;

/// Unsigned LEB128-style varint.
pub fn varint_encode(mut v: u64) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut b = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            b |= 0x80;
        }
        out.push(b);
        if v == 0 {
            break;
        }
    }
    out
}

pub fn varint_decode(input: &[u8]) -> Option<(u64, usize)> {
    let mut result = 0u64;
    let mut shift = 0u32;
    for (i, b) in input.iter().enumerate().take(10) {
        let bits = ((b & 0x7f) as u64)
            .checked_shl(shift)
            .filter(|_| shift < 64)?;
        result |= bits;
        if b & 0x80 == 0 {
            return Some((result, i + 1));
        }
        shift += 7;
    }
    None
}

/// Length-prefixed frame: `varint(len) ++ payload`, rejected over MAX_FRAME.
pub fn frame_encode(payload: &[u8]) -> Option<Vec<u8>> {
    if payload.len() > MAX_FRAME {
        return None;
    }
    let mut out = varint_encode(payload.len() as u64);
    out.extend_from_slice(payload);
    Some(out)
}

pub fn frame_decode(input: &[u8]) -> Option<(&[u8], usize)> {
    let (len, n) = varint_decode(input)?;
    let len = usize::try_from(len).ok()?;
    if len > MAX_FRAME || input.len() < n + len {
        return None;
    }
    Some((&input[n..n + len], n + len))
}

#[derive(Debug, Clone, PartialEq)]
pub enum MpValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Bin(Vec<u8>),
    Array(Vec<MpValue>),
    Map(Vec<(String, MpValue)>),
}

struct MpReader<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> MpReader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.pos + n > self.input.len() || n > MAX_FRAME {
            return None;
        }
        let s = &self.input[self.pos..self.pos + n];
        self.pos += n;
        Some(s)
    }

    fn byte(&mut self) -> Option<u8> {
        let b = *self.input.get(self.pos)?;
        self.pos += 1;
        Some(b)
    }

    fn be_u16(&mut self) -> Option<usize> {
        let b = self.take(2)?;
        Some(u16::from_be_bytes([b[0], b[1]]) as usize)
    }

    fn be_u32(&mut self) -> Option<usize> {
        let b = self.take(4)?;
        Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize)
    }

    fn value(&mut self, depth: usize) -> Option<MpValue> {
        if depth > MAX_DEPTH {
            return None;
        }
        let b = self.byte()?;
        match b {
            0xc0 => Some(MpValue::Nil),
            0xc2 => Some(MpValue::Bool(false)),
            0xc3 => Some(MpValue::Bool(true)),
            0x00..=0x7f => Some(MpValue::Int(b as i64)),
            0xe0..=0xff => Some(MpValue::Int((b as i8) as i64)),
            0xcc => Some(MpValue::Int(self.byte()? as i64)),
            0xcd => Some(MpValue::Int(self.be_u16()? as i64)),
            0xca => {
                let raw = self.be_u32()?;
                Some(MpValue::Float(f32::from_bits(raw as u32) as f64))
            }
            0xcb => {
                let hi = self.be_u32()? as u64;
                let lo = self.be_u32()? as u64;
                Some(MpValue::Float(f64::from_bits((hi << 32) | lo)))
            }
            0xd0 => Some(MpValue::Int(self.byte()? as i8 as i64)),
            0xd1 => Some(MpValue::Int(self.be_u16()? as i16 as i64)),
            0xa0..=0xbf => {
                let n = (b & 0x1f) as usize;
                let s = self.take(n)?;
                Some(MpValue::Str(String::from_utf8(s.to_vec()).ok()?))
            }
            0xd9 => {
                let n = self.byte()? as usize;
                let s = self.take(n)?;
                Some(MpValue::Str(String::from_utf8(s.to_vec()).ok()?))
            }
            0xc4 => {
                let n = self.byte()? as usize;
                Some(MpValue::Bin(self.take(n)?.to_vec()))
            }
            0x90..=0x9f => {
                let n = (b & 0x0f) as usize;
                if n > MAX_ELEMENTS {
                    return None;
                }
                let mut items = Vec::with_capacity(n);
                for _ in 0..n {
                    items.push(self.value(depth + 1)?);
                }
                Some(MpValue::Array(items))
            }
            0x80..=0x8f => {
                let n = (b & 0x0f) as usize;
                if n > MAX_ELEMENTS {
                    return None;
                }
                let mut items = Vec::with_capacity(n);
                for _ in 0..n {
                    let k = match self.value(depth + 1)? {
                        MpValue::Str(s) => s,
                        _ => return None,
                    };
                    let v = self.value(depth + 1)?;
                    items.push((k, v));
                }
                Some(MpValue::Map(items))
            }
            _ => None,
        }
    }
}

/// Decode one MessagePack value; rejects over-deep/over-large input.
/// JSON with UTF-8 validation + limits.
fn check_json_value_limits(
    value: &serde_json::Value,
    limits: &Limits,
    depth: u32,
) -> Result<(), SerError> {
    if depth > limits.max_depth {
        return Err(SerError::Recursion(format!(
            "depth {depth} exceeds max {}",
            limits.max_depth
        )));
    }
    match value {
        serde_json::Value::Array(items) => {
            if items.len() > limits.max_array_len {
                return Err(SerError::Overflow(format!(
                    "array len {} exceeds max {}",
                    items.len(),
                    limits.max_array_len
                )));
            }
            for item in items {
                check_json_value_limits(item, limits, depth + 1)?;
            }
        }
        serde_json::Value::Object(map) => {
            if map.len() > limits.max_array_len {
                return Err(SerError::Overflow(format!(
                    "object len {} exceeds max {}",
                    map.len(),
                    limits.max_array_len
                )));
            }
            for (_, v) in map.iter() {
                check_json_value_limits(v, limits, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Serializes to JSON applying `Limits` (size + depth + array len).
pub fn encode_json<T: serde::Serialize>(value: &T, limits: &Limits) -> Result<String, SerError> {
    let s = serde_json::to_string(value).map_err(SerError::from)?;
    if s.len() > limits.max_bytes {
        return Err(SerError::Overflow(format!(
            "json bytes {} exceeds max {}",
            s.len(),
            limits.max_bytes
        )));
    }
    let v: serde_json::Value = serde_json::from_str(&s).map_err(SerError::from)?;
    check_json_value_limits(&v, limits, 0)?;
    Ok(s)
}

/// Deserializes from bytes applying `Limits` + explicit UTF-8 validation.
pub fn decode_json<T>(bytes: &[u8], limits: &Limits) -> Result<T, SerError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    if bytes.len() > limits.max_bytes {
        return Err(SerError::Overflow(format!(
            "input bytes {} exceeds max {}",
            bytes.len(),
            limits.max_bytes
        )));
    }
    let s = std::str::from_utf8(bytes).map_err(|e| SerError::Utf8(e.to_string()))?;
    let v: serde_json::Value = serde_json::from_str(s).map_err(SerError::from)?;
    check_json_value_limits(&v, limits, 0)?;
    let out: T = serde_json::from_value(v).map_err(SerError::from)?;
    Ok(out)
}

pub fn msgpack_decode(input: &[u8]) -> Option<MpValue> {
    if input.len() > MAX_FRAME {
        return None;
    }
    MpReader { input, pos: 0 }.value(0)
}

fn mp_encode_into(out: &mut Vec<u8>, v: &MpValue) {
    match v {
        MpValue::Nil => out.push(0xc0),
        MpValue::Bool(false) => out.push(0xc2),
        MpValue::Bool(true) => out.push(0xc3),
        MpValue::Int(i) if (0..128).contains(i) => out.push(*i as u8),
        MpValue::Int(i) if (-32..0).contains(i) => out.push(*i as i8 as u8),
        MpValue::Int(i) if (0..256).contains(i) => {
            out.push(0xcc);
            out.push(*i as u8);
        }
        MpValue::Int(i) => {
            out.push(0xd1);
            out.extend_from_slice(&(*i as i16).to_be_bytes());
        }
        MpValue::Float(f) => {
            out.push(0xcb);
            out.extend_from_slice(&f.to_bits().to_be_bytes());
        }
        MpValue::Str(s) if s.len() < 32 => {
            out.push(0xa0 | s.len() as u8);
            out.extend_from_slice(s.as_bytes());
        }
        MpValue::Str(s) if s.len() < 256 => {
            out.push(0xd9);
            out.push(s.len() as u8);
            out.extend_from_slice(s.as_bytes());
        }
        MpValue::Str(_) => out.push(0xc0), // over-long strings refused: nil
        MpValue::Bin(b) if b.len() < 256 => {
            out.push(0xc4);
            out.push(b.len() as u8);
            out.extend_from_slice(b);
        }
        MpValue::Bin(_) => out.push(0xc0),
        MpValue::Array(items) if items.len() < 16 => {
            out.push(0x90 | items.len() as u8);
            for i in items {
                mp_encode_into(out, i);
            }
        }
        MpValue::Array(_) => out.push(0xc0),
        MpValue::Map(items) if items.len() < 16 => {
            out.push(0x80 | items.len() as u8);
            for (k, v) in items {
                mp_encode_into(out, &MpValue::Str(k.clone()));
                mp_encode_into(out, v);
            }
        }
        MpValue::Map(_) => out.push(0xc0),
    }
}

pub fn msgpack_encode(v: &MpValue) -> Vec<u8> {
    let mut out = Vec::new();
    mp_encode_into(&mut out, v);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_roundtrip() {
        for v in [0u64, 1, 127, 128, 300, u64::MAX] {
            let e = varint_encode(v);
            assert_eq!(varint_decode(&e), Some((v, e.len())));
        }
        assert_eq!(varint_decode(&[]), None);
    }

    #[test]
    fn frame_limits() {
        let encoded = frame_encode(b"hi").unwrap();
        let (payload, consumed) = frame_decode(&encoded).unwrap();
        assert_eq!((payload, consumed), (b"hi".as_slice(), 3));
        assert_eq!(frame_decode(&[0xff, 0xff, 0xff, 0xff, 0x0f]), None);
    }

    #[test]
    fn msgpack_roundtrip() {
        let v = MpValue::Map(vec![
            ("a".to_string(), MpValue::Int(1)),
            ("b".to_string(), MpValue::Array(vec![MpValue::Bool(true), MpValue::Nil])),
        ]);
        assert_eq!(msgpack_decode(&msgpack_encode(&v)), Some(v));
    }

    #[test]
    fn msgpack_rejects_garbage_and_depth() {
        // 0xC1 is never-used in MessagePack; 0xFF is valid (-1).
        assert_eq!(msgpack_decode(&[0xc1]), None);
        assert_eq!(msgpack_decode(&[0xff]), Some(MpValue::Int(-1)));
        // fixmap claiming 15 pairs but truncated.
        assert_eq!(msgpack_decode(&[0x8f, 0xa1]), None);
    }
}
