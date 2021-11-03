use std::cmp::Eq;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub trait Keyed = Eq + Hash;

/// A HashMap where each key is associated with a set of values.
/// Inserting different values for the same keys associates each
/// unique value with the given key.
pub struct KeyedBag<K: Keyed, V: Keyed> {
    /// The underlying container that matches key -> { value0, value1, ... }.
    container: HashMap<K, HashSet<V>>,

    /// Empty set for returns from method get. It is pretty silly that we have
    /// to create a new empty set for each keyed bag, but from what I can tell
    /// Rust does not let me create static variables that take generic
    /// parameters (i.e. HashSet<V> *inside* method get). So this is the best I
    /// can come up with right now.
    empty_set: HashSet<V>
}

impl<K: Keyed, V: Keyed> KeyedBag<K, V> {
    /// Create a new, empty KeyedBag.
    pub fn new() -> KeyedBag<K, V> {
        KeyedBag {
            container: HashMap::new(),
            empty_set: HashSet::new(),
        }
    }

    /// Insert value associated with key.
    pub fn insert(&mut self, key: K, value: V) {
        self.container.entry(key).or_default().insert(value);
    }

    /// Return the set of values associated with key. If no value was associated
    /// with the given key, this method returns an empty hash set.
    pub fn get(&self, key: &K) -> &HashSet<V> {
        match self.container.get(key) {
            None => &self.empty_set,
            Some(value) => value,
        }
    }

    /// Return a copy of all keys in this KeyedBag.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.container.keys()
    }
}

#[cfg(test)]
mod tests {
    use crate::KeyedBag;

    #[test]
    fn constructing_empty_bag() {
        let bag: KeyedBag<i32, i32> = KeyedBag::new();
        assert_eq!(bag.keys().len(), 0);
    }
}
