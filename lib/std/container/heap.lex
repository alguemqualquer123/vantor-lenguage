// Lexicon Standard Library — container/heap.
// Go-parity binary-heap vocabulary
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 2). The comparator is a
// lambda `less(a, b) -> bool` stored in the heap value, so `Push`/`Pop`
// need no extra arguments after `New`: `h = heap::Push(h, v)`.
// Import as `import std::container::heap;`.

/// A min/max-heap depending on `less` (Go's `heap.Interface` + slice).
pub struct Heap {
    items: Dynamic,
    less: Dynamic,
}

/// Result of a pop: the advanced heap, the top value and `ok`.
pub struct HeapPopResult {
    heap: Heap,
    value: Dynamic,
    ok: bool,
}

/// Builds an empty heap ordering with `less(a, b)` (true when `a`
/// belongs above `b`; e.g. `|a, b| a < b` for a min-heap).
pub fn New(less: Dynamic) -> Heap {
    return Heap { items: [], less: less };
}

/// Returns the element count.
pub fn Len(h: Heap) -> i64 {
    return h.items.len();
}

/// Returns the top element without removing it, or 0 when empty.
pub fn Peek(h: Heap) -> Dynamic {
    if h.items.len() == 0 {
        return 0;
    }
    return h.items[0];
}

/// Pushes `v`, restoring the heap order (Go's `heap.Push`).
pub fn Push(h: Heap, v: Dynamic) -> Heap {
    let items = h.items;
    items.push(v);
    let i = items.len() - 1;
    while i > 0 {
        let p = (i - 1) / 2;
        if h.less(items[i], items[p]) {
            let t = items[i];
            items[i] = items[p];
            items[p] = t;
            i = p;
        } else {
            break;
        }
    }
    return Heap { items: items, less: h.less };
}

/// Removes and returns the top element (Go's `heap.Pop`).
pub fn Pop(h: Heap) -> HeapPopResult {
    if h.items.len() == 0 {
        return HeapPopResult { heap: h, value: 0, ok: false };
    }
    let top = h.items[0];
    let items = h.items;
    items[0] = items[items.len() - 1];
    items.pop();
    let i = 0;
    while true {
        let l = 2 * i + 1;
        let r = 2 * i + 2;
        let best = i;
        if l < items.len() && h.less(items[l], items[best]) {
            best = l;
        }
        if r < items.len() && h.less(items[r], items[best]) {
            best = r;
        }
        if best == i {
            break;
        }
        let t = items[i];
        items[i] = items[best];
        items[best] = t;
        i = best;
    }
    return HeapPopResult { heap: Heap { items: items, less: h.less }, value: top, ok: true };
}
