// Lexicon Standard Library — sync/pool.
// Go-parity object pool (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 3).
// Lex lists copy on binding, so pool operations are value-semantic like the
// rest of the stdlib: reassign the pool — `p = pool::Put(p, x);`. The
// interpreter is single-threaded, so `sync.Pool`'s per-P caches collapse to
// one LIFO stack; observable behaviour (reuse, empty Get) matches Go.
// Import as `import std::sync::pool;`.

/// A pool of reusable values.
pub struct Pool {
    items: Dynamic,
}

/// Result of `Get`: the `value`, whether one was available, and the pool
/// left behind (Go's `Get()` returning nil on an empty pool).
pub struct Taken {
    value: Dynamic,
    ok: bool,
    pool: Pool,
}

/// An empty pool (Go's `var pool sync.Pool` / `&sync.Pool{New: ...}`).
pub fn New() -> Pool {
    return Pool { items: [] };
}

/// Returns a pool with `v` added (Go's `pool.Put`).
pub fn Put(p: Pool, v: Dynamic) -> Pool {
    let xs = p.items;
    xs.push(v);
    return Pool { items: xs };
}

/// Takes the most recently returned value (Go's `pool.Get`; LIFO like the
/// per-P stack). `ok=false` and an untouched pool when empty.
pub fn Get(p: Pool) -> Taken {
    let xs = p.items;
    let n = xs.len();
    if n == 0 {
        return Taken { value: "", ok: false, pool: p };
    }
    let last = xs[n - 1];
    let rest = [];
    let i = 0;
    while i < n - 1 {
        rest.push(xs[i]);
        i = i + 1;
    }
    return Taken { value: last, ok: true, pool: Pool { items: rest } };
}

/// Number of pooled objects (Go's unexported `poolCleanup` count, exposed
/// for tests).
pub fn Len(p: Pool) -> i64 {
    let xs = p.items;
    return xs.len();
}
