// Lexicon Standard Library — iter.
// Go-parity iterator adapters (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 2).
// Higher-order helpers take lambdas (`|x| x * 2`), now first-class values
// in the interpreter: `iter::Map([1, 2], |x| x * 2)`.

/// Result of pulling one element: the advanced iterator, the value and `ok`.
pub struct NextResult {
    it: Iter,
    value: Dynamic,
    ok: bool,
}

/// A pull iterator over a snapshot list (Go's `iter.Seq` pull form).
pub struct Iter {
    items: Dynamic,
    idx: i64,
}

/// Builds an iterator over `items`.
pub fn New(items: Dynamic) -> Iter {
    return Iter { items: items, idx: 0 };
}

/// Pulls the next element (`ok == false` when exhausted). Reassign:
/// `let n = iter::Next(it); it = n.it;`.
pub fn Next(it: Iter) -> NextResult {
    let pos = it.idx;
    let total = it.items.len();
    if pos < total {
        let v = it.items[it.idx];
        let next = Iter { items: it.items, idx: it.idx + 1 };
        return NextResult { it: next, value: v, ok: true };
    }
    return NextResult { it: it, value: 0, ok: false };
}

/// Returns a new list with `f` applied to every element (Go-style map).
pub fn Map(items: Dynamic, f: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < items.len() {
        out.push(f(items[i]));
        i = i + 1;
    }
    return out;
}

/// Returns the elements where `f(x)` is true (Go-style filter).
pub fn Filter(items: Dynamic, f: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < items.len() {
        if f(items[i]) {
            out.push(items[i]);
        }
        i = i + 1;
    }
    return out;
}

/// Folds from the left: `acc = f(acc, x)` (Go-style reduce).
pub fn Reduce(items: Dynamic, init: Dynamic, f: Dynamic) -> Dynamic {
    let acc = init;
    let i = 0;
    while i < items.len() {
        acc = f(acc, items[i]);
        i = i + 1;
    }
    return acc;
}

/// Returns the first `n` elements.
pub fn Take(items: Dynamic, n: i64) -> Dynamic {
    let out = [];
    let i = 0;
    while i < items.len() && i < n {
        out.push(items[i]);
        i = i + 1;
    }
    return out;
}

/// Returns everything after the first `n` elements.
pub fn Drop(items: Dynamic, n: i64) -> Dynamic {
    let out = [];
    let i = n;
    while i < items.len() {
        out.push(items[i]);
        i = i + 1;
    }
    return out;
}

/// Concatenates two sequences (Go's `iter` chaining).
pub fn Chain(a: Dynamic, b: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < a.len() {
        out.push(a[i]);
        i = i + 1;
    }
    let j = 0;
    while j < b.len() {
        out.push(b[j]);
        j = j + 1;
    }
    return out;
}

/// Counts elements where `f(x)` is true.
pub fn Count(items: Dynamic, f: Dynamic) -> i64 {
    let n = 0;
    let i = 0;
    while i < items.len() {
        if f(items[i]) {
            n = n + 1;
        }
        i = i + 1;
    }
    return n;
}
