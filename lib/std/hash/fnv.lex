// Lexicon Standard Library — hash/fnv.
// Go-parity FNV-1/FNV-1a (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 6).
// One-shot sums delegate to the native `Hash::*` builtins (single pass,
// no interpreter loop); the streaming `Hash` value keeps Go's `Write`/
// `Sum` shapes with value semantics — reassign: `h = fnv::Write(h, s)`.
// Import as `import std::hash::fnv;`.

/// Streaming 32-bit hash state (Go's `hash.Hash32`).
pub struct Hash32 {
    h: i64,
}

/// Streaming 64-bit hash state (Go's `hash.Hash64`; the i64 wraps the
/// uint64 bits, like Go's two's-complement view on overflow).
pub struct Hash64 {
    h: i64,
}

/// 32-bit FNV-1a of `s` (Go's `fnv.New32a().Sum` one-shot).
pub fn Sum32a(s: String) -> i64 {
    return Hash::fnv1a32(s);
}

/// 64-bit FNV-1a of `s` (Go's `fnv.New64a().Sum`, bits-wrapped).
pub fn Sum64a(s: String) -> i64 {
    return Hash::fnv1a64(s);
}

/// Starts a 32-bit FNV-1a stream (Go's `fnv.New32a()`).
pub fn New32a() -> Hash32 {
    return Hash32 { h: 2166136261 };
}

/// Starts a 64-bit FNV-1a stream.
pub fn New64a() -> Hash64 {
    return Hash64 { h: -3750763034362895579 };
}

/// Feeds `s` into a 32-bit stream (Go's `h.Write`; masked to 32 bits
/// every round so long streams match the native one-shot `Sum32a`).
pub fn Write32(h: Hash32, s: String) -> Hash32 {
    let acc = h.h;
    let i = 0;
    while i < s.len() {
        acc = ((acc ^ Text::code_at(s, i)) * 16777619) & 4294967295;
        i = i + 1;
    }
    return Hash32 { h: acc };
}

/// Feeds `s` into a 64-bit stream.
pub fn Write64(h: Hash64, s: String) -> Hash64 {
    let acc = h.h;
    let i = 0;
    while i < s.len() {
        acc = (acc ^ Text::code_at(s, i)) * 1099511628211;
        i = i + 1;
    }
    return Hash64 { h: acc };
}

/// Finalizes a 32-bit stream (Go's `h.Sum32()`).
pub fn Sum32(h: Hash32) -> i64 {
    return h.h;
}

/// Finalizes a 64-bit stream (Go's `h.Sum64()`, bits-wrapped).
pub fn Sum64(h: Hash64) -> i64 {
    return h.h;
}
