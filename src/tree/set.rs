use std::{borrow::Borrow, fmt::Debug};

use crate::key::Key;

use super::node::Node;
use super::B;

pub struct SrbTreeSet<V> {
    root: Node<V, (), B>,
}

impl<T> SrbTreeSet<T> {
    pub fn new() -> Self {
        Self {
            root: Node::new_internal(),
        }
    }
}

impl<T: Key> SrbTreeSet<T> {
    pub fn insert(&mut self, value: T) -> Option<T> {
        self.root.insert(0, value, ()).map(|(v, _)| v)
    }

    pub fn remove<Q: Borrow<T>>(&mut self, value: &Q) -> Option<T> {
        self.root.remove(0, value.borrow()).map(|(v, _)| v)
    }

    pub fn get<Q: Borrow<T>>(&self, value: &Q) -> Option<&T> {
        self.root.get(0, value.borrow()).map(|(v, _)| v)
    }
}

impl<V: Debug> Debug for SrbTreeSet<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut set = f.debug_set();
        self.root.traverse(|k, _| {
            set.entry(k);
        });
        set.finish()
    }
}
