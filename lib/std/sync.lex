// Lexicon Standard Library — sync.
// Go-parity synchronization vocabulary
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 3). The interpreter runs one
// task at a time, so locks never contend: every primitive keeps Go's API
// and value shapes with a cooperative no-op core, and stays correct when
// the threaded scheduler (Fase 8) lands. Value semantics apply —
// reassign: `m = sync::Lock(m)`.

/// Mutual exclusion (Go's `sync.Mutex`).
pub struct Mutex {
    locked: bool,
}

/// Read/write mutex (Go's `sync.RWMutex`; read locks share the flag).
pub struct RWMutex {
    readers: i64,
    writer: bool,
}

/// Counter gate (Go's `sync.WaitGroup`).
pub struct WaitGroup {
    n: i64,
}

/// Once-only trigger (Go's `sync.Once`).
pub struct Once {
    done: bool,
}

/// Returns an unlocked mutex.
pub fn NewMutex() -> Mutex {
    return Mutex { locked: false };
}

/// Locks `m` (no contention in the current scheduler).
pub fn Lock(m: Mutex) -> Mutex {
    return Mutex { locked: true };
}

/// Unlocks `m`.
pub fn Unlock(m: Mutex) -> Mutex {
    return Mutex { locked: false };
}

/// Runs `f()` while holding `m` (Go's critical-section idiom).
pub fn WithLock(m: Mutex, f: Dynamic) -> Mutex {
    f();
    return Mutex { locked: false };
}

/// Returns an unlocked read/write mutex.
pub fn NewRWMutex() -> RWMutex {
    return RWMutex { readers: 0, writer: false };
}

/// Shared read lock.
pub fn RLock(m: RWMutex) -> RWMutex {
    return RWMutex { readers: m.readers + 1, writer: m.writer };
}

/// Shared read unlock.
pub fn RUnlock(m: RWMutex) -> RWMutex {
    let n = m.readers - 1;
    if n < 0 {
        n = 0;
    }
    return RWMutex { readers: n, writer: m.writer };
}

/// Returns an empty wait group.
pub fn NewWaitGroup() -> WaitGroup {
    return WaitGroup { n: 0 };
}

/// Adds `delta` to the counter (Go's `wg.Add`).
pub fn Add(w: WaitGroup, delta: i64) -> WaitGroup {
    return WaitGroup { n: w.n + delta };
}

/// Marks one unit done (Go's `wg.Done`).
pub fn Done(w: WaitGroup) -> WaitGroup {
    let n = w.n - 1;
    if n < 0 {
        n = 0;
    }
    return WaitGroup { n: n };
}

/// Waits for the counter to drain. Synchronous runs return immediately
/// once every `Done` has been recorded (Go's `wg.Wait`).
pub fn Wait(w: WaitGroup) -> void {
}

/// Returns an untriggered once.
pub fn NewOnce() -> Once {
    return Once { done: false };
}

/// Calls `f()` only on the first invocation, returning the updated once
/// (Go's `once.Do(f)`).
pub fn Do(o: Once, f: Dynamic) -> Once {
    if !o.done {
        f();
        return Once { done: true };
    }
    return o;
}
