// Lexicon SDK — native signature stubs (Atomic).
// Documentation for IDE navigation ONLY (see native/console.lex header).

/// Allocates a shared atomic-i64 cell; returns its handle.
pub fn make(v: i64) -> i64 {
    panic("native stub");
}

/// Atomic load.
pub fn get(id: i64) -> i64 {
    panic("native stub");
}

/// Atomic store.
pub fn set(id: i64, v: i64) -> void {
    panic("native stub");
}

/// Atomically adds `d`; returns the new value.
pub fn add(id: i64, d: i64) -> i64 {
    panic("native stub");
}

/// Atomically swaps; returns the old value.
pub fn swap(id: i64, v: i64) -> i64 {
    panic("native stub");
}

/// Atomic compare-and-swap.
pub fn cas(id: i64, old: i64, new: i64) -> bool {
    panic("native stub");
}

/// Releases the cell.
pub fn drop(id: i64) -> void {
    panic("native stub");
}
