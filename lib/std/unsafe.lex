// Lexicon Standard Library — unsafe.
// Go-parity unsafe surface, memory-safe subset
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 8). The interpreter never
// exposes raw addresses: `Sizeof` follows the 64-bit word model
// (8 bytes per word; strings measured in bytes, composites per element),
// dispatching on the `inspect` shape since values are dynamically typed.
// Arbitrary pointer arithmetic is intentionally absent.
// Import as `import std::unsafe;` (the parser accepts `unsafe::`).

/// Size in bytes of a value (Go's `unsafe.Sizeof`, word model).
pub fn Sizeof(v: Dynamic) -> i64 {
    let d = inspect(v);
    let c = Text::slice(d, 0, 1);
    if c == "\"" {
        return d.len() - 2;
    }
    if c == "[" || c == "(" {
        return 8 + v.len() * 8;
    }
    if d == "null" {
        return 0;
    }
    if Text::index_of(d, " { ") >= 0 {
        return 8 + v.len() * 8;
    }
    return 8;
}

/// Alignment in bytes (Go's `unsafe.Alignof`): 8 on this 64-bit model.
pub fn Alignof(v: Dynamic) -> i64 {
    return 8;
}

/// Reports whether `v` is null (Go-inspired nil test for Dynamic).
pub fn IsNil(v: Dynamic) -> bool {
    return inspect(v) == "null";
}
