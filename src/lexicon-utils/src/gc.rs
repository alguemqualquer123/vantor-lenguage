// Tracing garbage collector skeleton (Spec §6).
//
// Model: stop-the-world mark/sweep with two generations (young/old),
// explicit root set, iterative marking, generational write barrier
// (old→young edges remembered), heap growth policy hook and production
// telemetry. Concurrent collection is expressed as an explicit
// incremental step API; the production runtime drives it off the
// scheduler tick.

use std::collections::{HashMap, HashSet};

pub type ObjId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Generation {
    Young,
    Old,
}

#[derive(Debug, Clone)]
struct GcObject {
    size: usize,
    refs: Vec<ObjId>,
    generation: Generation,
    marked: bool,
    age: u32,
}

#[derive(Debug, Clone, Default)]
pub struct GcCounters {
    pub collections: u64,
    pub young_collections: u64,
    pub old_collections: u64,
    pub allocated_bytes: u64,
    pub reclaimed_bytes: u64,
    pub live_objects: usize,
    pub last_pause_micros: u64,
    pub max_pause_micros: u64,
}

#[derive(Debug, Clone)]
pub struct HeapPolicy {
    pub young_max_bytes: usize,
    pub old_max_bytes: usize,
    pub promote_after: u32,
}

impl Default for HeapPolicy {
    fn default() -> Self {
        HeapPolicy { young_max_bytes: 4 << 20, old_max_bytes: 512 << 20, promote_after: 2 }
    }
}

pub struct TracingGc {
    objects: HashMap<ObjId, GcObject>,
    roots: HashSet<ObjId>,
    remembered: HashSet<(ObjId, ObjId)>,
    next_id: ObjId,
    young_bytes: usize,
    old_bytes: usize,
    policy: HeapPolicy,
    pub counters: GcCounters,
    // Incremental / "concurrent" marking state (Spec §6).
    // The production runtime drives `step()` off the scheduler tick
    // instead of blocking on a full mark; no background threads are
    // spawned — all progress is explicit and budget-bounded.
    inc_stack: Vec<ObjId>,
    inc_active: bool,
}

impl TracingGc {
    pub fn new(policy: HeapPolicy) -> Self {
        TracingGc {
            objects: HashMap::new(),
            roots: HashSet::new(),
            remembered: HashSet::new(),
            next_id: 1,
            young_bytes: 0,
            old_bytes: 0,
            policy,
            counters: GcCounters::default(),
            inc_stack: Vec::new(),
            inc_active: false,
        }
    }

    /// Allocate an object of `size` bytes referencing `refs`.
    pub fn alloc(&mut self, size: usize, refs: Vec<ObjId>) -> ObjId {
        let id = self.next_id;
        self.next_id += 1;
        self.young_bytes += size;
        self.counters.allocated_bytes += size as u64;
        self.objects.insert(id, GcObject {
            size, refs, generation: Generation::Young, marked: false, age: 0,
        });
        if self.young_bytes >= self.policy.young_max_bytes {
            self.collect_young();
        }
        id
    }

    pub fn add_root(&mut self, id: ObjId) {
        self.roots.insert(id);
    }

    pub fn remove_root(&mut self, id: ObjId) {
        self.roots.remove(&id);
    }

    /// Generational write barrier (Spec §6): record old→young edges so
    /// young collections don't scan the whole old generation.
    pub fn write_barrier(&mut self, obj: ObjId, target: ObjId) {
        let old_gen = self.objects.get(&obj).map(|o| o.generation) == Some(Generation::Old);
        let young = self.objects.get(&target).map(|o| o.generation) == Some(Generation::Young);
        if old_gen && young {
            self.remembered.insert((obj, target));
        }
        if let Some(o) = self.objects.get_mut(&obj) {
            if !o.refs.contains(&target) {
                o.refs.push(target);
            }
        }
    }

    fn mark(&mut self) {
        let mut stack: Vec<ObjId> = self.roots.iter().cloned().collect();
        // Remembered old→young edges seed the young mark set.
        for (_, young) in self.remembered.clone() {
            stack.push(young);
        }
        while let Some(id) = stack.pop() {
            let refs = match self.objects.get_mut(&id) {
                Some(o) if !o.marked => {
                    o.marked = true;
                    o.refs.clone()
                }
                _ => continue,
            };
            stack.extend(refs);
        }
    }

    fn sweep_young(&mut self) -> u64 {
        let mut reclaimed = 0u64;
        let mut dead = Vec::new();
        for (id, o) in self.objects.iter_mut() {
            if o.generation != Generation::Young {
                continue;
            }
            if o.marked {
                o.marked = false;
                o.age += 1;
            } else {
                reclaimed += o.size as u64;
                dead.push(*id);
            }
        }
        for id in dead {
            if let Some(o) = self.objects.remove(&id) {
                self.young_bytes = self.young_bytes.saturating_sub(o.size);
            }
        }
        // Promotion: survivors past the age threshold move to old.
        for o in self.objects.values_mut() {
            if o.generation == Generation::Young && o.age >= self.policy.promote_after {
                o.generation = Generation::Old;
                self.young_bytes = self.young_bytes.saturating_sub(o.size);
                self.old_bytes += o.size;
            }
        }
        reclaimed
    }

    fn sweep_old(&mut self) -> u64 {
        let mut reclaimed = 0u64;
        let mut dead = Vec::new();
        for (id, o) in self.objects.iter_mut() {
            if o.generation != Generation::Old {
                continue;
            }
            if o.marked {
                o.marked = false;
            } else {
                reclaimed += o.size as u64;
                dead.push(*id);
            }
        }
        for id in dead {
            if let Some(o) = self.objects.remove(&id) {
                self.old_bytes = self.old_bytes.saturating_sub(o.size);
            }
        }
        self.remembered.retain(|(a, b)| self.objects.contains_key(a) && self.objects.contains_key(b));
        reclaimed
    }

    fn record_pause(&mut self, micros: u64) {
        self.counters.last_pause_micros = micros;
        if micros > self.counters.max_pause_micros {
            self.counters.max_pause_micros = micros;
        }
    }

    // ------------------------------------------------------------------
    // Incremental ("concurrent") marking API (Spec §6).
    //
    // Contract: no background threads are spawned. The host drives
    // collection explicitly: `start_incremental()` seeds the mark stack
    // from roots + remembered old→young edges, then each `step(budget)`
    // call marks at most `budget` objects and returns `true` once the
    // mark stack drains. The caller then runs `collect_young()` /
    // `collect_full()` (which finish any remaining marks before
    // sweeping) or keeps stepping across scheduler ticks to bound
    // pause times. `step()` is idempotent after completion.
    // ------------------------------------------------------------------

    /// Seed an incremental marking session from the current root set.
    pub fn start_incremental(&mut self) {
        self.inc_stack = self.roots.iter().cloned().collect();
        for (_, young) in self.remembered.iter().cloned() {
            self.inc_stack.push(young);
        }
        self.inc_active = true;
    }

    /// Advance incremental marking by at most `budget` popped objects.
    /// Returns `true` when marking is complete (stack drained).
    /// A `budget` of 0 performs no work unless the stack is already empty.
    pub fn step(&mut self, budget: usize) -> bool {
        if !self.inc_active {
            self.start_incremental();
        }
        let mut processed = 0usize;
        while processed < budget {
            let id = match self.inc_stack.pop() {
                Some(id) => id,
                None => {
                    self.inc_active = false;
                    return true;
                }
            };
            processed += 1;
            let refs = match self.objects.get_mut(&id) {
                Some(o) if !o.marked => {
                    o.marked = true;
                    o.refs.clone()
                }
                _ => continue,
            };
            self.inc_stack.extend(refs);
        }
        if self.inc_stack.is_empty() {
            self.inc_active = false;
            true
        } else {
            false
        }
    }

    /// Whether an incremental marking session is still in progress.
    pub fn is_marking(&self) -> bool {
        self.inc_active
    }

    /// Number of objects still queued in the incremental mark stack.
    pub fn pending_marks(&self) -> usize {
        self.inc_stack.len()
    }

    /// Minor collection: young generation only.
    pub fn collect_young(&mut self) -> u64 {
        let t = std::time::Instant::now();
        self.mark();
        let reclaimed = self.sweep_young();
        self.counters.collections += 1;
        self.counters.young_collections += 1;
        self.counters.reclaimed_bytes += reclaimed;
        self.counters.live_objects = self.objects.len();
        self.record_pause(t.elapsed().as_micros() as u64);
        reclaimed
    }

    /// Major collection: whole heap.
    pub fn collect_full(&mut self) -> u64 {
        let t = std::time::Instant::now();
        self.mark();
        let reclaimed = self.sweep_young() + self.sweep_old();
        self.counters.collections += 1;
        self.counters.old_collections += 1;
        self.counters.reclaimed_bytes += reclaimed;
        self.counters.live_objects = self.objects.len();
        self.record_pause(t.elapsed().as_micros() as u64);
        reclaimed
    }

    /// Memory-pressure hook (Spec §6): collect or signal growth.
    pub fn on_pressure(&mut self) -> PressureAction {
        if self.old_bytes >= self.policy.old_max_bytes {
            PressureAction::CollectFull
        } else if self.young_bytes >= self.policy.young_max_bytes {
            PressureAction::CollectYoung
        } else {
            PressureAction::Grow
        }
    }

    pub fn live_objects(&self) -> usize {
        self.objects.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureAction {
    Grow,
    CollectYoung,
    CollectFull,
}

// ---------------------------------------------------------------------------
// Manual memory: arenas and pools (Spec §6, isolated from safe defaults)
//
// RUNTIME ISOLATION: `BumpArena` / `Pool<T>` are manual-memory escapes
// that MUST NOT be reachable from safe Lexicon code without an `unsafe`
// token. The parser / typeck (owned by another agent) gates `unsafe`
// blocks; this module only provides the runtime backing store plus the
// ownership rules below. Safe defaults always use `TracingGc` or owned
// `Vec`/`Box` values; arenas and pools are opt-in per scope and never
// shared across threads unless the caller provides synchronization.
// ---------------------------------------------------------------------------

/// Bump arena: linear allocation, bulk reset.
///
/// Ownership: the arena owns every byte in `buf`. `alloc` hands out
/// offsets (or derived slices via [`BumpArena::as_slice`]) that borrow
/// the arena — callers MUST NOT use offsets after `reset()` or after
/// the arena drops (lifetimes: treat offsets as `'arena` borrows).
/// Individual frees are not supported by design; drop the whole arena
/// (or `reset()`) to reclaim. `!Sync` sharing must be externally
/// synchronized; by default an arena is single-owner, single-thread.
pub struct BumpArena {
    buf: Vec<u8>,
    cursor: usize,
}

impl BumpArena {
    pub fn new(capacity: usize) -> Self {
        BumpArena { buf: vec![0u8; capacity], cursor: 0 }
    }

    /// Allocate `size` bytes with `align` alignment. Returns offset.
    /// Complexity: O(1). Fails (`None`) when the arena is full — the
    /// caller must grow/reset rather than overlapping existing loans.
    pub fn alloc(&mut self, size: usize, align: usize) -> Option<usize> {
        let align = align.max(1);
        let mis = self.cursor % align;
        let start = if mis == 0 { self.cursor } else { self.cursor + (align - mis) };
        if start + size > self.buf.len() {
            return None;
        }
        self.cursor = start + size;
        Some(start)
    }

    pub fn used(&self) -> usize {
        self.cursor
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }

    /// Borrow the live prefix (`0..used`) of the arena.
    /// Lifetime: the returned slice borrows `self`; it is invalidated
    /// by any subsequent `alloc`/`reset` (documented aliasing rule).
    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.cursor]
    }

    /// Borrow a previously allocated `offset..offset+len` range.
    /// Returns `None` when out of the live prefix (bounds-checked).
    pub fn slice_at(&self, offset: usize, len: usize) -> Option<&[u8]> {
        let end = offset.checked_add(len)?;
        if end <= self.cursor {
            Some(&self.buf[offset..end])
        } else {
            None
        }
    }
}

/// Fixed pool with explicit free list.
///
/// Ownership: slots are loaned by index; each live index has exactly
/// one logical owner until [`Pool::release`] returns it to the free
/// list. `get`/`get_mut` borrow the pool (`&self`/`&mut self`), so the
/// borrow checker statically forbids aliasing `&mut` loans. Slots MUST
/// be released before the pool drops for leak accounting (see `live()`).
/// Lifetime: values live exactly as long as their slot is held; the
/// pool owns all slots. Thread-safety: `Pool<T>` is `Send` iff `T` is
/// `Send`; share across threads only behind a lock or channel.
pub struct Pool<T: Default> {
    slots: Vec<Option<T>>,
    free: Vec<usize>,
}

impl<T: Default> Pool<T> {
    pub fn new(capacity: usize) -> Self {
        let mut slots = Vec::with_capacity(capacity);
        slots.resize_with(capacity, || None);
        Pool { slots, free: (0..capacity).rev().collect() }
    }

    pub fn acquire(&mut self) -> Option<usize> {
        let idx = self.free.pop()?;
        self.slots[idx] = Some(T::default());
        Some(idx)
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.slots.get(idx)?.as_ref()
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.slots.get_mut(idx)?.as_mut()
    }

    pub fn release(&mut self, idx: usize) -> bool {
        if idx < self.slots.len() && self.slots[idx].is_some() {
            self.slots[idx] = None;
            self.free.push(idx);
            true
        } else {
            false
        }
    }

    pub fn live(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    pub fn is_full(&self) -> bool {
        self.free.is_empty()
    }
}

/// Part X — distributed / actor model (Spec Part X).
///
/// Contract: actors own their state; the ONLY cross-actor channel is
/// the [`Mailbox`] message queue. Messages are `Send + 'static`
/// (no shared borrows cross actors). Delivery is FIFO per sender
/// through [`Mailbox`]; remote/distributed transports implement the
/// same `send`/`recv` shape with cancellation + timeout at the call
/// site. Backpressure is explicit: `send` fails when `close`d, and
/// `len`/`capacity` let schedulers shed load instead of growing
/// unbounded queues. Complexity: send/recv O(1) amortised.
// ---------------------------------------------------------------------------

/// Actor behaviour: stateful message handler (Spec Part X).
pub trait Actor: Send + 'static {
    type Message: Send + 'static;
    fn name(&self) -> &str;
    fn handle(&mut self, msg: Self::Message);
}

/// Bounded FIFO mailbox between actors (in-process transport).
pub struct Mailbox<T: Send + 'static> {
    tx: std::sync::mpsc::SyncSender<T>,
    rx: std::sync::mpsc::Receiver<T>,
    capacity: usize,
    len: std::sync::atomic::AtomicUsize,
}

impl<T: Send + 'static> Mailbox<T> {
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel(capacity.max(1));
        Mailbox { tx, rx, capacity: capacity.max(1), len: std::sync::atomic::AtomicUsize::new(0) }
    }

    /// Non-blocking send; `Err(msg)` (message returned) when full/closed.
    pub fn send(&self, msg: T) -> Result<(), T> {
        self.tx.try_send(msg).map_err(|e| match e {
            std::sync::mpsc::TrySendError::Full(m)
            | std::sync::mpsc::TrySendError::Disconnected(m) => m,
        }).map(|()| {
            self.len.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        })
    }

    /// Non-blocking receive; `None` when empty/disconnected.
    pub fn try_recv(&self) -> Option<T> {
        let v = self.rx.try_recv().ok()?;
        self.len.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        Some(v)
    }

    /// Blocking receive (caller applies its own timeout/cancel).
    pub fn recv(&self) -> Option<T> {
        let v = self.rx.recv().ok()?;
        self.len.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        Some(v)
    }

    pub fn len(&self) -> usize {
        self.len.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Drive one actor until its mailbox drains (deterministic step for
/// tests and the scheduler tick; no threads spawned here).
pub fn drain_actor<A: Actor>(actor: &mut A, mailbox: &Mailbox<A::Message>) {
    while let Some(msg) = mailbox.try_recv() {
        actor.handle(msg);
    }
}

// ---------------------------------------------------------------------------
// Part X / §61 — game APIs: ECS layout + job system stub + GPU/native hooks.
//
// Rules: ECS storage is SoA-friendly (components in dense `Vec`s keyed
// by generational [`Entity`]); systems borrow storages explicitly so
// aliasing is visible. `JobSystem` is a deterministic stub: jobs run
// FIFO on `drain` (no hidden thread pool), so game logic stays
// reproducible; a threaded pool is a backend binding. GUI-native hooks
// below describe windows/events without touching OS APIs (real
// windowing lives in `lexicon-gui`).
// ---------------------------------------------------------------------------

/// Generational entity id (index + generation guards use-after-free).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub index: u32,
    pub generation: u32,
}

/// Dense component storage with generational liveness.
#[derive(Debug, Default)]
pub struct ComponentStorage<T> {
    slots: Vec<Option<T>>,
    generations: Vec<u32>,
    free: Vec<u32>,
}

impl<T> ComponentStorage<T> {
    pub fn new() -> Self {
        ComponentStorage { slots: Vec::new(), generations: Vec::new(), free: Vec::new() }
    }

    pub fn spawn(&mut self, value: T) -> Entity {
        if let Some(idx) = self.free.pop() {
            let i = idx as usize;
            self.slots[i] = Some(value);
            return Entity { index: idx, generation: self.generations[i] };
        }
        let idx = self.slots.len() as u32;
        self.slots.push(Some(value));
        self.generations.push(0);
        Entity { index: idx, generation: 0 }
    }

    pub fn despawn(&mut self, e: Entity) -> bool {
        let i = e.index as usize;
        if i < self.slots.len() && self.generations[i] == e.generation && self.slots[i].is_some() {
            self.slots[i] = None;
            self.generations[i] = self.generations[i].wrapping_add(1);
            self.free.push(e.index);
            true
        } else {
            false
        }
    }

    pub fn get(&self, e: Entity) -> Option<&T> {
        let i = e.index as usize;
        if i < self.slots.len() && self.generations[i] == e.generation {
            self.slots[i].as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, e: Entity) -> Option<&mut T> {
        let i = e.index as usize;
        if i < self.slots.len() && self.generations[i] == e.generation {
            self.slots[i].as_mut()
        } else {
            None
        }
    }

    pub fn live_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }
}

/// Minimal world: entity liveness + two demo component stores.
/// Real engines add systems/schedules; layout rules stay identical.
#[derive(Debug, Default)]
pub struct World {
    next: u32,
    alive: Vec<bool>,
}

impl World {
    pub fn new() -> Self {
        World { next: 0, alive: Vec::new() }
    }

    pub fn spawn(&mut self) -> Entity {
        let idx = self.next;
        self.next += 1;
        if idx as usize >= self.alive.len() {
            self.alive.resize(idx as usize + 1, false);
        }
        self.alive[idx as usize] = true;
        Entity { index: idx, generation: 0 }
    }

    pub fn despawn(&mut self, e: Entity) -> bool {
        if (e.index as usize) < self.alive.len() && self.alive[e.index as usize] {
            self.alive[e.index as usize] = false;
            true
        } else {
            false
        }
    }

    pub fn is_alive(&self, e: Entity) -> bool {
        (e.index as usize) < self.alive.len() && self.alive[e.index as usize]
    }

    pub fn live_count(&self) -> usize {
        self.alive.iter().filter(|&&a| a).count()
    }
}

/// Deterministic job-system stub (FIFO, single-threaded drain).
///
/// Ownership: jobs are `FnOnce() + Send + 'static`; the system owns
/// them until `drain` runs each exactly once in submission order.
/// Thread-safety: `JobSystem` itself is `!Sync` by convention — share
/// across threads behind a lock or channel. Complexity: submit O(1);
/// drain O(jobs).
pub struct JobSystem {
    jobs: Vec<(String, Box<dyn FnOnce() + Send>)>,
}

impl JobSystem {
    pub fn new() -> Self {
        JobSystem { jobs: Vec::new() }
    }

    pub fn submit(&mut self, name: &str, job: impl FnOnce() + Send + 'static) {
        self.jobs.push((name.to_string(), Box::new(job)));
    }

    pub fn pending(&self) -> usize {
        self.jobs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    /// Run all queued jobs FIFO, returning their names in run order.
    pub fn drain(&mut self) -> Vec<String> {
        let jobs = std::mem::take(&mut self.jobs);
        let mut order = Vec::with_capacity(jobs.len());
        for (name, job) in jobs {
            job();
            order.push(name);
        }
        order
    }
}

impl Default for JobSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Part X — native GUI stubs (utils-level descriptors; no OS calls).
//
// `lexicon-gui` owns real windowing; these descriptors let the runtime,
// tooling and tests plan layouts/events portably (headless CI runs the
// same code with the `Headless` backend).
// ---------------------------------------------------------------------------

/// GUI rendering backend selector (stub: selection only, no init).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiBackend {
    Headless,
    Software,
    Wgpu,
    Native,
}

/// Portable window descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiWindowDesc {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub backend: GuiBackend,
}

impl GuiWindowDesc {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        GuiWindowDesc { title: title.into(), width: width.max(1), height: height.max(1), backend: GuiBackend::Headless }
    }

    pub fn with_backend(mut self, backend: GuiBackend) -> Self {
        self.backend = backend;
        self
    }

    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

/// Portable GUI event (produced by real backends, consumed by handlers).
#[derive(Debug, Clone, PartialEq)]
pub enum GuiEvent {
    Close,
    Resize { width: u32, height: u32 },
    Key { code: u32, pressed: bool },
    Mouse { x: i32, y: i32, pressed: bool },
    Tick { dt_ms: u64 },
}

/// Bounds check for safe code (Spec §44): returns the index on success,
/// or a diagnostic string identifying the violation.
pub fn check_bounds(index: usize, len: usize, what: &str) -> Result<usize, String> {
    if index < len {
        Ok(index)
    } else {
        Err(format!("E0501: out-of-bounds access on {}: index {} with length {}", what, index, len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn young_collection_reclaims_unrooted() {
        let mut gc = TracingGc::new(HeapPolicy {
            young_max_bytes: usize::MAX,
            old_max_bytes: usize::MAX,
            promote_after: 100,
        });
        let a = gc.alloc(64, vec![]);
        let b = gc.alloc(64, vec![a]);
        gc.add_root(b);
        let _c = gc.alloc(64, vec![]);
        let reclaimed = gc.collect_young();
        assert_eq!(reclaimed, 64);
        assert_eq!(gc.live_objects(), 2);
    }

    #[test]
    fn promotion_to_old_after_threshold() {
        let mut gc = TracingGc::new(HeapPolicy {
            young_max_bytes: usize::MAX,
            old_max_bytes: usize::MAX,
            promote_after: 1,
        });
        let a = gc.alloc(32, vec![]);
        gc.add_root(a);
        gc.collect_young();
        assert_eq!(gc.objects.get(&a).unwrap().generation, Generation::Old);
    }

    #[test]
    fn write_barrier_remembers_edge() {
        let mut gc = TracingGc::new(HeapPolicy {
            young_max_bytes: usize::MAX,
            old_max_bytes: usize::MAX,
            promote_after: 0,
        });
        let old = gc.alloc(16, vec![]);
        gc.add_root(old);
        gc.collect_young();
        let young = gc.alloc(16, vec![]);
        gc.write_barrier(old, young);
        assert!(gc.remembered.contains(&(old, young)));
    }

    #[test]
    fn arena_bump_and_reset() {
        let mut arena = BumpArena::new(64);
        let a = arena.alloc(8, 8).unwrap();
        let b = arena.alloc(8, 8).unwrap();
        assert_eq!((a, b), (0, 8));
        assert!(arena.alloc(64, 1).is_none());
        arena.reset();
        assert_eq!(arena.used(), 0);
    }

    #[test]
    fn pool_acquire_release() {
        let mut pool: Pool<String> = Pool::new(2);
        let a = pool.acquire().unwrap();
        let b = pool.acquire().unwrap();
        assert!(pool.acquire().is_none());
        assert_eq!(pool.live(), 2);
        assert!(pool.release(a));
        assert_eq!(pool.live(), 1);
        let _ = b;
    }

    #[test]
    fn bounds_check_reports() {
        assert_eq!(check_bounds(2, 5, "slice"), Ok(2));
        assert!(check_bounds(5, 5, "slice").is_err());
    }

    #[test]
    fn incremental_step_marks_within_budget() {
        let mut gc = TracingGc::new(HeapPolicy {
            young_max_bytes: usize::MAX,
            old_max_bytes: usize::MAX,
            promote_after: 100,
        });
        // Chain a <- b <- c <- d <- e; root at e keeps all alive.
        let a = gc.alloc(16, vec![]);
        let b = gc.alloc(16, vec![a]);
        let c = gc.alloc(16, vec![b]);
        let d = gc.alloc(16, vec![c]);
        let e = gc.alloc(16, vec![d]);
        gc.add_root(e);
        gc.start_incremental();
        assert!(gc.is_marking());
        // Budget 2: should still be in progress (5 reachable).
        assert!(!gc.step(2));
        assert!(gc.is_marking());
        // Drain the rest; completion is idempotent.
        assert!(gc.step(100));
        assert!(!gc.is_marking());
        assert!(gc.step(10));
        // A full collection now reclaims nothing: everything was marked.
        let reclaimed = gc.collect_young();
        assert_eq!(reclaimed, 0);
        assert_eq!(gc.live_objects(), 5);
    }

    #[test]
    fn incremental_step_zero_budget_is_noop_until_drained() {
        let mut gc = TracingGc::new(HeapPolicy {
            young_max_bytes: usize::MAX,
            old_max_bytes: usize::MAX,
            promote_after: 100,
        });
        let a = gc.alloc(8, vec![]);
        gc.add_root(a);
        gc.start_incremental();
        assert!(!gc.step(0));
        assert!(gc.step(8));
    }

    #[test]
    fn arena_slice_views() {
        let mut arena = BumpArena::new(32);
        let off = arena.alloc(4, 1).unwrap();
        assert_eq!(off, 0);
        assert_eq!(arena.as_slice().len(), 4);
        assert!(arena.slice_at(0, 4).is_some());
        assert!(arena.slice_at(0, 5).is_none());
    }

    #[test]
    fn pool_capacity_helpers() {
        let mut pool: Pool<u32> = Pool::new(1);
        assert_eq!(pool.capacity(), 1);
        assert!(!pool.is_full());
        let _a = pool.acquire().unwrap();
        assert!(pool.is_full());
    }

    #[test]
    fn actor_mailbox_fifo_and_drain() {
        struct Counter {
            count: i32,
        }
        impl Actor for Counter {
            type Message = i32;
            fn name(&self) -> &str {
                "counter"
            }
            fn handle(&mut self, msg: i32) {
                self.count += msg;
            }
        }
        let mb = Mailbox::new(4);
        assert_eq!(mb.capacity(), 4);
        mb.send(1).unwrap();
        mb.send(2).unwrap();
        assert_eq!(mb.len(), 2);
        let mut actor = Counter { count: 0 };
        drain_actor(&mut actor, &mb);
        assert_eq!(actor.count, 3);
        assert!(mb.is_empty());
        assert_eq!(actor.name(), "counter");
    }

    #[test]
    fn ecs_generational_storage() {
        let mut store: ComponentStorage<String> = ComponentStorage::new();
        let a = store.spawn("pos".to_string());
        assert_eq!(store.get(a).map(|s| s.as_str()), Some("pos"));
        assert_eq!(store.live_count(), 1);
        assert!(store.despawn(a));
        assert!(store.get(a).is_none());
        // Stale handle stays dead even after slot reuse.
        let b = store.spawn("vel".to_string());
        assert_ne!(a.generation, b.generation);
        assert!(store.get(a).is_none());
        let _ = b;
    }

    #[test]
    fn world_and_job_system_deterministic() {
        let mut w = World::new();
        let a = w.spawn();
        let b = w.spawn();
        assert!(w.is_alive(a) && w.is_alive(b));
        assert_eq!(w.live_count(), 2);
        assert!(w.despawn(a));
        assert!(!w.is_alive(a));

        let order = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut js = JobSystem::new();
        for name in ["j1", "j2", "j3"] {
            let o2 = order.clone();
            let n = name.to_string();
            let n2 = n.clone();
            js.submit(&n, move || o2.lock().unwrap().push(n2));
        }
        assert_eq!(js.pending(), 3);
        assert_eq!(js.drain(), vec!["j1".to_string(), "j2".to_string(), "j3".to_string()]);
        assert!(js.is_empty());
        assert_eq!(*order.lock().unwrap(), vec!["j1".to_string(), "j2".to_string(), "j3".to_string()]);
    }

    #[test]
    fn gui_descriptors_headless() {
        let d = GuiWindowDesc::new("Game", 800, 600).with_backend(GuiBackend::Headless);
        assert_eq!(d.area(), 480_000);
        let ev = GuiEvent::Resize { width: 1024, height: 768 };
        assert!(matches!(ev, GuiEvent::Resize { .. }));
        assert_eq!(GuiEvent::Tick { dt_ms: 16 }, GuiEvent::Tick { dt_ms: 16 });
    }
}
