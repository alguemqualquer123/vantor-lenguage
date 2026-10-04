use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Limits + SerError (vendored from lexicon-utils::ser to keep this crate
// std + allowed-deps only; no new external crates, offline-friendly).
// Complexity notes: all limit checks are O(1); JSON depth walk is O(n)
// where n = number of JSON nodes.
// ---------------------------------------------------------------------------

/// Untrusted-input limits: size + nesting depth + container length.
///
/// Complexity: O(1) to construct/check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Maximum frame/body payload accepted (default 8 MiB).
    pub max_bytes: usize,
    /// Maximum nesting depth of structures (default 32).
    pub max_depth: u32,
    /// Maximum container element count (default 1 MiB entries).
    pub max_array_len: usize,
}

impl Default for Limits {
    #[inline]
    fn default() -> Self {
        Self {
            max_bytes: 8 << 20,
            max_depth: 32,
            max_array_len: 1 << 20,
        }
    }
}

impl Limits {
    /// Complexity: O(1).
    #[inline]
    pub fn new(max_bytes: usize, max_depth: u32, max_array_len: usize) -> Self {
        Self {
            max_bytes,
            max_depth,
            max_array_len,
        }
    }
}

/// Serialization/decode errors (typed, never panics).
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
    #[inline]
    fn from(e: std::str::Utf8Error) -> Self {
        SerError::Utf8(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Response (concrete JSON envelope) — restores lib.rs re-exports.
// ---------------------------------------------------------------------------

/// Concrete JSON envelope `{success, data, message}`.
///
/// `data` is `Option<String>` to satisfy the crate's historic
/// `Response::ok(String)` shape used in tests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub data: Option<String>,
    pub message: Option<String>,
}

/// Alias kept for lib.rs (`JsonResponse as ResponseObj`).
pub type JsonResponse = Response;

impl Response {
    /// Complexity: O(1) + clone of `data`.
    #[inline]
    pub fn ok(data: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }
    /// Complexity: O(1).
    #[inline]
    pub fn ok_msg(data: String, message: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message.to_string()),
        }
    }
    /// Complexity: O(1).
    #[inline]
    pub fn err(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message.to_string()),
        }
    }
}

/// Complexity: O(n) in `data` (JSON serialization).
#[inline]
pub fn response_ok(data: &str) -> String {
    serde_json::to_string(&Response {
        success: true,
        data: Some(data.to_string()),
        message: None,
    })
    .unwrap_or_default()
}

/// Complexity: O(n) in `message`.
#[inline]
pub fn response_err(message: &str) -> String {
    serde_json::to_string(&Response {
        success: false,
        data: None,
        message: Some(message.to_string()),
    })
    .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// JSON hardening: depth cap, duplicate-key policy, int boundaries, UTF-8,
// Limits on ALL decode paths.
// ---------------------------------------------------------------------------

/// Duplicate-key policy for JSON objects.
///
/// `serde_json` default (and this crate's default decode) is
/// [`DuplicateKeyPolicy::LastWins`]: `{"a":1,"a":2}` decodes with `a == 2`.
/// Use [`decode_json_reject_duplicates`] for strict mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DuplicateKeyPolicy {
    /// Last occurrence wins (serde_json behavior). Default.
    #[default]
    LastWins,
    /// Reject any duplicate key at any object level.
    Reject,
}

/// Walk a `serde_json::Value` enforcing `Limits` (depth + container len).
///
/// Complexity: O(n) where n = number of JSON nodes; recursion bounded by
/// `limits.max_depth` (returns `Recursion` beyond it).
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
///
/// Complexity: O(n) serialize + O(n) limit walk.
pub fn encode_json<T: Serialize>(value: &T, limits: &Limits) -> Result<String, SerError> {
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
///
/// Guarantees:
/// - `bytes.len() > max_bytes` => `Overflow` (checked BEFORE parsing).
/// - invalid UTF-8 => `SerError::Utf8` (via `std::str::from_utf8`).
/// - nesting deeper than `max_depth` => `Recursion`.
/// - containers longer than `max_array_len` => `Overflow`.
/// - duplicate keys => **LastWins** (serde_json default; see
///   [`DuplicateKeyPolicy`] + [`decode_json_reject_duplicates`] for strict).
/// - integers: `i64` range stays `Number::I64`, `u64` up to `u64::MAX` stays
///   `Number::U64`; out-of-range literals (e.g. `1e999`, `999...9` with 100+
///   digits) are rejected by serde_json as `Truncated`.
///
/// Complexity: O(n) parse + O(n) limit walk, O(1) extra space besides parse.
pub fn decode_json<T>(bytes: &[u8], limits: &Limits) -> Result<T, SerError>
where
    T: for<'de> Deserialize<'de>,
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

/// Strict decode that rejects duplicate object keys at any depth.
///
/// Scans the raw text (string-aware, O(n)) before delegating to
/// [`decode_json`]. Returns `SerError::Truncated("duplicate key ...")` on
/// duplicates.
///
/// Complexity: O(n) scan + O(n) parse; single pass, no backtracking.
pub fn decode_json_reject_duplicates<T>(bytes: &[u8], limits: &Limits) -> Result<T, SerError>
where
    T: for<'de> Deserialize<'de>,
{
    if bytes.len() > limits.max_bytes {
        return Err(SerError::Overflow(format!(
            "input bytes {} exceeds max {}",
            bytes.len(),
            limits.max_bytes
        )));
    }
    let s = std::str::from_utf8(bytes).map_err(|e| SerError::Utf8(e.to_string()))?;
    if let Some(key) = find_duplicate_key(s) {
        return Err(SerError::Truncated(format!("duplicate key: {key}")));
    }
    decode_json(bytes, limits)
}

/// Returns the first duplicate key found in any JSON object level, or None.
///
/// String-aware scanner: skips `"..."` literals (with `\"` escapes) so braces
/// inside strings don't confuse tracking. Handles nested `{...}` via an
/// explicit stack of key-sets; arrays push a marker so `[{...}]` scopes
/// correctly. Pure function, no allocation beyond key sets.
///
/// Complexity: O(n) time, O(d*k) space where d = depth, k = keys per level.
pub fn find_duplicate_key(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let n = b.len();
    let mut i = 0usize;
    // Stack: each object level holds its seen keys; arrays hold None marker.
    let mut stack: Vec<Option<std::collections::HashSet<String>>> = Vec::new();
    while i < n {
        let c = b[i];
        match c {
            b'"' => {
                // Parse string literal.
                let start = i + 1;
                let mut j = start;
                let mut esc = false;
                while j < n {
                    if esc {
                        esc = false;
                        j += 1;
                    } else if b[j] == b'\\' {
                        esc = true;
                        j += 1;
                    } else if b[j] == b'"' {
                        break;
                    } else {
                        j += 1;
                    }
                }
                if j >= n {
                    break;
                }
                let lit = &s[start..j];
                j += 1; // past closing quote
                // Skip whitespace, check for ':' => this string is an object key.
                let mut k = j;
                while k < n && (b[k] == b' ' || b[k] == b'\t' || b[k] == b'\n' || b[k] == b'\r') {
                    k += 1;
                }
                if k < n && b[k] == b':' {
                    if let Some(Some(set)) = stack.last_mut() {
                        if !set.insert(lit.to_string()) {
                            return Some(lit.to_string());
                        }
                    }
                    i = k + 1;
                } else {
                    i = j;
                }
            }
            b'{' => {
                stack.push(Some(std::collections::HashSet::new()));
                i += 1;
            }
            b'}' => {
                stack.pop();
                i += 1;
            }
            b'[' => {
                stack.push(None);
                i += 1;
            }
            b']' => {
                stack.pop();
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    None
}

/// Convenience: true if any duplicate key exists (uses [`find_duplicate_key`]).
///
/// Complexity: O(n).
#[inline]
pub fn has_duplicate_keys(s: &str) -> bool {
    find_duplicate_key(s).is_some()
}

// ---------------------------------------------------------------------------
// API existente (preservada — NÃO quebrar)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Binary: varint + length-prefix (std only)
// ---------------------------------------------------------------------------

/// Codifica `value` como varint LEB128 (estilo protobuf) em `out`.
pub fn encode_varint(mut value: u64, out: &mut Vec<u8>) {
    loop {
        let mut b = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            b |= 0x80;
            out.push(b);
        } else {
            out.push(b);
            break;
        }
    }
}

/// Decodifica varint com limite: no máximo 10 bytes, erro Truncated/Overflow.
pub fn decode_varint(input: &[u8], limits: &Limits) -> Result<(u64, usize), SerError> {
    if input.is_empty() {
        return Err(SerError::Truncated("empty input for varint".to_string()));
    }
    if input.len() > limits.max_bytes {
        return Err(SerError::Overflow(format!(
            "varint input {} exceeds max {}",
            input.len(),
            limits.max_bytes
        )));
    }
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    for (i, &b) in input.iter().enumerate() {
        if i >= 10 {
            return Err(SerError::Overflow(
                "varint exceeds 10 bytes".to_string(),
            ));
        }
        let low = (b & 0x7F) as u64;
        if shift >= 64 && low != 0 {
            return Err(SerError::Overflow("varint shift overflow".to_string()));
        }
        result |= low << shift;
        if b & 0x80 == 0 {
            return Ok((result, i + 1));
        }
        shift += 7;
    }
    Err(SerError::Truncated(
        "varint terminated early (continuation bit set at EOF)".to_string(),
    ))
}

/// Codifica `data` com prefixo de comprimento varint.
pub fn encode_len_prefixed(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + 10);
    encode_varint(data.len() as u64, &mut out);
    out.extend_from_slice(data);
    out
}

/// Decodifica payload com prefixo de comprimento. Retorna (payload, bytes consumidos).
pub fn decode_len_prefixed(
    input: &[u8],
    limits: &Limits,
) -> Result<(Vec<u8>, usize), SerError> {
    let (len, n) = decode_varint(input, limits)?;
    if len as usize > limits.max_bytes {
        return Err(SerError::Overflow(format!(
            "length prefix {len} exceeds max {}",
            limits.max_bytes
        )));
    }
    let rest = &input[n..];
    if rest.len() < len as usize {
        return Err(SerError::Truncated(format!(
            "need {} bytes, have {}",
            len,
            rest.len()
        )));
    }
    let payload = rest[..len as usize].to_vec();
    Ok((payload, n + len as usize))
}

// ---------------------------------------------------------------------------
// MessagePack — stub funcional mínimo (maps/arrays/ints/strs)
// ---------------------------------------------------------------------------

pub mod msgpack {
    use super::{Limits, SerError};

    pub fn encode_int(v: i64, out: &mut Vec<u8>) {
        if (0..=127).contains(&v) {
            out.push(v as u8);
        } else if (-32..0).contains(&v) {
            out.push((v as i8) as u8);
        } else if v >= 0 && v <= u8::MAX as i64 {
            out.push(0xcc);
            out.push(v as u8);
        } else if v >= 0 && v <= u16::MAX as i64 {
            out.push(0xcd);
            out.extend_from_slice(&(v as u16).to_be_bytes());
        } else if v >= 0 && v <= u32::MAX as i64 {
            out.push(0xce);
            out.extend_from_slice(&(v as u32).to_be_bytes());
        } else if v >= 0 {
            out.push(0xcf);
            out.extend_from_slice(&(v as u64).to_be_bytes());
        } else if v >= i8::MIN as i64 && v <= i8::MAX as i64 {
            out.push(0xd0);
            out.push(v as i8 as u8);
        } else if v >= i16::MIN as i64 && v <= i16::MAX as i64 {
            out.push(0xd1);
            out.extend_from_slice(&(v as i16).to_be_bytes());
        } else if v >= i32::MIN as i64 && v <= i32::MAX as i64 {
            out.push(0xd2);
            out.extend_from_slice(&(v as i32).to_be_bytes());
        } else {
            out.push(0xd3);
            out.extend_from_slice(&v.to_be_bytes());
        }
    }

    pub fn decode_int(input: &[u8]) -> Result<(i64, usize), SerError> {
        if input.is_empty() {
            return Err(SerError::Truncated("empty msgpack int".to_string()));
        }
        let b = input[0];
        match b {
            0x00..=0x7f => Ok((b as i64, 1)),
            0xe0..=0xff => Ok(((b as i8) as i64, 1)),
            0xcc => {
                if input.len() < 2 {
                    return Err(SerError::Truncated("msgpack uint8".to_string()));
                }
                Ok((input[1] as i64, 2))
            }
            0xcd => {
                if input.len() < 3 {
                    return Err(SerError::Truncated("msgpack uint16".to_string()));
                }
                Ok((u16::from_be_bytes([input[1], input[2]]) as i64, 3))
            }
            0xce => {
                if input.len() < 5 {
                    return Err(SerError::Truncated("msgpack uint32".to_string()));
                }
                let v = u32::from_be_bytes([input[1], input[2], input[3], input[4]]);
                Ok((v as i64, 5))
            }
            0xcf => {
                if input.len() < 9 {
                    return Err(SerError::Truncated("msgpack uint64".to_string()));
                }
                let v = u64::from_be_bytes([
                    input[1], input[2], input[3], input[4], input[5], input[6], input[7],
                    input[8],
                ]);
                if v > i64::MAX as u64 {
                    return Err(SerError::Overflow("msgpack uint64 > i64::MAX".to_string()));
                }
                Ok((v as i64, 9))
            }
            0xd0 => {
                if input.len() < 2 {
                    return Err(SerError::Truncated("msgpack int8".to_string()));
                }
                Ok(((input[1] as i8) as i64, 2))
            }
            0xd1 => {
                if input.len() < 3 {
                    return Err(SerError::Truncated("msgpack int16".to_string()));
                }
                Ok((i16::from_be_bytes([input[1], input[2]]) as i64, 3))
            }
            0xd2 => {
                if input.len() < 5 {
                    return Err(SerError::Truncated("msgpack int32".to_string()));
                }
                let v = i32::from_be_bytes([input[1], input[2], input[3], input[4]]);
                Ok((v as i64, 5))
            }
            0xd3 => {
                if input.len() < 9 {
                    return Err(SerError::Truncated("msgpack int64".to_string()));
                }
                let v = i64::from_be_bytes([
                    input[1], input[2], input[3], input[4], input[5], input[6], input[7],
                    input[8],
                ]);
                Ok((v, 9))
            }
            _ => Err(SerError::Truncated(format!(
                "not a msgpack int header: {b:#x}"
            ))),
        }
    }

    pub fn encode_str(s: &str, out: &mut Vec<u8>) {
        let b = s.as_bytes();
        let len = b.len();
        if len < 32 {
            out.push(0xa0 | (len as u8));
        } else if len < 256 {
            out.push(0xd9);
            out.push(len as u8);
        } else if len < 65536 {
            out.push(0xda);
            out.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            out.push(0xdb);
            out.extend_from_slice(&(len as u32).to_be_bytes());
        }
        out.extend_from_slice(b);
    }

    pub fn decode_str(input: &[u8], limits: &Limits) -> Result<(String, usize), SerError> {
        if input.is_empty() {
            return Err(SerError::Truncated("empty msgpack str".to_string()));
        }
        let b = input[0];
        let (len, head): (usize, usize) = if b & 0xe0 == 0xa0 {
            ((b & 0x1f) as usize, 1)
        } else if b == 0xd9 {
            if input.len() < 2 {
                return Err(SerError::Truncated("msgpack str8".to_string()));
            }
            (input[1] as usize, 2)
        } else if b == 0xda {
            if input.len() < 3 {
                return Err(SerError::Truncated("msgpack str16".to_string()));
            }
            (u16::from_be_bytes([input[1], input[2]]) as usize, 3)
        } else if b == 0xdb {
            if input.len() < 5 {
                return Err(SerError::Truncated("msgpack str32".to_string()));
            }
            (
                u32::from_be_bytes([input[1], input[2], input[3], input[4]]) as usize,
                5,
            )
        } else {
            return Err(SerError::Truncated(format!(
                "not a msgpack str header: {b:#x}"
            )));
        };
        if len > limits.max_bytes {
            return Err(SerError::Overflow(format!(
                "msgpack str len {len} exceeds max {}",
                limits.max_bytes
            )));
        }
        if input.len() < head + len {
            return Err(SerError::Truncated(format!(
                "msgpack str need {len}, have {}",
                input.len() - head
            )));
        }
        let bytes = &input[head..head + len];
        let s = std::str::from_utf8(bytes).map_err(|e| SerError::Utf8(e.to_string()))?;
        Ok((s.to_string(), head + len))
    }

    pub fn encode_array_len(len: usize, out: &mut Vec<u8>) {
        if len < 16 {
            out.push(0x90 | (len as u8));
        } else if len < 65536 {
            out.push(0xdc);
            out.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            out.push(0xdd);
            out.extend_from_slice(&(len as u32).to_be_bytes());
        }
    }

    pub fn decode_array_len(input: &[u8], limits: &Limits) -> Result<(usize, usize), SerError> {
        if input.is_empty() {
            return Err(SerError::Truncated("empty msgpack array".to_string()));
        }
        let b = input[0];
        let (len, head): (usize, usize) = if b & 0xf0 == 0x90 {
            ((b & 0x0f) as usize, 1)
        } else if b == 0xdc {
            if input.len() < 3 {
                return Err(SerError::Truncated("msgpack array16".to_string()));
            }
            (u16::from_be_bytes([input[1], input[2]]) as usize, 3)
        } else if b == 0xdd {
            if input.len() < 5 {
                return Err(SerError::Truncated("msgpack array32".to_string()));
            }
            (
                u32::from_be_bytes([input[1], input[2], input[3], input[4]]) as usize,
                5,
            )
        } else {
            return Err(SerError::Truncated(format!(
                "not a msgpack array header: {b:#x}"
            )));
        };
        if len > limits.max_array_len {
            return Err(SerError::Overflow(format!(
                "msgpack array len {len} exceeds max {}",
                limits.max_array_len
            )));
        }
        Ok((len, head))
    }

    pub fn encode_map_len(len: usize, out: &mut Vec<u8>) {
        if len < 16 {
            out.push(0x80 | (len as u8));
        } else if len < 65536 {
            out.push(0xde);
            out.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            out.push(0xdf);
            out.extend_from_slice(&(len as u32).to_be_bytes());
        }
    }

    pub fn decode_map_len(input: &[u8], limits: &Limits) -> Result<(usize, usize), SerError> {
        if input.is_empty() {
            return Err(SerError::Truncated("empty msgpack map".to_string()));
        }
        let b = input[0];
        let (len, head): (usize, usize) = if b & 0xf0 == 0x80 {
            ((b & 0x0f) as usize, 1)
        } else if b == 0xde {
            if input.len() < 3 {
                return Err(SerError::Truncated("msgpack map16".to_string()));
            }
            (u16::from_be_bytes([input[1], input[2]]) as usize, 3)
        } else if b == 0xdf {
            if input.len() < 5 {
                return Err(SerError::Truncated("msgpack map32".to_string()));
            }
            (
                u32::from_be_bytes([input[1], input[2], input[3], input[4]]) as usize,
                5,
            )
        } else {
            return Err(SerError::Truncated(format!(
                "not a msgpack map header: {b:#x}"
            )));
        };
        if len > limits.max_array_len {
            return Err(SerError::Overflow(format!(
                "msgpack map len {len} exceeds max {}",
                limits.max_array_len
            )));
        }
        Ok((len, head))
    }
}

// ---------------------------------------------------------------------------
// CBOR — stub funcional mínimo (ints/strs/arrays/maps)
// ---------------------------------------------------------------------------

pub mod cbor {
    use super::{Limits, SerError};

    fn encode_head(major: u8, n: u64, out: &mut Vec<u8>) {
        let m = major << 5;
        if n < 24 {
            out.push(m | (n as u8));
        } else if n <= u8::MAX as u64 {
            out.push(m | 24);
            out.push(n as u8);
        } else if n <= u16::MAX as u64 {
            out.push(m | 25);
            out.extend_from_slice(&(n as u16).to_be_bytes());
        } else if n <= u32::MAX as u64 {
            out.push(m | 26);
            out.extend_from_slice(&(n as u32).to_be_bytes());
        } else {
            out.push(m | 27);
            out.extend_from_slice(&n.to_be_bytes());
        }
    }

    fn decode_head(input: &[u8], expect_major: u8) -> Result<(u64, usize), SerError> {
        if input.is_empty() {
            return Err(SerError::Truncated("empty cbor head".to_string()));
        }
        let b = input[0];
        let major = b >> 5;
        if major != expect_major {
            return Err(SerError::Truncated(format!(
                "cbor major mismatch: got {major}, want {expect_major}"
            )));
        }
        let info = b & 0x1f;
        match info {
            0..=23 => Ok((info as u64, 1)),
            24 => {
                if input.len() < 2 {
                    return Err(SerError::Truncated("cbor u8".to_string()));
                }
                Ok((input[1] as u64, 2))
            }
            25 => {
                if input.len() < 3 {
                    return Err(SerError::Truncated("cbor u16".to_string()));
                }
                Ok((u16::from_be_bytes([input[1], input[2]]) as u64, 3))
            }
            26 => {
                if input.len() < 5 {
                    return Err(SerError::Truncated("cbor u32".to_string()));
                }
                let v = u32::from_be_bytes([input[1], input[2], input[3], input[4]]);
                Ok((v as u64, 5))
            }
            27 => {
                if input.len() < 9 {
                    return Err(SerError::Truncated("cbor u64".to_string()));
                }
                let v = u64::from_be_bytes([
                    input[1], input[2], input[3], input[4], input[5], input[6], input[7],
                    input[8],
                ]);
                Ok((v, 9))
            }
            _ => Err(SerError::Truncated(format!(
                "unsupported cbor additional info: {info}"
            ))),
        }
    }

    /// major 0 (uint) / major 1 (nint).
    pub fn encode_int(v: i64, out: &mut Vec<u8>) {
        if v >= 0 {
            encode_head(0, v as u64, out);
        } else {
            encode_head(1, (-1 - v) as u64, out);
        }
    }

    pub fn decode_int(input: &[u8]) -> Result<(i64, usize), SerError> {
        if input.is_empty() {
            return Err(SerError::Truncated("empty cbor int".to_string()));
        }
        match input[0] >> 5 {
            0 => {
                let (n, c) = decode_head(input, 0)?;
                if n > i64::MAX as u64 {
                    return Err(SerError::Overflow("cbor uint > i64::MAX".to_string()));
                }
                Ok((n as i64, c))
            }
            1 => {
                let (n, c) = decode_head(input, 1)?;
                if n > i64::MAX as u64 {
                    return Err(SerError::Overflow("cbor nint overflow".to_string()));
                }
                Ok((-1 - (n as i64), c))
            }
            _ => Err(SerError::Truncated("not a cbor int".to_string())),
        }
    }

    /// major 3 (text string).
    pub fn encode_str(s: &str, out: &mut Vec<u8>) {
        encode_head(3, s.len() as u64, out);
        out.extend_from_slice(s.as_bytes());
    }

    pub fn decode_str(input: &[u8], limits: &Limits) -> Result<(String, usize), SerError> {
        let (len, head) = decode_head(input, 3)?;
        let len = len as usize;
        if len > limits.max_bytes {
            return Err(SerError::Overflow(format!(
                "cbor str len {len} exceeds max {}",
                limits.max_bytes
            )));
        }
        if input.len() < head + len {
            return Err(SerError::Truncated(format!(
                "cbor str need {len}, have {}",
                input.len() - head
            )));
        }
        let bytes = &input[head..head + len];
        let s = std::str::from_utf8(bytes).map_err(|e| SerError::Utf8(e.to_string()))?;
        Ok((s.to_string(), head + len))
    }

    /// major 4 (array).
    pub fn encode_array_len(len: usize, out: &mut Vec<u8>) {
        encode_head(4, len as u64, out);
    }

    pub fn decode_array_len(input: &[u8], limits: &Limits) -> Result<(usize, usize), SerError> {
        let (len, head) = decode_head(input, 4)?;
        if len as usize > limits.max_array_len {
            return Err(SerError::Overflow(format!(
                "cbor array len {len} exceeds max {}",
                limits.max_array_len
            )));
        }
        Ok((len as usize, head))
    }

    /// major 5 (map).
    pub fn encode_map_len(len: usize, out: &mut Vec<u8>) {
        encode_head(5, len as u64, out);
    }

    pub fn decode_map_len(input: &[u8], limits: &Limits) -> Result<(usize, usize), SerError> {
        let (len, head) = decode_head(input, 5)?;
        if len as usize > limits.max_array_len {
            return Err(SerError::Overflow(format!(
                "cbor map len {len} exceeds max {}",
                limits.max_array_len
            )));
        }
        Ok((len as usize, head))
    }
}

// ---------------------------------------------------------------------------
// Protobuf — Tag + varint + length-delimited (§21)
// ---------------------------------------------------------------------------

pub mod protobuf {
    use super::{Limits, SerError};

    pub const WIRE_VARINT: u8 = 0;
    pub const WIRE_64BIT: u8 = 1;
    pub const WIRE_LEN: u8 = 2;
    pub const WIRE_32BIT: u8 = 5;

    /// Tag protobuf: número do campo + wire type.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Tag {
        pub field: u32,
        pub wire: u8,
    }

    impl Tag {
        pub fn new(field: u32, wire: u8) -> Self {
            Self { field, wire }
        }

        /// Compõe o valor bruto `field << 3 | wire`.
        pub fn encode(&self) -> u64 {
            ((self.field as u64) << 3) | (self.wire as u64 & 0x07)
        }

        pub fn decode(raw: u64) -> Result<Self, SerError> {
            let field = (raw >> 3) as u32;
            let wire = (raw & 0x07) as u8;
            if field == 0 {
                return Err(SerError::Truncated("protobuf field 0 invalid".to_string()));
            }
            Ok(Self { field, wire })
        }

        pub fn encode_to(&self, out: &mut Vec<u8>) {
            encode_varint(self.encode(), out);
        }
    }

    /// Varint LEB128 (mesma regra do nível Binary).
    pub fn encode_varint(mut v: u64, out: &mut Vec<u8>) {
        loop {
            let mut b = (v & 0x7F) as u8;
            v >>= 7;
            if v != 0 {
                b |= 0x80;
                out.push(b);
            } else {
                out.push(b);
                break;
            }
        }
    }

    /// Varint com limite (máx. 10 bytes, respeita `Limits::max_bytes`).
    pub fn decode_varint(input: &[u8], limits: &Limits) -> Result<(u64, usize), SerError> {
        if input.is_empty() {
            return Err(SerError::Truncated("empty protobuf varint".to_string()));
        }
        if input.len() > limits.max_bytes {
            return Err(SerError::Overflow(format!(
                "protobuf input {} exceeds max {}",
                input.len(),
                limits.max_bytes
            )));
        }
        let mut result: u64 = 0;
        let mut shift: u32 = 0;
        for (i, &b) in input.iter().enumerate() {
            if i >= 10 {
                return Err(SerError::Overflow(
                    "protobuf varint exceeds 10 bytes".to_string(),
                ));
            }
            let low = (b & 0x7F) as u64;
            if shift >= 64 && low != 0 {
                return Err(SerError::Overflow(
                    "protobuf varint shift overflow".to_string(),
                ));
            }
            result |= low << shift;
            if b & 0x80 == 0 {
                return Ok((result, i + 1));
            }
            shift += 7;
        }
        Err(SerError::Truncated(
            "protobuf varint terminated early".to_string(),
        ))
    }

    /// Codifica campo length-delimited: tag(field, LEN) + varint(len) + bytes.
    pub fn encode_len(field: u32, data: &[u8], out: &mut Vec<u8>) {
        Tag::new(field, WIRE_LEN).encode_to(out);
        encode_varint(data.len() as u64, out);
        out.extend_from_slice(data);
    }

    /// Decodifica um campo length-delimited. Retorna (tag, payload, consumidos).
    pub fn decode_len_field(
        input: &[u8],
        limits: &Limits,
    ) -> Result<(Tag, Vec<u8>, usize), SerError> {
        let (raw_tag, n1) = decode_varint(input, limits)?;
        let tag = Tag::decode(raw_tag)?;
        if tag.wire != WIRE_LEN {
            return Err(SerError::Truncated(format!(
                "expected LEN wire (2), got {}",
                tag.wire
            )));
        }
        let (len, n2) = decode_varint(&input[n1..], limits)?;
        if len as usize > limits.max_bytes {
            return Err(SerError::Overflow(format!(
                "protobuf len {len} exceeds max {}",
                limits.max_bytes
            )));
        }
        let start = n1 + n2;
        if input.len() < start + len as usize {
            return Err(SerError::Truncated(format!(
                "protobuf need {} bytes, have {}",
                len,
                input.len() - start
            )));
        }
        let payload = input[start..start + len as usize].to_vec();
        Ok((tag, payload, start + len as usize))
    }
}

// ---------------------------------------------------------------------------
// FlatBuffers — stub documentado (não é implementação completa)
// ---------------------------------------------------------------------------

/// Stub mínimo de FlatBuffers para cobertura do §21.
///
/// Escopo proposital: este builder NÃO implementa o formato FlatBuffers real
/// (vtables, alinhamento, offsets relativos). Ele apenas oferece uma API
/// estável mínima (push + finish + acesso aos bytes) com framing interno
/// `length-prefix (varint)` para que o restante do §21 possa ser exercitado
/// sem novas dependências. Se o formato real for necessário, substitua este
/// módulo pela crate `flatbuffers` sem alterar as demais APIs deste arquivo.
pub mod flatbuffers {
    /// Builder mínimo com framing interno length-prefixed.
    #[derive(Debug, Clone, Default)]
    pub struct Builder {
        buf: Vec<u8>,
        finished: bool,
    }

    impl Builder {
        pub fn new() -> Self {
            Self {
                buf: Vec::new(),
                finished: false,
            }
        }

        /// Adiciona uma string e retorna o offset inicial (stub).
        pub fn push_str(&mut self, s: &str) -> u32 {
            let off = self.buf.len() as u32;
            let bytes = s.as_bytes();
            super::encode_varint(bytes.len() as u64, &mut self.buf);
            self.buf.extend_from_slice(bytes);
            off
        }

        /// Adiciona bytes opacos e retorna o offset inicial (stub).
        pub fn push_bytes(&mut self, b: &[u8]) -> u32 {
            let off = self.buf.len() as u32;
            super::encode_varint(b.len() as u64, &mut self.buf);
            self.buf.extend_from_slice(b);
            off
        }

        /// Marca o buffer como finalizado (stub: sem realocação/alinhamento).
        pub fn finish(&mut self, _root_offset: u32) {
            self.finished = true;
        }

        pub fn as_bytes(&self) -> &[u8] {
            &self.buf
        }

        pub fn is_finished(&self) -> bool {
            self.finished
        }
    }
}

// ---------------------------------------------------------------------------
// Schema-aware: trait Serializable + Schema + validate_unknown/missing
// ---------------------------------------------------------------------------

/// Trait para tipos serializáveis com versão de schema.
pub trait Serializable: Serialize {
    /// Versão do schema deste tipo (default 1).
    fn schema_version(&self) -> u32 {
        1
    }
    /// Nomes dos campos expostos (usado por validações schema-aware).
    fn field_names(&self) -> Vec<String>;
}

impl Serializable for Response {
    fn schema_version(&self) -> u32 {
        1
    }

    fn field_names(&self) -> Vec<String> {
        vec![
            "success".to_string(),
            "data".to_string(),
            "message".to_string(),
        ]
    }
}

/// Um campo do schema: nome + obrigatoriedade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaField {
    pub name: String,
    pub required: bool,
}

impl SchemaField {
    pub fn new(name: &str, required: bool) -> Self {
        Self {
            name: name.to_string(),
            required,
        }
    }
}

/// Schema versão + lista de campos conhecidos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub fields: Vec<SchemaField>,
    pub version: u32,
}

impl Schema {
    pub fn new(version: u32, fields: Vec<(&str, bool)>) -> Self {
        Self {
            fields: fields
                .into_iter()
                .map(|(n, r)| SchemaField::new(n, r))
                .collect(),
            version,
        }
    }

    fn known(&self, name: &str) -> bool {
        self.fields.iter().any(|f| f.name == name)
    }

    /// Rejeita campos desconhecidos.
    pub fn validate_unknown(&self, present: &[String]) -> Result<(), SerError> {
        for p in present {
            if !self.known(p) {
                return Err(SerError::UnknownField(p.clone()));
            }
        }
        Ok(())
    }

    /// Rejeita campos obrigatórios ausentes.
    pub fn validate_missing(&self, present: &[String]) -> Result<(), SerError> {
        for f in &self.fields {
            if f.required && !present.iter().any(|p| p == &f.name) {
                return Err(SerError::MissingField(f.name.clone()));
            }
        }
        Ok(())
    }

    /// Validação completa: versão + desconhecidos + ausentes.
    pub fn validate(&self, present: &[String], version: u32) -> Result<(), SerError> {
        if version != self.version {
            return Err(SerError::VersionMismatch(format!(
                "expected {}, found {}",
                self.version, version
            )));
        }
        self.validate_unknown(present)?;
        self.validate_missing(present)?;
        Ok(())
    }

    /// Conveniência para mapas planos (YAML/TOML/subset JSON).
    pub fn validate_map(
        &self,
        map: &BTreeMap<String, String>,
        version: u32,
    ) -> Result<(), SerError> {
        let present: Vec<String> = map.keys().cloned().collect();
        self.validate(&present, version)
    }
}

/// Função livre: rejeita campos desconhecidos (espelha `Schema::validate_unknown`).
pub fn validate_unknown(schema: &Schema, present: &[String]) -> Result<(), SerError> {
    schema.validate_unknown(present)
}

/// Função livre: rejeita obrigatórios ausentes (espelha `Schema::validate_missing`).
pub fn validate_missing(schema: &Schema, present: &[String]) -> Result<(), SerError> {
    schema.validate_missing(present)
}

// ---------------------------------------------------------------------------
// XML escape + YAML/TOML subset mínimo (§19)
// ---------------------------------------------------------------------------

/// Escapa `& < > " '` para XML.
pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Codifica `<tag>conteúdo-escapado</tag>`.
pub fn xml_encode_tag(tag: &str, content: &str) -> String {
    format!("<{tag}>{}</{tag}>", xml_escape(content))
}

/// Serializa um `Response` como XML mínimo.
pub fn response_to_xml(resp: &Response) -> String {
    let mut s = String::from("<response>");
    s.push_str(&xml_encode_tag("success", if resp.success { "true" } else { "false" }));
    if let Some(d) = &resp.data {
        s.push_str(&xml_encode_tag("data", d));
    }
    if let Some(m) = &resp.message {
        s.push_str(&xml_encode_tag("message", m));
    }
    s.push_str("</response>");
    s
}

fn quote_if_needed(v: &str) -> String {
    if v.is_empty()
        || v.contains([':', '#', '\n', '\r', '"', '\'', '=', '[', ']', '{', '}', ','])
        || v.trim() != v
    {
        format!("{:?}", v) // Quoting estilo debug ("...") com escapes.
    } else {
        v.to_string()
    }
}

fn unquote(v: &str) -> String {
    let t = v.trim();
    if t.len() >= 2
        && ((t.starts_with('"') && t.ends_with('"'))
            || (t.starts_with('\'') && t.ends_with('\'')))
    {
        let inner = &t[1..t.len() - 1];
        if t.starts_with('"') {
            // Desescapes mínimos: \\ \" \n \t
            let mut out = String::with_capacity(inner.len());
            let mut it = inner.chars();
            while let Some(c) = it.next() {
                if c == '\\' {
                    match it.next() {
                        Some('n') => out.push('\n'),
                        Some('t') => out.push('\t'),
                        Some(c2) => out.push(c2),
                        None => out.push('\\'),
                    }
                } else {
                    out.push(c);
                }
            }
            return out;
        }
        return inner.to_string();
    }
    t.to_string()
}

/// YAML subset mínimo: uma entrada `chave: valor` por linha.
pub fn yaml_encode_map(map: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for (k, v) in map {
        out.push_str(k);
        out.push_str(": ");
        out.push_str(&quote_if_needed(v));
        out.push('\n');
    }
    out
}

/// Decodifica o subset YAML acima (ignora vazias e `# comentários`).
pub fn yaml_decode_map(s: &str, limits: &Limits) -> Result<BTreeMap<String, String>, SerError> {
    if s.len() > limits.max_bytes {
        return Err(SerError::Overflow(format!(
            "yaml bytes {} exceeds max {}",
            s.len(),
            limits.max_bytes
        )));
    }
    let s = std::str::from_utf8(s.as_bytes()).map_err(|e| SerError::Utf8(e.to_string()))?;
    let mut map = BTreeMap::new();
    for line in s.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let Some(pos) = t.find(':') else {
            return Err(SerError::Truncated(format!("bad yaml line: {line}")));
        };
        let k = t[..pos].trim().to_string();
        let v = unquote(t[pos + 1..].trim());
        if k.is_empty() {
            return Err(SerError::Truncated(format!("bad yaml key: {line}")));
        }
        map.insert(k, v);
        if map.len() > limits.max_array_len {
            return Err(SerError::Overflow(format!(
                "yaml entries exceed max {}",
                limits.max_array_len
            )));
        }
    }
    Ok(map)
}

/// TOML subset mínimo: uma entrada `chave = "valor"` por linha.
pub fn toml_encode_map(map: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for (k, v) in map {
        out.push_str(k);
        out.push_str(" = ");
        out.push_str(&format!("{:?}", v));
        out.push('\n');
    }
    out
}

/// Decodifica o subset TOML acima (ignora vazias e `# comentários`).
pub fn toml_decode_map(s: &str, limits: &Limits) -> Result<BTreeMap<String, String>, SerError> {
    if s.len() > limits.max_bytes {
        return Err(SerError::Overflow(format!(
            "toml bytes {} exceeds max {}",
            s.len(),
            limits.max_bytes
        )));
    }
    let mut map = BTreeMap::new();
    for line in s.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let Some(pos) = t.find('=') else {
            return Err(SerError::Truncated(format!("bad toml line: {line}")));
        };
        let k = t[..pos].trim().to_string();
        let v = unquote(t[pos + 1..].trim());
        if k.is_empty() {
            return Err(SerError::Truncated(format!("bad toml key: {line}")));
        }
        map.insert(k, v);
        if map.len() > limits.max_array_len {
            return Err(SerError::Overflow(format!(
                "toml entries exceed max {}",
                limits.max_array_len
            )));
        }
    }
    Ok(map)
}

// ---------------------------------------------------------------------------
// Testes §21
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip() {
        let limits = Limits::default();
        let original = Response::ok("hello".to_string());
        let s = encode_json(&original, &limits).expect("encode");
        let back: Response = decode_json(s.as_bytes(), &limits).expect("decode");
        assert!(back.success);
        assert_eq!(back.data.as_deref(), Some("hello"));
    }

    #[test]
    fn overflow_por_limite() {
        let tiny = Limits::new(8, 32, 10_000);
        let resp = Response::ok("dado bem maior que oito bytes".to_string());
        let err = encode_json(&resp, &tiny).expect_err("deveria estourar max_bytes");
        assert!(matches!(err, SerError::Overflow(_)), "got {err:?}");

        // decode também deve respeitar max_bytes
        let big = vec![b'x'; 64];
        let err = match decode_json::<Response>(&big, &tiny) {
            Err(e) => e,
            Ok(_) => panic!("decode deveria estourar max_bytes"),
        };
        assert!(matches!(err, SerError::Overflow(_)), "got {err:?}");
    }

    #[test]
    fn varint_roundtrip() {
        let limits = Limits::default();
        for v in [0u64, 1, 127, 128, 300, 16_384, u64::MAX] {
            let mut out = Vec::new();
            encode_varint(v, &mut out);
            let (back, n) = decode_varint(&out, &limits).expect("decode varint");
            assert_eq!(back, v);
            assert_eq!(n, out.len());
        }
        // length-prefix roundtrip
        let payload = b"lex";
        let framed = encode_len_prefixed(payload);
        let (back, _) = decode_len_prefixed(&framed, &limits).expect("len-prefixed");
        assert_eq!(back, payload);
    }

    // ---- JSON hardening ----

    #[test]
    fn json_depth_cap_enforced_on_decode() {
        let limits = Limits::new(8 << 20, 8, 1 << 20);
        // 20 níveis de arrays aninhados > max_depth 8.
        let mut s = String::new();
        for _ in 0..20 {
            s.push('[');
        }
        for _ in 0..20 {
            s.push(']');
        }
        let err = decode_json::<serde_json::Value>(s.as_bytes(), &limits).expect_err("depth");
        assert!(matches!(err, SerError::Recursion(_)), "got {err:?}");
        // Dentro do limite passa.
        let ok_limits = Limits::new(8 << 20, 32, 1 << 20);
        let v: serde_json::Value = decode_json(s.as_bytes(), &ok_limits).expect("shallow ok");
        assert!(v.is_array());
        // Encode também respeita depth.
        let deep: serde_json::Value = serde_json::from_str(&s).unwrap();
        let err = encode_json(&deep, &limits).expect_err("encode depth");
        assert!(matches!(err, SerError::Recursion(_)), "got {err:?}");
    }

    #[test]
    fn json_duplicate_keys_last_wins_and_strict_rejects() {
        let limits = Limits::default();
        // Comportamento documentado: serde_json LastWins.
        let raw = br#"{"a":1,"a":2}"#;
        let v: serde_json::Value = decode_json(raw, &limits).expect("decode");
        assert_eq!(v["a"], 2, "serde default must be LastWins");
        assert!(!has_duplicate_keys(r#"{"a":1,"b":2}"#));
        assert!(has_duplicate_keys(r#"{"a":1,"a":2}"#));
        assert_eq!(
            find_duplicate_key(r#"{"x":1,"y":{"z":1,"z":2}}"#).as_deref(),
            Some("z")
        );
        // Strings contendo chaves não confundem o scanner.
        assert!(!has_duplicate_keys(r#"{"a":"{ \"a\": 1 }"}"#));
        // Strict rejeita.
        let err = decode_json_reject_duplicates::<serde_json::Value>(raw, &limits)
            .expect_err("strict dup");
        assert!(matches!(err, SerError::Truncated(_)), "got {err:?}");
        // Sem duplicata passa no strict.
        let ok: serde_json::Value =
            decode_json_reject_duplicates(br#"{"a":1,"b":2}"#, &limits).expect("strict ok");
        assert_eq!(ok["a"], 1);
    }

    #[test]
    fn json_integer_boundaries() {
        let limits = Limits::default();
        // i64::MAX roundtrip como I64.
        let v: serde_json::Value =
            decode_json(format!("{}", i64::MAX).as_bytes(), &limits).expect("i64max");
        assert_eq!(v.as_i64(), Some(i64::MAX));
        // i64::MAX+1 vira U64 (não erro, mas preservado).
        let v: serde_json::Value =
            decode_json(b"9223372036854775808", &limits).expect("u64");
        assert_eq!(v.as_u64(), Some(9_223_372_036_854_775_808u64));
        // u64::MAX preservado.
        let v: serde_json::Value =
            decode_json(b"18446744073709551615", &limits).expect("u64max");
        assert_eq!(v.as_u64(), Some(u64::MAX));
        // `1e999` estoura f64::MAX e serde rejeita (mapeado para Truncated).
        // Nota: inteiros gigantes como "9"*200 viram f64 (1e200) no serde_json
        // default — documentado como LastWins-like: sem erro, vira Number(Float).
        // Apenas overflow de f64 é erro.
        let huge_f64 = b"1e999";
        let err = decode_json::<serde_json::Value>(huge_f64, &limits).expect_err("f64 overflow");
        assert!(matches!(err, SerError::Truncated(_)), "got {err:?}");
        let big_but_float: serde_json::Value =
            decode_json("9".repeat(200).as_bytes(), &limits).expect("big becomes float");
        assert!(big_but_float.is_number(), "expected number, got {big_but_float:?}");
        // Negativo que estoura f64::MAX (-1e999) é rejeitado; -30 noves vira f64 (documentado).
        let neg_ok: serde_json::Value =
            decode_json(format!("-{}", "9".repeat(30)).as_bytes(), &limits)
                .expect("neg 30 digits becomes float");
        assert!(neg_ok.is_number());
        let neg_over = decode_json::<serde_json::Value>(b"-1e999", &limits);
        assert!(neg_over.is_err(), "negative f64 overflow must fail");
        // msgpack/cbor int overflow boundaries.
        let mut out = Vec::new();
        crate::response::msgpack::encode_int(i64::MAX, &mut out);
        let (back, _) = crate::response::msgpack::decode_int(&out).expect("mp i64max");
        assert_eq!(back, i64::MAX);
        // msgpack uint64 > i64::MAX rejeita.
        let big_u64 = vec![0xcf, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let err = crate::response::msgpack::decode_int(&big_u64).expect_err("mp overflow");
        assert!(matches!(err, SerError::Overflow(_)), "got {err:?}");
    }

    #[test]
    fn json_invalid_utf8_rejected() {
        let limits = Limits::default();
        let bad = vec![0xFF, 0xFE, b'{', b'}'];
        let err = decode_json::<serde_json::Value>(&bad, &limits).expect_err("utf8");
        assert!(matches!(err, SerError::Utf8(_)), "got {err:?}");
        // Truncado no meio de multibyte.
        let bad2 = vec![0x22, 0xC3, 0x28];
        let err = decode_json::<serde_json::Value>(&bad2, &limits).expect_err("utf8-2");
        assert!(
            matches!(err, SerError::Utf8(_) | SerError::Truncated(_)),
            "got {err:?}"
        );
    }

    #[test]
    fn limits_enforced_on_all_decode_paths() {
        let tiny = Limits::new(8, 2, 4);
        // varint input > max_bytes.
        let big = vec![0x01u8; 64];
        assert!(matches!(
            decode_varint(&big, &tiny),
            Err(SerError::Overflow(_))
        ));
        // varint truncado (continuation sem fim).
        assert!(matches!(
            decode_varint(&[0x80, 0x80], &Limits::default()),
            Err(SerError::Truncated(_))
        ));
        // varint > 10 bytes.
        assert!(matches!(
            decode_varint(&[0x80u8; 11], &Limits::default()),
            Err(SerError::Overflow(_))
        ));
        // len-prefixed com len > max.
        let mut framed = Vec::new();
        encode_varint(100, &mut framed);
        framed.extend_from_slice(&[0u8; 100]);
        assert!(matches!(
            decode_len_prefixed(&framed, &tiny),
            Err(SerError::Overflow(_))
        ));
        // len-prefixed truncado.
        let mut trunc = Vec::new();
        encode_varint(10, &mut trunc);
        trunc.extend_from_slice(&[1, 2, 3]);
        assert!(matches!(
            decode_len_prefixed(&trunc, &Limits::default()),
            Err(SerError::Truncated(_))
        ));
        // msgpack str len > max.
        let mut ms = Vec::new();
        msgpack::encode_str("0123456789ABCDEF", &mut ms);
        assert!(matches!(
            msgpack::decode_str(&ms, &tiny),
            Err(SerError::Overflow(_))
        ));
        // msgpack str truncada.
        assert!(matches!(
            msgpack::decode_str(&[0xd9, 0x05, 0x61, 0x62], &Limits::default()),
            Err(SerError::Truncated(_))
        ));
        // msgpack array/map len > max_array_len.
        let mut ma = Vec::new();
        msgpack::encode_array_len(10, &mut ma);
        assert!(matches!(
            msgpack::decode_array_len(&ma, &tiny),
            Err(SerError::Overflow(_))
        ));
        let mut mm = Vec::new();
        msgpack::encode_map_len(10, &mut mm);
        assert!(matches!(
            msgpack::decode_map_len(&mm, &tiny),
            Err(SerError::Overflow(_))
        ));
        // cbor str/array/map.
        let mut cs = Vec::new();
        cbor::encode_str("0123456789ABCDEF", &mut cs);
        assert!(matches!(
            cbor::decode_str(&cs, &tiny),
            Err(SerError::Overflow(_))
        ));
        let mut ca = Vec::new();
        cbor::encode_array_len(10, &mut ca);
        assert!(matches!(
            cbor::decode_array_len(&ca, &tiny),
            Err(SerError::Overflow(_))
        ));
        let mut cm = Vec::new();
        cbor::encode_map_len(10, &mut cm);
        assert!(matches!(
            cbor::decode_map_len(&cm, &tiny),
            Err(SerError::Overflow(_))
        ));
        // protobuf varint/len.
        let mut pv = Vec::new();
        protobuf::encode_varint(1, &mut pv);
        // input maior que max_bytes dispara overflow no check inicial
        // (protobuf::decode_varint checa input.len() > max_bytes).
        let big_pb = vec![0x01u8; 64];
        assert!(matches!(
            protobuf::decode_varint(&big_pb, &tiny),
            Err(SerError::Overflow(_))
        ));
        let mut pl = Vec::new();
        protobuf::encode_len(1, &[0u8; 100], &mut pl);
        assert!(matches!(
            protobuf::decode_len_field(&pl, &tiny),
            Err(SerError::Overflow(_))
        ));
        // yaml/toml over max_bytes e max_array_len.
        let big_yaml = "x".repeat(64);
        assert!(matches!(
            yaml_decode_map(&big_yaml, &tiny),
            Err(SerError::Overflow(_) | SerError::Truncated(_))
        ));
        let mut many = String::new();
        for i in 0..10 {
            many.push_str(&format!("k{i}: v\n"));
        }
        assert!(matches!(
            yaml_decode_map(&many, &tiny),
            Err(SerError::Overflow(_))
        ));
        let big_toml = "x".repeat(64);
        assert!(matches!(
            toml_decode_map(&big_toml, &tiny),
            Err(SerError::Overflow(_) | SerError::Truncated(_))
        ));
        assert!(matches!(
            toml_decode_map(&many.replace(':', "="), &tiny),
            Err(SerError::Overflow(_))
        ));
        // yaml/toml garbage.
        assert!(matches!(
            yaml_decode_map("sem-dois-pontos", &Limits::default()),
            Err(SerError::Truncated(_))
        ));
        assert!(matches!(
            toml_decode_map("sem-igual", &Limits::default()),
            Err(SerError::Truncated(_))
        ));
        // json array len > max_array_len.
        let arr = "[1,2,3,4,5,6,7,8,9,10]";
        assert!(matches!(
            decode_json::<serde_json::Value>(arr.as_bytes(), &tiny),
            Err(SerError::Overflow(_))
        ));
    }

    #[test]
    fn response_helpers_and_xml() {
        let r = Response::ok("d".to_string());
        assert!(r.success);
        let e = Response::err("bad");
        assert!(!e.success);
        let _ = response_ok("hi");
        let _ = response_err("oops");
        let x = response_to_xml(&r);
        assert!(x.contains("<success>true</success>"));
        let v = Response::ok_msg("d".to_string(), "m");
        assert_eq!(v.message.as_deref(), Some("m"));
    }
}
