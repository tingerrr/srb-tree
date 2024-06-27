use std::{borrow::Borrow, fmt::Debug};

use crate::key::Key;

use super::node::Node;
use super::B;

pub struct SrbTreeMap<K, V> {
    root: Node<K, V, B>,
}

impl<K, V> SrbTreeMap<K, V> {
    pub fn new() -> Self {
        Self {
            root: Node::new_internal(),
        }
    }
}

impl<K: Key, V> SrbTreeMap<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        self.root.insert(0, key, value)
    }

    pub fn remove<Q: Borrow<K>>(&mut self, key: &Q) -> Option<(K, V)> {
        self.root.remove(0, key.borrow())
    }

    pub fn get<Q: Borrow<K>>(&self, key: &Q) -> Option<(&K, &V)> {
        self.root.get(0, key.borrow())
    }

    pub fn get_mut<Q: Borrow<K>>(&mut self, key: &Q) -> Option<(&K, &mut V)> {
        self.root.get_mut(0, key.borrow()).map(|(k, v)| (&*k, v))
    }
}

impl<K: Debug, V: Debug> Debug for SrbTreeMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        self.root.traverse(|k, v| {
            map.entry(k, v);
        });
        map.finish()
    }
}
