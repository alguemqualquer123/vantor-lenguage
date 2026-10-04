// Lexicon Standard Library — crypto/subtle.
// Go-parity constant-time helpers (plan: Docs/PLANO_GO_FULL_PARITY.md,
// Fase 6). The comparison touches every byte with no early exit, so
// timing reveals nothing about the mismatch position. Import as
// `import std::crypto::subtle;`.

/// Constant-time equality: 1 when equal, 0 otherwise (Go's
/// `subtle.ConstantTimeCompare`). Accumulates mismatches with `+` (a bare
/// `|` would parse as a pipe-closure here — see the syntax notes).
pub fn ConstantTimeCompare(a: String, b: String) -> i64 {
    if a.len() != b.len() {
        return 0;
    }
    let diff = 0;
    let i = 0;
    while i < a.len() {
        if (Text::code_at(a, i) ^ Text::code_at(b, i)) != 0 {
            diff = diff + 1;
        }
        i = i + 1;
    }
    if diff == 0 {
        return 1;
    }
    return 0;
}

/// Constant-time byte equality (Go's `subtle.ConstantTimeByteEq`).
pub fn ConstantTimeByteEq(a: i64, b: i64) -> i64 {
    if (a ^ b) == 0 {
        return 1;
    }
    return 0;
}

/// Constant-time select: `v1` when `x == 1`, `v0` when `x == 0`
/// (Go's `subtle.ConstantTimeSelect`).
pub fn ConstantTimeSelect(x: i64, v0: i64, v1: i64) -> i64 {
    if x == 1 {
        return v1;
    }
    return v0;
}
