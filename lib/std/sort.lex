// Lexicon Standard Library — sort.
// Go-parity sorting for homogeneous lists
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1).
//
// Two deliberate adaptations to Lex value semantics (lists pass by
// copy across function calls — there is no shared backing array):
//   * Ints/Floats/Strings RETURN the sorted list instead of mutating
//     in place: write `xs = sort::Ints(xs)`. Merge sort replaces
//     quicksort because recursion must compose by return value.
//   * `sort.Slice` / `SliceFunc` need function values; lambda values
//     land in Fase 8, so the typed variants are the generic entry
//     points (matching the classic Go sort package API).

// ---------------------------------------------------------------------------
// Native fast path (Rust `sort_unstable` — no per-level allocations like
// the Lex merge sort below, which stays as the documented fallback).
// ---------------------------------------------------------------------------

/// Returns a new list with the i64 elements of `xs` sorted ascending
/// (native; write `xs = sort::Ints(xs)` — lists pass by value).
pub fn Ints(xs: Dynamic) -> Dynamic {
    return List::sort_ints(xs);
}

/// Returns a new list with the f64 elements of `xs` sorted ascending
/// (native).
pub fn Floats(xs: Dynamic) -> Dynamic {
    return List::sort_floats(xs);
}

/// Returns a new list with the String elements of `xs` sorted ascending
/// (native).
pub fn Strings(xs: Dynamic) -> Dynamic {
    return List::sort_strings(xs);
}

/// Reports whether `xs` (list of i64) is sorted ascending.
pub fn IntsAreSorted(xs: Dynamic) -> bool {
    let i = 1;
    let n = xs.len();
    while i < n {
        if xs[i] < xs[i - 1] {
            return false;
        }
        i = i + 1;
    }
    return true;
}

/// Reports whether `xs` (list of f64) is sorted ascending.
pub fn FloatsAreSorted(xs: Dynamic) -> bool {
    let i = 1;
    let n = xs.len();
    while i < n {
        if xs[i] < xs[i - 1] {
            return false;
        }
        i = i + 1;
    }
    return true;
}

/// Reports whether `xs` (list of String) is sorted ascending.
pub fn StringsAreSorted(xs: Dynamic) -> bool {
    let i = 1;
    let n = xs.len();
    while i < n {
        if xs[i] < xs[i - 1] {
            return false;
        }
        i = i + 1;
    }
    return true;
}

/// Binary search over a sorted list of i64: returns the smallest index
/// with `xs[i] >= x`, or `n` if there is none (Go's `sort.SearchInts`).
pub fn SearchInts(xs: Dynamic, x: i64) -> i64 {
    let lo = 0;
    let hi = xs.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if xs[mid] < x {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    return lo;
}

/// Binary search over a sorted list of f64 (Go's `sort.SearchFloat64s`).
pub fn SearchFloats(xs: Dynamic, x: f64) -> i64 {
    let lo = 0;
    let hi = xs.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if xs[mid] < x {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    return lo;
}

/// Binary search over a sorted list of String (Go's `sort.SearchStrings`).
pub fn SearchStrings(xs: Dynamic, x: String) -> i64 {
    let lo = 0;
    let hi = xs.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if xs[mid] < x {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    return lo;
}

// ---------------------------------------------------------------------------
// internal: merge sort (returns new lists; Lex passes lists by value)
// ---------------------------------------------------------------------------

fn sort_msInt(xs: Dynamic, lo: i64, hi: i64) -> Dynamic {
    if lo >= hi {
        let single = [];
        single.push(xs[lo]);
        return single;
    }
    let mid = (lo + hi) / 2;
    let left = sort_msInt(xs, lo, mid);
    let right = sort_msInt(xs, mid + 1, hi);
    return sort_mergeInt(left, right);
}

fn sort_mergeInt(a: Dynamic, b: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    let j = 0;
    let n = a.len();
    let m = b.len();
    while i < n && j < m {
        if a[i] <= b[j] {
            out.push(a[i]);
            i = i + 1;
        } else {
            out.push(b[j]);
            j = j + 1;
        }
    }
    while i < n {
        out.push(a[i]);
        i = i + 1;
    }
    while j < m {
        out.push(b[j]);
        j = j + 1;
    }
    return out;
}

fn sort_msFloat(xs: Dynamic, lo: i64, hi: i64) -> Dynamic {
    if lo >= hi {
        let single = [];
        single.push(xs[lo]);
        return single;
    }
    let mid = (lo + hi) / 2;
    let left = sort_msFloat(xs, lo, mid);
    let right = sort_msFloat(xs, mid + 1, hi);
    return sort_mergeFloat(left, right);
}

fn sort_mergeFloat(a: Dynamic, b: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    let j = 0;
    let n = a.len();
    let m = b.len();
    while i < n && j < m {
        if a[i] <= b[j] {
            out.push(a[i]);
            i = i + 1;
        } else {
            out.push(b[j]);
            j = j + 1;
        }
    }
    while i < n {
        out.push(a[i]);
        i = i + 1;
    }
    while j < m {
        out.push(b[j]);
        j = j + 1;
    }
    return out;
}

fn sort_msStr(xs: Dynamic, lo: i64, hi: i64) -> Dynamic {
    if lo >= hi {
        let single = [];
        single.push(xs[lo]);
        return single;
    }
    let mid = (lo + hi) / 2;
    let left = sort_msStr(xs, lo, mid);
    let right = sort_msStr(xs, mid + 1, hi);
    return sort_mergeStr(left, right);
}

fn sort_mergeStr(a: Dynamic, b: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    let j = 0;
    let n = a.len();
    let m = b.len();
    while i < n && j < m {
        if a[i] <= b[j] {
            out.push(a[i]);
            i = i + 1;
        } else {
            out.push(b[j]);
            j = j + 1;
        }
    }
    while i < n {
        out.push(a[i]);
        i = i + 1;
    }
    while j < m {
        out.push(b[j]);
        j = j + 1;
    }
    return out;
}
