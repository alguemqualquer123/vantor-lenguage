// Lexicon Standard Library — hash/crc64.
// Go-parity CRC-64 ECMA (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 6).
// One-shot is native; streaming mirrors `hash.Hash64` with value
// semantics. The ECMA polynomial exceeds i64::MAX, so it travels wrapped
// (two's-complement view, like Go on overflow) with an exact logical
// shift. Import as `import std::hash::crc64;`.

/// Streaming CRC-64 state.
pub struct Hash64State {
    crc: i64,
}

/// ECMA CRC-64 of `s` (Go's `crc64.Checksum(data, ECMA-table)`).
pub fn Checksum(s: String) -> i64 {
    return Hash::crc64(s);
}

/// Starts a stream (Go's `crc64.New(table)`).
pub fn New() -> Hash64State {
    return Hash64State { crc: 0 };
}

/// Feeds `s` into the stream (Go's `h.Write`). Bit-by-bit in Lex —
/// prefer the native `Checksum` one-shot for bulk data.
pub fn Write(h: Hash64State, s: String) -> Hash64State {
    let crc = h.crc ^ -1;
    let i = 0;
    while i < s.len() {
        crc = crc ^ Text::code_at(s, i);
        let k = 0;
        while k < 8 {
            if crc & 1 != 0 {
                crc = shr1(crc) ^ -3932672073523589310;
            } else {
                crc = shr1(crc);
            }
            k = k + 1;
        }
        i = i + 1;
    }
    return Hash64State { crc: crc ^ -1 };
}

/// Finalizes the stream (Go's `h.Sum64()`, bits-wrapped).
pub fn Sum64(h: Hash64State) -> i64 {
    return h.crc;
}

// Logical shift right by 1 (exact for negatives; `/` truncates).
fn shr1(c: i64) -> i64 {
    if c >= 0 {
        return c / 2;
    }
    let t = c / 2;
    if c % 2 == 0 {
        return t + 9223372036854775807 + 1;
    }
    return t - 1 + 9223372036854775807 + 1;
}
