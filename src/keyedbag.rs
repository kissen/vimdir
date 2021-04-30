use std::clone::Clone;
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::vec::Vec;
use std::option::Option;

pub struct KeyedBag<K: Eq + Hash + Clone, V: Eq + Clone> {
    map: HashMap<K, Vec<V>>,
}

impl<K: Eq + Hash + Clone, V: Eq + Clone> KeyedBag<K, V> {
    pub fn new() -> KeyedBag<K, V> {
        KeyedBag {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &K, value: &V) {
        let key = key.clone();
        let value = value.clone();
        self.map.entry(key).or_default().push(value);
    }

    pub fn get(&self, key: &K) -> Option<Vec<V>> {
        match self.map.get(key) {
            Some(items) => Some(items.clone()),
            None => None
        }
    }

    pub fn keys(&self) -> Vec<K> {
        self.map.keys().map(|e| e.clone()).collect()
    }
}
