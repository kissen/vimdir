use std::clone::Clone;
use std::cmp::Eq;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::option::Option;

pub struct KeyedBag<K: Eq + Hash + Clone, V: Eq + Hash + Clone> {
    map: HashMap<K, HashSet<V>>,
}

impl<K: Eq + Hash + Clone, V: Eq + Hash + Clone> KeyedBag<K, V> {
    pub fn new() -> KeyedBag<K, V> {
        KeyedBag {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &K, value: &V) {
        let key = key.clone();
        let value = value.clone();
        self.map.entry(key).or_default().insert(value);
    }

    pub fn get(&self, key: &K) -> Option<HashSet<V>> {
        Some(self.map.get(key)?.clone())
    }

    pub fn keys(&self) -> HashSet<K> {
        self.map.keys().map(|e| e.clone()).collect()
    }
}
