// Lexicon Standard Library — container/ring.
// Go-parity circular-ring vocabulary
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 2). The ring is an
// insertion-ordered vector plus a cursor; `Next`/`Prev` move the cursor
// and return the updated ring (value semantics — reassign).
// Import as `import std::container::ring;`.

/// A circular ring (Go's `ring.Ring`).
pub struct Ring {
    items: Dynamic,
    pos: i64,
}

/// Builds a ring over `values` (Go's `ring.New(n)` element-adapted:
// pass the elements instead of just the size).
pub fn New(values: Dynamic) -> Ring {
    let out = [];
    let i = 0;
    while i < values.len() {
        out.push(values[i]);
        i = i + 1;
    }
    return Ring { items: out, pos: 0 };
}

/// Returns the element count.
pub fn Len(r: Ring) -> i64 {
    return r.items.len();
}

/// Returns the current element (Go's `r.Value`), or 0 when empty.
pub fn Value(r: Ring) -> Dynamic {
    if r.items.len() == 0 {
        return 0;
    }
    return r.items[r.pos];
}

/// Returns a ring with the cursor one step forward (Go's `r.Next()`).
pub fn Next(r: Ring) -> Ring {
    if r.items.len() == 0 {
        return r;
    }
    return Ring { items: r.items, pos: (r.pos + 1) % r.items.len() };
}

/// Returns a ring with the cursor one step back (Go's `r.Prev()`).
pub fn Prev(r: Ring) -> Ring {
    if r.items.len() == 0 {
        return r;
    }
    let p = r.pos - 1;
    if p < 0 {
        p = r.items.len() - 1;
    }
    return Ring { items: r.items, pos: p };
}

/// Returns a ring with the current element replaced (Go's `r.Value = v`
/// assignment, value-adapted).
pub fn Set(r: Ring, v: Dynamic) -> Ring {
    if r.items.len() == 0 {
        return r;
    }
    let items = r.items;
    items[r.pos] = v;
    return Ring { items: items, pos: r.pos };
}

/// Calls `f(value)` for every element starting at the cursor (Go's
/// `r.Do(f)` with a lambda).
pub fn Do(r: Ring, f: Dynamic) -> void {
    let i = 0;
    while i < r.items.len() {
        f(r.items[(r.pos + i) % r.items.len()]);
        i = i + 1;
    }
}

/// Returns the elements starting at the cursor as a plain list.
pub fn ToList(r: Ring) -> Dynamic {
    let out = [];
    let i = 0;
    while i < r.items.len() {
        out.push(r.items[(r.pos + i) % r.items.len()]);
        i = i + 1;
    }
    return out;
}
