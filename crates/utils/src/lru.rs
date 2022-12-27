#![allow(unused)]

use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Debug)]
struct Item<K, V> {
    value: V,
    newer: Option<K>,
    older: Option<K>,
}

/// A LRU cache.
#[derive(Clone, Debug)]
pub struct Lru<K, V>
where
    K: Hash + Eq + Clone,
{
    items: HashMap<K, Item<K, V>>,
    newest: Option<K>,
    oldest: Option<K>,
    capacity: usize,
}

impl<K, V> Lru<K, V>
where
    K: Hash + Eq + Clone,
{
    /// Creates a new [`Lru`] maxed at `capacity` items.
    pub fn new(capacity: usize) -> Self {
        Self {
            items: HashMap::with_capacity(capacity),
            newest: None,
            oldest: None,
            capacity,
        }
    }

    /// Returns the number of items in this `Lru`.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns the maximum number of items allowed in this [`Lru`].
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns a reference to the value corresponding to the key,
    /// which becomes the newest.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let (k, v) = self.remove(key)?;
        self.set(k, v);

        self.items.get(&key).map(|item| &item.value)
    }

    /// Inserts a key-value pair, which becomes the newest.
    /// The oldest value might be removed.
    pub fn set(&mut self, key: K, value: V) {
        // Remove oldest if full
        if self.items.len() == self.capacity {
            if let Some(oldest) = self.oldest.clone() {
                self.remove(&oldest);
            }
        }

        // Update newest item
        if let Some(newest) = self.newest.as_ref() {
            let newest = self.items.get_mut(newest).expect("newest");
            debug_assert!(newest.newer.is_none());
            newest.newer = Some(key.clone());
        }

        // Update oldest key
        if self.oldest.is_none() {
            debug_assert!(self.newest.is_none());
            self.oldest = Some(key.clone());
        }

        // Update newest key
        let older = std::mem::replace(&mut self.newest, Some(key.clone()));

        // Insert the new item
        self.items.insert(
            key,
            Item {
                value,
                newer: None,
                older,
            },
        );
    }

    /// Inserts the value if not present and returns it, which becomes the newest.
    /// The oldest value might be removed.
    pub fn get_or_set<F>(&mut self, key: K, f: F) -> &V
    where
        F: FnOnce() -> V,
    {
        if self.get(&key).is_none() {
            self.set(key.clone(), f());
        }

        self.get(&key).expect("item")
    }

    /// Removes a value from this [`Lru`].
    pub fn remove(&mut self, key: &K) -> Option<(K, V)> {
        let (key, item) = self.items.remove_entry(key)?;

        match (item.newer, item.older) {
            // -> item <-
            (None, None) => {
                debug_assert!(self.items.len() == 0);
                debug_assert!(self.newest.as_ref() == Some(&key));
                debug_assert!(self.oldest.as_ref() == Some(&key));

                self.newest = None;
                self.oldest = None;

                return Some((key, item.value));
            }
            // -> item <-> older <-> ... <-> oldest <-
            (None, Some(older)) => {
                debug_assert!(self.newest.as_ref() == Some(&key));

                self.items.get_mut(&older).expect("older").newer = None;
                self.newest = Some(older);

                return Some((key, item.value));
            }
            // <-> newest <-> ... <-> newer <-> item <-
            (Some(newer), None) => {
                debug_assert!(self.oldest.as_ref() == Some(&key));

                self.items.get_mut(&newer).expect("newer").older = None;
                self.oldest = Some(newer);

                return Some((key, item.value));
            }
            // <-> newest <-> ... <-> newer <-> item <-> older <-> ... <-> oldest <-
            (Some(newer), Some(older)) => {
                self.items.get_mut(&newer).expect("newer").older = Some(older.clone());
                self.items.get_mut(&older).expect("older").newer = Some(newer);

                return Some((key, item.value));
            }
        }

        None
    }

    /// Clears this [`Lru`].
    pub fn clear(&mut self) {
        self.items = Default::default();
        self.newest = None;
        self.oldest = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut lru = Lru::new(10);
        assert!(lru.len() == 0);
        assert!(lru.newest == None);
        assert!(lru.oldest == None);

        lru.set("a", "a");
        assert!(lru.len() == 1);
        assert!(lru.newest == Some("a"));
        assert!(lru.oldest == Some("a"));
        assert!(lru.get(&"a") == Some(&"a"));

        lru.set("b", "b");
        assert!(lru.len() == 2);
        assert!(lru.newest == Some("b"));
        assert!(lru.oldest == Some("a"));
        assert!(lru.get(&"b") == Some(&"b"));

        {
            let mut lru = lru.clone();

            assert!(lru.remove(&"b") == Some(("b", "b")));
            assert!(lru.len() == 1);
            assert!(lru.newest == Some("a"));
            assert!(lru.oldest == Some("a"));
            assert!(lru.get(&"b") == None);
        }
        {
            let mut lru = lru.clone();

            assert!(lru.remove(&"a") == Some(("a", "a")));
            assert!(lru.len() == 1);
            assert!(lru.newest == Some("b"));
            assert!(lru.oldest == Some("b"));
            assert!(lru.get(&"a") == None);
        }
        {
            let mut lru = lru.clone();

            lru.remove(&"a");
            lru.remove(&"b");
            assert!(lru.len() == 0);
            assert!(lru.newest == None);
            assert!(lru.oldest == None);
            assert!(lru.get(&"a") == None);
            assert!(lru.get(&"b") == None);
        }

        lru.clear();
        assert!(lru.len() == 0);
        assert!(lru.newest == None);
        assert!(lru.oldest == None);
        assert!(lru.get(&"a") == None);
        assert!(lru.get(&"b") == None);
    }
}
