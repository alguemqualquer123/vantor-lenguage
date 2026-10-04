// Lexicon Standard Library — encoding/binary.
// Go-parity fixed-size integer codecs (plan: Docs/PLANO_GO_FULL_PARITY.md,
// Fase 4). Byte buffers are lists of 0-255 integers, so `binary.Write` /
// `binary.Read` become pure functions: `Put…` returns the bytes, `Be…` /
// `Le…` read them back. Import as `import std::encoding::binary;`.

/// 2 bytes, big-endian (Go's `binary.BigEndian.PutUint16`).
pub fn PutBE16(v: i64) -> Dynamic {
    return [v / 256 % 256, v % 256];
}

/// 4 bytes, big-endian (Go's `binary.BigEndian.PutUint32`).
pub fn PutBE32(v: i64) -> Dynamic {
    return [v / 16777216 % 256, v / 65536 % 256, v / 256 % 256, v % 256];
}

/// 8 bytes, big-endian (Go's `binary.BigEndian.PutUint64`).
pub fn PutBE64(v: i64) -> Dynamic {
    return [v / 72057594037927936 % 256, v / 281474976710656 % 256,
            v / 1099511627776 % 256, v / 4294967296 % 256,
            v / 16777216 % 256, v / 65536 % 256, v / 256 % 256, v % 256];
}

/// 2 bytes, little-endian (Go's `binary.LittleEndian.PutUint16`).
pub fn PutLE16(v: i64) -> Dynamic {
    return [v % 256, v / 256 % 256];
}

/// 4 bytes, little-endian (Go's `binary.LittleEndian.PutUint32`).
pub fn PutLE32(v: i64) -> Dynamic {
    return [v % 256, v / 256 % 256, v / 65536 % 256, v / 16777216 % 256];
}

/// 8 bytes, little-endian (Go's `binary.LittleEndian.PutUint64`).
pub fn PutLE64(v: i64) -> Dynamic {
    return [v % 256, v / 256 % 256, v / 65536 % 256, v / 16777216 % 256,
            v / 4294967296 % 256, v / 4294967296 / 256 % 256,
            v / 4294967296 / 65536 % 256, v / 4294967296 / 16777216 % 256];
}

/// Big-endian uint16 from `b[0..2)` (Go's `BigEndian.Uint16`).
pub fn Be16(b: Dynamic) -> i64 {
    return b[0] * 256 + b[1];
}

/// Big-endian uint32 from `b[0..4)` (Go's `BigEndian.Uint32`).
pub fn Be32(b: Dynamic) -> i64 {
    return b[0] * 16777216 + b[1] * 65536 + b[2] * 256 + b[3];
}

/// Big-endian uint64 from `b[0..8)` (Go's `BigEndian.Uint64`).
pub fn Be64(b: Dynamic) -> i64 {
    let acc = 0;
    let i = 0;
    while i < 8 {
        acc = acc * 256 + b[i];
        i = i + 1;
    }
    return acc;
}

/// Little-endian uint16 from `b[0..2)` (Go's `LittleEndian.Uint16`).
pub fn Le16(b: Dynamic) -> i64 {
    return b[1] * 256 + b[0];
}

/// Little-endian uint32 from `b[0..4)` (Go's `LittleEndian.Uint32`).
pub fn Le32(b: Dynamic) -> i64 {
    return b[3] * 16777216 + b[2] * 65536 + b[1] * 256 + b[0];
}

/// Little-endian uint64 from `b[0..8)` (Go's `LittleEndian.Uint64`).
pub fn Le64(b: Dynamic) -> i64 {
    let acc = 0;
    let i = 7;
    while i >= 0 {
        acc = acc * 256 + b[i];
        i = i - 1;
    }
    return acc;
}

/// Big-endian bytes of every element of `vs` (Go's `binary.Write` for a
/// `[]uint32`).
pub fn WriteBE32All(vs: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < vs.len() {
        let bs = PutBE32(vs[i]);
        let j = 0;
        while j < 4 {
            out.push(bs[j]);
            j = j + 1;
        }
        i = i + 1;
    }
    return out;
}
