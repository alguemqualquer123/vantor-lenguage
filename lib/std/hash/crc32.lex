// Lexicon Standard Library — hash/crc32.
// Go-parity CRC-32 IEEE (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 6).
// The one-shot sum is native; the streaming value mirrors `hash.Hash32`
// with value semantics. Import as `import std::hash::crc32;`.

/// Streaming CRC-32 state (Go's `crc32.digest`).
pub struct CrcHash {
    crc: i64,
}

/// IEEE CRC-32 of `s` (Go's `crc32.ChecksumIEEE`).
pub fn Checksum(s: String) -> i64 {
    return Hash::crc32(s);
}

/// Starts a stream (Go's `crc32.NewIEEE()`).
pub fn New() -> CrcHash {
    return CrcHash { crc: 0 };
}

/// Feeds `s` into the stream (Go's `h.Write`). Bit-by-bit in Lex —
// prefer the native `Checksum` one-shot for bulk data.
pub fn Write(h: CrcHash, s: String) -> CrcHash {
    let crc = h.crc ^ 4294967295;
    let i = 0;
    while i < s.len() {
        crc = crc ^ Text::code_at(s, i);
        let k = 0;
        while k < 8 {
            if crc % 2 == 1 {
                crc = (crc / 2) ^ 3988292384;
            } else {
                crc = crc / 2;
            }
            k = k + 1;
        }
        i = i + 1;
    }
    return CrcHash { crc: crc ^ 4294967295 };
}

/// Finalizes the stream (Go's `h.Sum32()`).
pub fn Sum32(h: CrcHash) -> i64 {
    return h.crc & 4294967295;
}
