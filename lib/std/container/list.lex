// Lexicon Standard Library — container/list.
// Go-parity doubly-linked-list vocabulary
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 2). Elements live in one
// insertion-ordered vector (no raw pointers in the interpreter); the
// API mirrors Go (`PushBack`, `PushFront`, `Front`, `Back`, `Remove`)
// with Lex value semantics — mutating ops return the new list, so write
// `l = list::PushBack(l, v)`. Import as `import std::container::list;`.

/// A list value (Go's `list.List`).
pub struct List {
    items: Dynamic,
}

/// Result of a pop: the advanced list, the value and `ok`.
pub struct ListPopResult {
    list: List,
    value: Dynamic,
    ok: bool,
}

/// Returns an empty list (Go's `list.New`).
pub fn New() -> List {
    return List { items: [] };
}

/// Returns the element count (Go's `l.Len()`).
pub fn Len(l: List) -> i64 {
    return l.items.len();
}

/// Returns the first element, or 0 when empty (Go's `Front().Value`).
pub fn Front(l: List) -> Dynamic {
    if l.items.len() == 0 {
        return 0;
    }
    return l.items[0];
}

/// Returns the last element, or 0 when empty (Go's `Back().Value`).
pub fn Back(l: List) -> Dynamic {
    if l.items.len() == 0 {
        return 0;
    }
    return l.items[l.items.len() - 1];
}

/// Appends `v` (Go's `PushBack`).
pub fn PushBack(l: List, v: Dynamic) -> List {
    let items = l.items;
    items.push(v);
    return List { items: items };
}

/// Prepends `v` (Go's `PushFront`).
pub fn PushFront(l: List, v: Dynamic) -> List {
    let out = [v];
    let i = 0;
    while i < l.items.len() {
        out.push(l.items[i]);
        i = i + 1;
    }
    return List { items: out };
}

/// Removes and returns the last element (mirror of `PushBack`).
pub fn PopBack(l: List) -> ListPopResult {
    if l.items.len() == 0 {
        return ListPopResult { list: l, value: 0, ok: false };
    }
    let v = l.items[l.items.len() - 1];
    let out = [];
    let i = 0;
    while i < l.items.len() - 1 {
        out.push(l.items[i]);
        i = i + 1;
    }
    return ListPopResult { list: List { items: out }, value: v, ok: true };
}

/// Removes and returns the first element (mirror of `PushFront`).
pub fn PopFront(l: List) -> ListPopResult {
    if l.items.len() == 0 {
        return ListPopResult { list: l, value: 0, ok: false };
    }
    let v = l.items[0];
    let out = [];
    let i = 1;
    while i < l.items.len() {
        out.push(l.items[i]);
        i = i + 1;
    }
    return ListPopResult { list: List { items: out }, value: v, ok: true };
}

/// Removes the element at `idx` (Go's `l.Remove(e)` index-adapted).
pub fn Remove(l: List, idx: i64) -> List {
    let out = [];
    let i = 0;
    while i < l.items.len() {
        if i != idx {
            out.push(l.items[i]);
        }
        i = i + 1;
    }
    return List { items: out };
}

/// Returns the elements as a plain list.
pub fn ToList(l: List) -> Dynamic {
    return l.items;
}
