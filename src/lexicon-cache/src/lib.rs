//! Collections for Lexicon (Spec §10).
//!
//! This crate provides the documented runtime collection set on top of
//! [`MemoryCache`]. Every type below documents: complexity of major ops,
//! ownership, iterator-invalidation rules, thread-safety, and ordering
//! guarantees.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// MemoryCache (pre-existing; kept working)
// ---------------------------------------------------------------------------

pub struct MemoryCache<K, V> {
    store: Arc<RwLock<HashMap<K, V>>>,
}

impl<K, V> MemoryCache<K, V> {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<K, V> MemoryCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn get(&self, key: &K) -> Option<V> {
        self.store.read().get(key).cloned()
    }

    pub fn set(&self, key: K, value: V) {
        self.store.write().insert(key, value);
    }

    pub fn remove(&self, key: &K) {
        self.store.write().remove(key);
    }

    pub fn clear(&self) {
        self.store.write().clear();
    }

    pub fn len(&self) -> usize {
        self.store.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.read().is_empty()
    }
}

impl<K, V> Default for MemoryCache<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Clone for MemoryCache<K, V> {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
        }
    }
}

// ---------------------------------------------------------------------------
// RingBuffer<T>
// ---------------------------------------------------------------------------

/// Fixed-capacity FIFO ring that overwrites the oldest element when full.
///
/// Complexity: `push` O(1) amortised, `pop` O(1), `len`/`is_empty` O(1),
/// `iter` O(n).
/// Ownership: the buffer owns all elements; `pop` transfers ownership to
/// the caller; `push` on a full buffer drops (returns) the evicted oldest.
/// Iterator-invalidation: `iter()` borrows the buffer; any `push`/`pop`
/// invalidates outstanding iterators/slices (do not hold across mutation).
/// Thread-safety: `Send` iff `T: Send`; `!Sync` — share across threads
/// only behind a lock or channel.
/// Ordering: FIFO from oldest to newest; full `push` evicts the oldest.
pub struct RingBuffer<T> {
    inner: std::collections::VecDeque<T>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        RingBuffer {
            inner: std::collections::VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Push `item`; if full, evicts and returns the oldest element.
    pub fn push(&mut self, item: T) -> Option<T> {
        if self.capacity == 0 {
            return Some(item);
        }
        let evicted = if self.inner.len() == self.capacity {
            self.inner.pop_front()
        } else {
            None
        };
        self.inner.push_back(item);
        evicted
    }

    /// Pop the oldest element. Returns `None` when empty.
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.inner.len() == self.capacity
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }
}

// ---------------------------------------------------------------------------
// BitSet
// ---------------------------------------------------------------------------

/// Fixed-size dense bit set backed by `Vec<u64>`.
///
/// Complexity: `set`/`clear`/`get` O(1); `count_ones` O(n/64);
/// `union_with`/`intersect_with` O(n/64).
/// Ownership: owns its words; all ops are by value/index, no aliasing.
/// Iterator-invalidation: N/A (no iterators handed out; `get` returns `bool`).
/// Thread-safety: `Send` + `!Sync` by convention — single-owner; share
/// behind a lock for concurrent use.
/// Ordering: bit-index order `0..nbits`; word layout is an implementation
/// detail and not exposed.
pub struct BitSet {
    words: Vec<u64>,
    nbits: usize,
}

impl BitSet {
    pub fn new(nbits: usize) -> Self {
        let words = vec![0u64; nbits.div_ceil(64)];
        BitSet { words, nbits }
    }

    pub fn len(&self) -> usize {
        self.nbits
    }

    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|w| *w == 0)
    }

    fn check(&self, index: usize) -> bool {
        index < self.nbits
    }

    /// Set bit `index`. Returns `false` when out of range.
    pub fn set(&mut self, index: usize) -> bool {
        if !self.check(index) {
            return false;
        }
        self.words[index / 64] |= 1u64 << (index % 64);
        true
    }

    /// Clear bit `index`. Returns `false` when out of range.
    pub fn clear(&mut self, index: usize) -> bool {
        if !self.check(index) {
            return false;
        }
        self.words[index / 64] &= !(1u64 << (index % 64));
        true
    }

    /// Read bit `index`. Returns `None` when out of range.
    pub fn get(&self, index: usize) -> Option<bool> {
        if !self.check(index) {
            return None;
        }
        Some(self.words[index / 64] & (1u64 << (index % 64)) != 0)
    }

    /// Number of bits set.
    pub fn count_ones(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    fn binary_op(&mut self, other: &BitSet, f: impl Fn(u64, u64) -> u64) {
        let n = self.words.len().min(other.words.len());
        for i in 0..n {
            self.words[i] = f(self.words[i], other.words[i]);
        }
    }

    pub fn union_with(&mut self, other: &BitSet) {
        self.binary_op(other, |a, b| a | b);
    }

    pub fn intersect_with(&mut self, other: &BitSet) {
        self.binary_op(other, |a, b| a & b);
    }
}

// ---------------------------------------------------------------------------
// LexStack<T>
// ---------------------------------------------------------------------------

/// LIFO stack (`Vec` wrapper).
///
/// Complexity: `push` O(1) amortised, `pop`/`peek` O(1), `len` O(1).
/// Ownership: owns elements; `pop` moves the top out; `peek` borrows.
/// Iterator-invalidation: `peek`/iter borrows are invalidated by any `push`
/// that reallocates (treat borrows as short-lived).
/// Thread-safety: `Send` iff `T: Send`; `!Sync` — single-owner by default.
/// Ordering: LIFO (last pushed is first popped).
pub struct LexStack<T> {
    inner: Vec<T>,
}

impl<T> LexStack<T> {
    pub fn new() -> Self {
        LexStack { inner: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.inner.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.inner.last()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }
}

impl<T> Default for LexStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// LexQueue<T>
// ---------------------------------------------------------------------------

/// FIFO queue (`VecDeque` wrapper).
///
/// Complexity: `enqueue`/`dequeue`/`peek` O(1), `len` O(1).
/// Ownership: owns elements; `dequeue` moves the front out.
/// Iterator-invalidation: borrows are invalidated by mutation that grows
/// the buffer; do not hold `peek`/iter across `enqueue`/`dequeue`.
/// Thread-safety: `Send` iff `T: Send`; `!Sync` — use [`ConcurrentQueue`]
/// for multi-threaded producers/consumers.
/// Ordering: strict FIFO.
pub struct LexQueue<T> {
    inner: std::collections::VecDeque<T>,
}

impl<T> LexQueue<T> {
    pub fn new() -> Self {
        LexQueue {
            inner: std::collections::VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, item: T) {
        self.inner.push_back(item);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    pub fn peek(&self) -> Option<&T> {
        self.inner.front()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }
}

impl<T> Default for LexQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// LexDeque<T>
// ---------------------------------------------------------------------------

/// Double-ended queue (`VecDeque` wrapper).
///
/// Complexity: `push_front`/`push_back`/`pop_front`/`pop_back`/`get` O(1),
/// `len` O(1).
/// Ownership: owns elements; pops move values out; `get` borrows.
/// Iterator-invalidation: any push/pop that reallocates invalidates
/// outstanding `get`/iter borrows.
/// Thread-safety: `Send` iff `T: Send`; `!Sync` — single-owner.
/// Ordering: insertion order; front-to-back iteration.
pub struct LexDeque<T> {
    inner: std::collections::VecDeque<T>,
}

impl<T> LexDeque<T> {
    pub fn new() -> Self {
        LexDeque {
            inner: std::collections::VecDeque::new(),
        }
    }

    pub fn push_front(&mut self, item: T) {
        self.inner.push_front(item);
    }

    pub fn push_back(&mut self, item: T) {
        self.inner.push_back(item);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.inner.pop_back()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }
}

impl<T> Default for LexDeque<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// PriorityQueue<T>
// ---------------------------------------------------------------------------

/// Max-heap priority queue (`BinaryHeap` wrapper).
///
/// Complexity: `push` O(log n), `pop` O(log n), `peek` O(1), `len` O(1).
/// Ownership: owns elements; `pop` moves the greatest out.
/// Iterator-invalidation: heap order is internal; `iter()` yields elements
/// in arbitrary heap order and borrows the queue — mutation invalidates it.
/// Thread-safety: `Send` iff `T: Send`; `!Sync` — single-owner.
/// Ordering: greatest `Ord` value pops first; use
/// `std::cmp::Reverse` for min-heap behaviour. Not stable: equal elements
/// pop in unspecified order.
pub struct PriorityQueue<T: Ord> {
    inner: std::collections::BinaryHeap<T>,
}

impl<T: Ord> PriorityQueue<T> {
    pub fn new() -> Self {
        PriorityQueue {
            inner: std::collections::BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, item: T) {
        self.inner.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.inner.peek()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Iterate in arbitrary heap order (NOT sorted order).
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }
}

impl<T: Ord> Default for PriorityQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SortedMap<K, V> / SortedSet<T>
// ---------------------------------------------------------------------------

/// Sorted map (`BTreeMap` wrapper).
///
/// Complexity: `insert`/`get`/`remove` O(log n), `len` O(1),
/// range iteration O(log n + k).
/// Ownership: owns keys and values; `get` borrows, `remove` moves out.
/// Iterator-invalidation: `BTreeMap` iterators stay valid across
/// `insert`/`remove` except for the removed element itself; still, prefer
/// short-lived borrows.
/// Thread-safety: `Send` iff `K, V: Send`; `!Sync` — single-owner.
/// Ordering: keys in ascending `Ord` order; iteration is sorted.
pub struct SortedMap<K: Ord, V> {
    inner: std::collections::BTreeMap<K, V>,
}

impl<K: Ord, V> SortedMap<K, V> {
    pub fn new() -> Self {
        SortedMap {
            inner: std::collections::BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.inner.get_mut(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.inner.iter()
    }

    pub fn range<R>(&self, range: R) -> impl Iterator<Item = (&K, &V)>
    where
        R: std::ops::RangeBounds<K>,
    {
        self.inner.range(range)
    }
}

impl<K: Ord, V> Default for SortedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

/// Sorted set (`BTreeSet` wrapper).
///
/// Complexity: `insert`/`contains`/`remove` O(log n), `len` O(1).
/// Ownership: owns elements; no value payload.
/// Iterator-invalidation: iterators stay valid across `insert`/`remove`
/// except for the removed element; prefer short-lived borrows.
/// Thread-safety: `Send` iff `T: Send`; `!Sync` — single-owner.
/// Ordering: ascending `Ord` order; iteration is sorted.
pub struct SortedSet<T: Ord> {
    inner: std::collections::BTreeSet<T>,
}

impl<T: Ord> SortedSet<T> {
    pub fn new() -> Self {
        SortedSet {
            inner: std::collections::BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> bool {
        self.inner.insert(value)
    }

    pub fn contains(&self, value: &T) -> bool {
        self.inner.contains(value)
    }

    pub fn remove(&mut self, value: &T) -> bool {
        self.inner.remove(value)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }
}

impl<T: Ord> Default for SortedSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ConcurrentMap
// ---------------------------------------------------------------------------

/// Concurrent hash map (`dashmap::DashMap` wrapper).
///
/// Complexity: `insert`/`get`/`remove` O(1) amortised under sharded locks.
/// Ownership: the map owns keys and values; `get` returns a cloned `V`
/// (no cross-call borrows, so no guard lifetimes leak).
/// Iterator-invalidation: this wrapper never hands out `Ref` guards —
/// snapshots are cloned — so there is nothing to invalidate; if you add
/// guard-returning APIs later, do not hold guards across writes (deadlock).
/// Thread-safety: `Send + Sync` when `K, V: Send + Sync`; safe to share
/// via `Arc`. Cloning the map is NOT supported (unlike [`MemoryCache`]);
/// wrap in `Arc` explicitly.
/// Ordering: NONE — sharded hash order, iteration order unspecified.
pub struct ConcurrentMap<K, V> {
    inner: dashmap::DashMap<K, V>,
}

impl<K, V> ConcurrentMap<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        ConcurrentMap {
            inner: dashmap::DashMap::new(),
        }
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        self.inner.get(key).map(|r| r.value().clone())
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.inner.remove(key).map(|(_, v)| v)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<K, V> Default for ConcurrentMap<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ConcurrentQueue
// ---------------------------------------------------------------------------

/// Multi-producer / multi-consumer FIFO queue with close semantics.
///
/// Backed by `parking_lot::Mutex<VecDeque<T>>`.
///
/// Complexity: `push`/`pop` O(1) amortised under a single mutex; `len`
/// O(1) (locks briefly).
/// Ownership: the queue owns buffered items; `pop` moves the front out;
/// a failed `push` after [`ConcurrentQueue::close`] returns the item back
/// as `Err(item)` so the caller retains ownership.
/// Iterator-invalidation: no iterators are exposed (use repeated `pop` to
/// drain); nothing to invalidate.
/// Thread-safety: `Send + Sync` when `T: Send`; share via `Arc`.
/// Ordering: strict FIFO among successful `push` calls (linearized by the
/// mutex). After `close()`: no new pushes accepted; buffered items still
/// drain via `pop` until empty.
pub struct ConcurrentQueue<T> {
    inner: parking_lot::Mutex<std::collections::VecDeque<T>>,
    closed: parking_lot::Mutex<bool>,
}

impl<T> ConcurrentQueue<T> {
    pub fn new() -> Self {
        ConcurrentQueue {
            inner: parking_lot::Mutex::new(std::collections::VecDeque::new()),
            closed: parking_lot::Mutex::new(false),
        }
    }

    /// Push `item` unless the queue is closed (then `Err(item)`).
    pub fn push(&self, item: T) -> Result<(), T> {
        if *self.closed.lock() {
            return Err(item);
        }
        self.inner.lock().push_back(item);
        Ok(())
    }

    /// Pop the front item, or `None` when empty (draining still works
    /// after close until the buffer is empty).
    pub fn pop(&self) -> Option<T> {
        self.inner.lock().pop_front()
    }

    /// Close the queue: future `push` calls fail; buffered items drain.
    pub fn close(&self) {
        *self.closed.lock() = true;
    }

    pub fn is_closed(&self) -> bool {
        *self.closed.lock()
    }

    pub fn len(&self) -> usize {
        self.inner.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().is_empty()
    }
}

impl<T> Default for ConcurrentQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_cache_roundtrip() {
        let c: MemoryCache<String, i32> = MemoryCache::new();
        assert!(c.is_empty());
        c.set("a".to_string(), 1);
        assert_eq!(c.get(&"a".to_string()), Some(1));
        assert_eq!(c.len(), 1);
        c.remove(&"a".to_string());
        assert!(c.is_empty());
        c.set("b".to_string(), 2);
        c.clear();
        assert!(c.is_empty());
    }

    #[test]
    fn ring_buffer_overwrites_oldest() {
        let mut r = RingBuffer::new(2);
        assert_eq!(r.push(1), None);
        assert_eq!(r.push(2), None);
        assert!(r.is_full());
        assert_eq!(r.push(3), Some(1));
        assert_eq!(r.pop(), Some(2));
        assert_eq!(r.pop(), Some(3));
        assert_eq!(r.pop(), None);
        let mut zero: RingBuffer<i32> = RingBuffer::new(0);
        assert_eq!(zero.push(9), Some(9));
        assert!(zero.is_empty());
    }

    #[test]
    fn bitset_set_get_count_ops() {
        let mut b = BitSet::new(130);
        assert_eq!(b.len(), 130);
        assert!(b.is_empty());
        assert!(b.set(0));
        assert!(b.set(129));
        assert_eq!(b.get(0), Some(true));
        assert_eq!(b.get(1), Some(false));
        assert_eq!(b.get(200), None);
        assert!(!b.set(200));
        assert_eq!(b.count_ones(), 2);
        let mut c = BitSet::new(130);
        c.set(1);
        c.set(129);
        b.union_with(&c);
        assert_eq!(b.count_ones(), 3);
        b.intersect_with(&c);
        assert_eq!(b.get(0), Some(false));
        assert_eq!(b.get(1), Some(true));
        assert!(b.clear(1));
        assert_eq!(b.get(1), Some(false));
    }

    #[test]
    fn stack_lifo() {
        let mut s = LexStack::new();
        assert!(s.is_empty());
        s.push(1);
        s.push(2);
        assert_eq!(s.peek(), Some(&2));
        assert_eq!(s.pop(), Some(2));
        assert_eq!(s.pop(), Some(1));
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn queue_fifo() {
        let mut q: LexQueue<i32> = LexQueue::new();
        q.enqueue(1);
        q.enqueue(2);
        assert_eq!(q.peek(), Some(&1));
        assert_eq!(q.dequeue(), Some(1));
        assert_eq!(q.dequeue(), Some(2));
        assert!(q.is_empty());
    }

    #[test]
    fn deque_both_ends() {
        let mut d = LexDeque::new();
        d.push_back(2);
        d.push_front(1);
        d.push_back(3);
        assert_eq!(d.get(0), Some(&1));
        assert_eq!(d.pop_front(), Some(1));
        assert_eq!(d.pop_back(), Some(3));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn priority_queue_max_first() {
        let mut p = PriorityQueue::new();
        p.push(1);
        p.push(5);
        p.push(3);
        assert_eq!(p.peek(), Some(&5));
        assert_eq!(p.pop(), Some(5));
        assert_eq!(p.pop(), Some(3));
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn sorted_map_sorted_order_and_range() {
        let mut m = SortedMap::new();
        m.insert(2, "b");
        m.insert(1, "a");
        m.insert(3, "c");
        let keys: Vec<i32> = m.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, vec![1, 2, 3]);
        assert_eq!(m.get(&2), Some(&"b"));
        assert!(m.contains_key(&1));
        let ranged: Vec<i32> = m.range(1..=2).map(|(k, _)| *k).collect();
        assert_eq!(ranged, vec![1, 2]);
        assert_eq!(m.remove(&2), Some("b"));
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn sorted_set_sorted_unique() {
        let mut s = SortedSet::new();
        assert!(s.insert(2));
        assert!(!s.insert(2));
        s.insert(1);
        s.insert(3);
        let vals: Vec<i32> = s.iter().cloned().collect();
        assert_eq!(vals, vec![1, 2, 3]);
        assert!(s.contains(&1));
        assert!(s.remove(&1));
        assert!(!s.contains(&1));
    }

    #[test]
    fn concurrent_map_shared() {
        use std::sync::Arc;
        let m: Arc<ConcurrentMap<String, i32>> = Arc::new(ConcurrentMap::new());
        m.insert("k".to_string(), 7);
        assert_eq!(m.get(&"k".to_string()), Some(7));
        assert!(m.contains_key(&"k".to_string()));
        let m2 = Arc::clone(&m);
        std::thread::scope(|s| {
            s.spawn(move || {
                m2.insert("t".to_string(), 1);
            });
        });
        assert_eq!(m.get(&"t".to_string()), Some(1));
        assert_eq!(m.len(), 2);
        assert_eq!(m.remove(&"k".to_string()), Some(7));
        assert!(!m.is_empty());
    }

    #[test]
    fn concurrent_queue_close_semantics() {
        use std::sync::Arc;
        let q: Arc<ConcurrentQueue<i32>> = Arc::new(ConcurrentQueue::new());
        q.push(1).unwrap();
        q.push(2).unwrap();
        assert_eq!(q.len(), 2);
        assert_eq!(q.pop(), Some(1));
        q.close();
        assert!(q.is_closed());
        assert!(q.push(3).is_err());
        assert_eq!(q.pop(), Some(2));
        assert_eq!(q.pop(), None);
        assert!(q.is_empty());
    }
}
