use std::array;
use std::fmt::Debug;

use crate::key::Key;

mod iter;

pub use iter::Keys;
pub use iter::Pairs;
pub use iter::Values;

pub(super) enum Node<K, V, const B: usize> {
    Internal {
        children: Box<[Option<Self>; B]>,
    },
    Leaf {
        keys: Box<[Option<K>; B]>,
        values: Box<[Option<V>; B]>,
    },
}

impl<K, V, const B: usize> Node<K, V, B> {
    pub fn new_internal() -> Self {
        Self::Internal {
            children: Box::new(array::from_fn(|_| None)),
        }
    }

    pub fn new_leaf() -> Self {
        Self::Leaf {
            keys: Box::new(array::from_fn(|_| None)),
            values: Box::new(array::from_fn(|_| None)),
        }
    }

    pub fn replace_child_at(&mut self, idx: usize, child: Option<Self>) -> Option<Self> {
        match self {
            Node::Internal { children } => std::mem::replace(&mut children[idx], child),
            Node::Leaf { .. } => panic!("can't swap child into leaf node"),
        }
    }

    pub fn replace_key_value_at(
        &mut self,
        idx: usize,
        key_value: Option<(K, V)>,
    ) -> Option<(K, V)> {
        match self {
            Node::Internal { .. } => panic!("can't insert key value into internal node"),
            Node::Leaf { keys, values } => {
                let (key, value) = Option::unzip(key_value);
                Option::zip(
                    std::mem::replace(&mut keys[idx], key),
                    std::mem::replace(&mut values[idx], value),
                )
            }
        }
    }
}

impl<K, V, const B: usize> Node<K, V, B> {
    pub fn keys(&self) -> Keys<'_, K, V, B> {
        Keys {
            inner: iter::Iter::new(self),
        }
    }

    pub fn values(&self) -> Values<'_, K, V, B> {
        Values {
            inner: iter::Iter::new(self),
        }
    }

    pub fn pairs(&self) -> Pairs<'_, K, V, B> {
        Pairs {
            inner: iter::Iter::new(self),
        }
    }

    pub fn traverse<F>(&self, mut f: F)
    where
        F: FnMut(&K, &V),
    {
        self.pairs().for_each(|(k, v)| f(k, v))
    }
}

impl<K: Key, V, const B: usize> Node<K, V, B> {
    fn assert_depth(&self, depth: usize) {
        match self {
            Node::Internal { .. } => {
                debug_assert_ne!(
                    depth,
                    K::max_depth(B),
                    "internal nodes can only be below max depth"
                );
            }
            Node::Leaf { .. } => {
                debug_assert_eq!(depth, K::max_depth(B), "leafs can only be at max depth");
            }
        }
    }

    pub fn insert(&mut self, depth: usize, key: K, value: V) -> Option<(K, V)> {
        self.assert_depth(depth);

        match self {
            Node::Internal { children } => match &mut children[key.index_at(B, depth)] {
                Some(child) => child.insert(depth + 1, key, value),
                child @ None => {
                    let mut new = if depth + 1 == K::max_depth(B) {
                        Self::new_leaf()
                    } else {
                        Self::new_internal()
                    };

                    new.insert(depth + 1, key, value);
                    *child = Some(new);
                    None
                }
            },
            Node::Leaf { .. } => {
                self.replace_key_value_at(key.index_at(B, depth), Some((key, value)))
            }
        }
    }

    pub fn remove(&mut self, depth: usize, key: &K) -> Option<(K, V)> {
        self.assert_depth(depth);

        let idx = key.index_at(B, depth);
        match self {
            Node::Internal { children } => {
                match children.get_mut(idx).map(Option::as_mut).flatten() {
                    Some(child) => child.remove(depth + 1, key),
                    None => None,
                }
            }
            Node::Leaf { .. } => self.replace_key_value_at(idx, None),
        }
    }

    pub fn get(&self, depth: usize, key: &K) -> Option<(&K, &V)> {
        self.assert_depth(depth);

        let idx = key.index_at(B, depth);
        match self {
            Node::Internal { children } => children
                .get(idx)
                .and_then(Option::as_ref)
                .and_then(|child| child.get(depth + 1, key)),
            Node::Leaf { keys, values } => Option::zip(
                keys.get(idx).and_then(Option::as_ref),
                values.get(idx).and_then(Option::as_ref),
            ),
        }
    }

    pub fn get_mut(&mut self, depth: usize, key: &K) -> Option<(&mut K, &mut V)> {
        self.assert_depth(depth);

        let idx = key.index_at(B, depth);
        match self {
            Node::Internal { children } => children
                .get_mut(idx)
                .and_then(Option::as_mut)
                .and_then(|child| child.get_mut(depth + 1, key)),
            Node::Leaf { keys, values } => Option::zip(
                keys.get_mut(idx).and_then(Option::as_mut),
                values.get_mut(idx).and_then(Option::as_mut),
            ),
        }
    }
}

impl<K: Debug, V: Debug, const B: usize> Debug for Node<K, V, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        self.traverse(|k, v| {
            map.entry(k, v);
        });
        map.finish()
    }
}
