use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

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
