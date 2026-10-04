// Lexicon Standard Library — maps.
// Go-parity map helpers (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 2).
// Maps are insertion-ordered lists of `[key, value]` pairs (there is no
// addressable hash table in the interpreter yet): linear scan, O(n) per
// op, zero hidden allocations. Like `sort`, mutating ops return the new
// map — write `m = maps::Set(m, k, v)`. Keys compare with `==`
// (homogeneous string or numeric keys, like Go's comparable constraint).

/// Result of a lookup: `value` plus `ok` (Go's `v, ok := m[k]` idiom).
pub struct GetResult {
    value: Dynamic,
    ok: bool,
}

/// Returns an empty map.
pub fn New() -> Dynamic {
    return [];
}

/// Returns the number of entries.
pub fn Len(m: Dynamic) -> i64 {
    return m.len();
}

/// Reports whether `k` is present.
pub fn Has(m: Dynamic, k: Dynamic) -> bool {
    let i = 0;
    while i < m.len() {
        if m[i][0] == k {
            return true;
        }
        i = i + 1;
    }
    return false;
}

/// Looks up `k` (Go's `m[k]` + comma-ok).
pub fn Get(m: Dynamic, k: Dynamic) -> GetResult {
    let i = 0;
    while i < m.len() {
        if m[i][0] == k {
            return GetResult { value: m[i][1], ok: true };
        }
        i = i + 1;
    }
    return GetResult { value: 0, ok: false };
}

/// Returns a new map with `k` bound to `v` (insert or replace).
pub fn Set(m: Dynamic, k: Dynamic, v: Dynamic) -> Dynamic {
    let out = [];
    let done = false;
    let i = 0;
    while i < m.len() {
        if m[i][0] == k {
            out.push([k, v]);
            done = true;
        } else {
            out.push(m[i]);
        }
        i = i + 1;
    }
    if !done {
        out.push([k, v]);
    }
    return out;
}

/// Returns a new map without `k` (Go's `delete(m, k)`; missing keys are
/// a no-op, like Go).
pub fn Delete(m: Dynamic, k: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < m.len() {
        if m[i][0] != k {
            out.push(m[i]);
        }
        i = i + 1;
    }
    return out;
}

/// Returns the keys in insertion order (Go's `maps.Keys` as a list).
pub fn Keys(m: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < m.len() {
        out.push(m[i][0]);
        i = i + 1;
    }
    return out;
}

/// Returns the values in insertion order (Go's `maps.Values` as a list).
pub fn Values(m: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < m.len() {
        out.push(m[i][1]);
        i = i + 1;
    }
    return out;
}

/// Returns a copy of `m` (Go's `maps.Clone`).
pub fn Clone(m: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < m.len() {
        out.push(m[i]);
        i = i + 1;
    }
    return out;
}

/// Reports whether `a` and `b` hold the same bindings regardless of order
/// (Go's `maps.Equal`).
pub fn Equal(a: Dynamic, b: Dynamic) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let i = 0;
    while i < a.len() {
        let g = Get(b, a[i][0]);
        if !g.ok || g.value != a[i][1] {
            return false;
        }
        i = i + 1;
    }
    return true;
}

/// Merges `b` into `a` (entries of `b` win) and returns the result
/// (Go's `maps.Copy` destination-adapted: `m = maps::Merge(a, b)`).
pub fn Merge(a: Dynamic, b: Dynamic) -> Dynamic {
    let out = Clone(a);
    let i = 0;
    while i < b.len() {
        out = Set(out, b[i][0], b[i][1]);
        i = i + 1;
    }
    return out;
}
