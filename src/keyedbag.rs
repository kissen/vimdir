use std::clone::Clone;
use std::cmp::Eq;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::option::Option;

/// A HashMap where each key is associated with a set of values.
/// Inserting different values for the same keys associates each
/// unique value with the given key.
pub struct KeyedBag<K: Eq + Hash + Clone, V: Eq + Hash + Clone> {
    map: HashMap<K, HashSet<V>>,
}

impl<K: Eq + Hash + Clone, V: Eq + Hash + Clone> KeyedBag<K, V> {
    /// Create a new, empty KeyedBag.
    pub fn new() -> KeyedBag<K, V> {
        KeyedBag {
            map: HashMap::new(),
        }
    }

    /// Insert value associated with key. Both key and value are
    /// copied.
    pub fn insert(&mut self, key: &K, value: &V) {
        let key = key.clone();
        let value = value.clone();
        self.map.entry(key).or_default().insert(value);
    }

    /// Return a copy of the set of values associated with key.
    pub fn get(&self, key: &K) -> Option<HashSet<V>> {
        Some(self.map.get(key)?.clone())
    }

    /// Return a copy of all keys in this KeyedBag.
    pub fn keys(&self) -> HashSet<K> {
        self.map.keys().map(|e| e.clone()).collect()
    }
}
