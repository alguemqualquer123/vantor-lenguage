// Lexicon Standard Library — cmp.
// Go-parity three-way comparisons (plan: Docs/PLANO_GO_FULL_PARITY.md,
// Fase 2). Generics in the AST stay monomorphic in the interpreter, so
// each kind gets a typed entry point — same ordering Go documents.

/// Compares two i64 values: -1, 0 or +1 (Go's `cmp.Compare[int]`).
pub fn CompareInt(a: i64, b: i64) -> i64 {
    if a < b {
        return -1;
    }
    if a > b {
        return 1;
    }
    return 0;
}

/// Compares two f64 values: -1, 0 or +1.
pub fn CompareFloat(a: f64, b: f64) -> i64 {
    if a < b {
        return -1;
    }
    if a > b {
        return 1;
    }
    return 0;
}

/// Compares two strings lexicographically: -1, 0 or +1
/// (Go's `cmp.Compare[string]`).
pub fn CompareString(a: String, b: String) -> i64 {
    if a < b {
        return -1;
    }
    if a > b {
        return 1;
    }
    return 0;
}

/// Reports whether `a < b` for numbers-or-strings held as Dynamic
/// (Go's `cmp.Less`).
pub fn Less(a: Dynamic, b: Dynamic) -> bool {
    return a < b;
}

/// Reports whether `a > b` (Go's `cmp.Greater`).
pub fn Greater(a: Dynamic, b: Dynamic) -> bool {
    return a > b;
}

/// Returns the first non-zero comparison result, or zero (Go's `cmp.Or`).
pub fn Or(vals: Dynamic) -> i64 {
    let i = 0;
    while i < vals.len() {
        if vals[i] != 0 {
            return vals[i];
        }
        i = i + 1;
    }
    return 0;
}

/// Returns the smaller of two Dynamic numbers-or-strings.
pub fn Min(a: Dynamic, b: Dynamic) -> Dynamic {
    if a <= b {
        return a;
    }
    return b;
}

/// Returns the larger of two Dynamic numbers-or-strings.
pub fn Max(a: Dynamic, b: Dynamic) -> Dynamic {
    if a >= b {
        return a;
    }
    return b;
}
