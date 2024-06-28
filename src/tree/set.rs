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

    pub fn storage_bytes(&self) -> usize {
        self.root.storage_bytes()
    }

    pub fn storage(&self) -> f32 {
        let (total, used) = self.root.storage_util();
        used as f32 / total as f32
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

    pub fn first(&self) -> Option<&T> {
        self.root.first().map(|(v, _)| v)
    }

    pub fn last(&self) -> Option<&T> {
        self.root.last().map(|(v, _)| v)
    }
}

impl<T: Debug> Debug for SrbTreeSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut set = f.debug_set();
        self.root.pairs().for_each(|(v, _)| {
            set.entry(v);
        });
        set.finish()
    }
}

impl<T: Key> Extend<T> for SrbTreeSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.insert(item);
        }
    }
}

impl<T: Key> FromIterator<T> for SrbTreeSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut this = Self::new();
        this.extend(iter);
        this
    }
}
