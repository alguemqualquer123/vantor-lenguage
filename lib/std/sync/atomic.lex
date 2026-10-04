// Lexicon Standard Library — sync/atomic.
// Go-parity atomic int64 operations
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 3). Cells live in the native
// `Atomic::*` registry and are referenced by id handle, so `Add`/`Swap`/
// `CompareAndSwap` keep atomicity even under the threaded scheduler of
// Fase 8. Import as `import std::sync::atomic;`.

/// An atomic int64 cell (Go's `atomic.Int64`).
pub struct Int64 {
    id: i64,
}

/// Allocates a cell holding `v` (Go's `&atomic.Int64{}` + `Store`).
pub fn NewInt64(v: i64) -> Int64 {
    return Int64 { id: Atomic::make(v) };
}

/// Loads the value (Go's `LoadInt64`).
pub fn Load(a: Int64) -> i64 {
    return Atomic::get(a.id);
}

/// Stores `v` (Go's `StoreInt64`).
pub fn Store(a: Int64, v: i64) -> void {
    Atomic::set(a.id, v);
}

/// Adds `d`, returning the new value (Go's `AddInt64`).
pub fn Add(a: Int64, d: i64) -> i64 {
    return Atomic::add(a.id, d);
}

/// Swaps in `v`, returning the old value (Go's `SwapInt64`).
pub fn Swap(a: Int64, v: i64) -> i64 {
    return Atomic::swap(a.id, v);
}

/// Compares and swaps (Go's `CompareAndSwapInt64`).
pub fn CompareAndSwap(a: Int64, exp: i64, next: i64) -> bool {
    return Atomic::cas(a.id, exp, next);
}

/// Releases the cell (no Go counterpart — keeps long runs lean).
pub fn Free(a: Int64) -> void {
    Atomic::drop(a.id);
}
