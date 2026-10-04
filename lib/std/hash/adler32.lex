// Lexicon Standard Library — hash/adler32.
// Go-parity Adler-32 (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 6).
// Import as `import std::hash::adler32;`.

/// Streaming Adler-32 state (Go's `adler32.digest`).
pub struct AdlerHash {
    a: i64,
    b: i64,
}

/// Adler-32 of `s` (Go's `adler32.Checksum`).
pub fn Checksum(s: String) -> i64 {
    return Hash::adler32(s);
}

/// Starts a stream (Go's `adler32.New()`).
pub fn New() -> AdlerHash {
    return AdlerHash { a: 1, b: 0 };
}

/// Feeds `s` into the stream (Go's `h.Write`).
pub fn Write(h: AdlerHash, s: String) -> AdlerHash {
    let a = h.a;
    let b = h.b;
    let i = 0;
    while i < s.len() {
        a = (a + Text::code_at(s, i)) % 65521;
        b = (b + a) % 65521;
        i = i + 1;
    }
    return AdlerHash { a: a, b: b };
}

/// Finalizes the stream (Go's `h.Sum32()`).
pub fn Sum32(h: AdlerHash) -> i64 {
    return h.b * 65536 + h.a;
}
