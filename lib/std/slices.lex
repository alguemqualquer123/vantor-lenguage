// Lexicon Standard Library — slices.
// Go 1.21+ `slices` package parity for Dynamic lists
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1).
//
// Go's func-taking helpers (SortFunc, Filter, Map...) await lambda
// values (plan, Fase 8). Functions that mutate in Go return a new list
// here where Lex's value semantics require it; noted per function.

/// Returns the first index of `v` in `xs`, or -1 (Go's `slices.Index`).
pub fn Index(xs: Dynamic, v: Dynamic) -> i64 {
    let i = 0;
    let n = xs.len();
    while i < n {
        if xs[i] == v {
            return i;
        }
        i = i + 1;
    }
    return -1;
}

/// Reports whether `v` is present in `xs` (Go's `slices.Contains`).
/// Calls `slices::Index` qualified: `strings::Index` shares the flattened
/// name and would win by load order otherwise.
pub fn Contains(xs: Dynamic, v: Dynamic) -> bool {
    return slices::Index(xs, v) >= 0;
}

/// Returns a shallow copy of `xs` (Go's `slices.Clone`).
pub fn Clone(xs: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    let n = xs.len();
    while i < n {
        out.push(xs[i]);
        i = i + 1;
    }
    return out;
}

/// Returns a new list with the elements of `xs` in reverse order
/// (Go's `slices.Reverse` mutates in place; Lex returns a new list —
/// write `xs = slices::Reverse(xs)`).
pub fn Reverse(xs: Dynamic) -> Dynamic {
    let out = [];
    let i = xs.len() - 1;
    while i >= 0 {
        out.push(xs[i]);
        i = i - 1;
    }
    return out;
}

/// Returns the smallest element of a non-empty list (Go's `slices.Min`;
/// homogeneous numeric or string elements).
pub fn Min(xs: Dynamic) -> Dynamic {
    let m = xs[0];
    let i = 1;
    let n = xs.len();
    while i < n {
        if xs[i] < m {
            m = xs[i];
        }
        i = i + 1;
    }
    return m;
}

/// Returns the largest element of a non-empty list (Go's `slices.Max`).
pub fn Max(xs: Dynamic) -> Dynamic {
    let m = xs[0];
    let i = 1;
    let n = xs.len();
    while i < n {
        if xs[i] > m {
            m = xs[i];
        }
        i = i + 1;
    }
    return m;
}

/// Binary search: returns the smallest index with `xs[i] >= v`, or `n`
/// if there is none (Go's `slices.BinarySearch`).
pub fn BinarySearch(xs: Dynamic, v: Dynamic) -> i64 {
    let lo = 0;
    let hi = xs.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if xs[mid] < v {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    return lo;
}

/// Returns a new list with `v` inserted at index `i`
/// (Go's `slices.Insert`; Go mutates the backing array, Lex returns new).
pub fn Insert(xs: Dynamic, i: i64, v: Dynamic) -> Dynamic {
    let out = [];
    let n = xs.len();
    let k = 0;
    while k < n {
        if k == i {
            out.push(v);
        }
        out.push(xs[k]);
        k = k + 1;
    }
    if i >= n {
        out.push(v);
    }
    return out;
}

/// Returns a new list with elements `xs[i:j]` removed
/// (Go's `slices.Delete`).
pub fn Delete(xs: Dynamic, i: i64, j: i64) -> Dynamic {
    let out = [];
    let n = xs.len();
    let k = 0;
    while k < n {
        if k < i || k >= j {
            out.push(xs[k]);
        }
        k = k + 1;
    }
    return out;
}

/// Returns a new list with `xs[i:j]` replaced by `v` (Go's `slices.Replace`).
pub fn Replace(xs: Dynamic, i: i64, j: i64, v: Dynamic) -> Dynamic {
    let out = [];
    let n = xs.len();
    let k = 0;
    while k < n {
        if k == i {
            out.push(v);
        }
        if k < i || k >= j {
            out.push(xs[k]);
        }
        k = k + 1;
    }
    return out;
}

/// Reports whether `a` and `b` have equal elements (Go's `slices.Equal`).
pub fn Equal(a: Dynamic, b: Dynamic) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let i = 0;
    let n = a.len();
    while i < n {
        if a[i] != b[i] {
            return false;
        }
        i = i + 1;
    }
    return true;
}

/// Lexicographic three-way comparison of two lists (Go's `slices.Compare`):
/// -1, 0, or +1. Homogeneous numeric or string elements.
pub fn Compare(a: Dynamic, b: Dynamic) -> i64 {
    let n = a.len();
    let m = b.len();
    let k = m;
    if n < m {
        k = n;
    }
    let i = 0;
    while i < k {
        if a[i] < b[i] {
            return -1;
        }
        if a[i] > b[i] {
            return 1;
        }
        i = i + 1;
    }
    if n < m {
        return -1;
    }
    if n > m {
        return 1;
    }
    return 0;
}

/// Returns a new list with consecutive duplicate elements removed
/// (Go's `slices.Compact`).
pub fn Compact(xs: Dynamic) -> Dynamic {
    let out = [];
    let n = xs.len();
    if n == 0 {
        return out;
    }
    out.push(xs[0]);
    let i = 1;
    while i < n {
        if xs[i] != xs[i - 1] {
            out.push(xs[i]);
        }
        i = i + 1;
    }
    return out;
}

/// Concatenates any number of lists (Go's `slices.Concat`; pass a list
/// of lists: `slices::Concat([[1], [2, 3]])`).
pub fn Concat(xss: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < xss.len() {
        let part = xss[i];
        let j = 0;
        while j < part.len() {
            out.push(part[j]);
            j = j + 1;
        }
        i = i + 1;
    }
    return out;
}

/// Returns a new list with `f` applied to every element. Lambdas are
/// first-class: `slices::Map(xs, |x| x * 2)`.
pub fn Map(xs: Dynamic, f: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < xs.len() {
        out.push(f(xs[i]));
        i = i + 1;
    }
    return out;
}

/// Returns the elements where `f(x)` is true:
/// `slices::Filter(xs, |x| x > 0)`.
pub fn Filter(xs: Dynamic, f: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < xs.len() {
        if f(xs[i]) {
            out.push(xs[i]);
        }
        i = i + 1;
    }
    return out;
}

/// Folds from the left: `acc = f(acc, x)`.
pub fn Reduce(xs: Dynamic, init: Dynamic, f: Dynamic) -> Dynamic {
    let acc = init;
    let i = 0;
    while i < xs.len() {
        acc = f(acc, xs[i]);
        i = i + 1;
    }
    return acc;
}

/// Reports whether `f(x)` is true for every element (Go's `slices.All`
/// spelling as a boolean fold).
pub fn All(xs: Dynamic, f: Dynamic) -> bool {
    let i = 0;
    while i < xs.len() {
        if !f(xs[i]) {
            return false;
        }
        i = i + 1;
    }
    return true;
}

/// Reports whether `f(x)` is true for some element.
pub fn Any(xs: Dynamic, f: Dynamic) -> bool {
    let i = 0;
    while i < xs.len() {
        if f(xs[i]) {
            return true;
        }
        i = i + 1;
    }
    return false;
}

/// Sorts with a `less(a, b)` comparator (insertion sort — prefer the
/// native `sort::Ints/Floats/Strings` for plain orders; this is the
/// generic `slices.SortFunc` entry point).
pub fn SortFunc(xs: Dynamic, less: Dynamic) -> Dynamic {
    let out = Clone(xs);
    let i = 1;
    while i < out.len() {
        let v = out[i];
        let j = i;
        while j > 0 && less(v, out[j - 1]) {
            out[j] = out[j - 1];
            j = j - 1;
        }
        out[j] = v;
        i = i + 1;
    }
    return out;
}

/// Returns a sorted copy of i64 elements (native fast path).
pub fn SortedInts(xs: Dynamic) -> Dynamic {
    return List::sort_ints(xs);
}

/// Returns a sorted copy of f64 elements (native fast path).
pub fn SortedFloats(xs: Dynamic) -> Dynamic {
    return List::sort_floats(xs);
}

/// Returns a sorted copy of String elements (native fast path).
pub fn SortedStrings(xs: Dynamic) -> Dynamic {
    return List::sort_strings(xs);
}

/// Returns `n` copies of `xs` concatenated (Go's `slices.Repeat`).
pub fn Repeat(xs: Dynamic, n: i64) -> Dynamic {
    let out = [];
    let i = 0;
    while i < n {
        let j = 0;
        while j < xs.len() {
            out.push(xs[j]);
            j = j + 1;
        }
        i = i + 1;
    }
    return out;
}

/// Splits into consecutive chunks of at most `n` (Go's `slices.Chunk`;
/// panics when `n <= 0`, like Go).
pub fn Chunk(xs: Dynamic, n: i64) -> Dynamic {
    let out = [];
    let i = 0;
    while i < xs.len() {
        let part = [];
        let j = 0;
        while j < n && i + j < xs.len() {
            part.push(xs[i + j]);
            j = j + 1;
        }
        out.push(part);
        i = i + n;
    }
    return out;
}
