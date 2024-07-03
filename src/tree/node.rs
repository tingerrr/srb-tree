use std::array;
use std::fmt::Debug;

use crate::key::Key;

pub mod iter;

pub use iter::Keys;
pub use iter::Pairs;
pub use iter::Values;

fn get_non_empty<K, V, const B: usize, const FIRST_LAST: bool>(
    children: &[Option<Node<K, V, B>>],
) -> Option<&Node<K, V, B>> {
    let mut children = children.iter().filter_map(Option::as_ref);

    if FIRST_LAST {
        children.find(|n| n.len != 0)
    } else {
        children.rfind(|n| n.len != 0)
    }
}

fn get_non_empty_mut<K, V, const B: usize, const FIRST_LAST: bool>(
    children: &mut [Option<Node<K, V, B>>],
) -> Option<&mut Node<K, V, B>> {
    let mut children = children.iter_mut().filter_map(Option::as_mut);

    if FIRST_LAST {
        children.find(|n| n.len != 0)
    } else {
        children.rfind(|n| n.len != 0)
    }
}

pub(super) enum Repr<K, V, const B: usize> {
    Internal {
        children: Box<[Option<Node<K, V, B>>; B]>,
    },
    Leaf {
        keys: Box<[Option<K>; B]>,
        values: Box<[Option<V>; B]>,
    },
}

pub(super) struct Node<K, V, const B: usize> {
    pub(super) repr: Repr<K, V, B>,
    pub(super) len: usize,
}

impl<K, V, const B: usize> Node<K, V, B> {
    pub fn new_internal() -> Self {
        Self {
            repr: Repr::Internal {
                children: Box::new(array::from_fn(|_| None)),
            },
            len: 0,
        }
    }

    pub fn new_leaf() -> Self {
        Self {
            repr: Repr::Leaf {
                keys: Box::new(array::from_fn(|_| None)),
                values: Box::new(array::from_fn(|_| None)),
            },
            len: 0,
        }
    }

    pub fn replace_child_at(&mut self, idx: usize, child: Option<Self>) -> Option<Self> {
        match &mut self.repr {
            Repr::Internal { children } => {
                let new_len = child.as_ref().map(|child| child.len).unwrap_or_default();
                let old = std::mem::replace(&mut children[idx], child);
                let old_len = old.as_ref().map(|child| child.len).unwrap_or_default();

                self.len = self.len - old_len + new_len;

                old
            }
            Repr::Leaf { .. } => panic!("can't swap child into leaf node"),
        }
    }

    pub fn replace_key_value_at(
        &mut self,
        idx: usize,
        key_value: Option<(K, V)>,
    ) -> Option<(K, V)> {
        match &mut self.repr {
            Repr::Internal { .. } => panic!("can't insert key value into internal node"),
            Repr::Leaf { keys, values } => {
                let new_some = key_value.is_some();

                let (key, value) = Option::unzip(key_value);
                let old = Option::zip(
                    std::mem::replace(&mut keys[idx], key),
                    std::mem::replace(&mut values[idx], value),
                );

                let old_some = old.is_some();

                match (old_some, new_some) {
                    (true, false) => self.len -= 1,
                    (false, true) => self.len += 1,
                    _ => {}
                }

                old
            }
        }
    }

    pub fn storage_bytes(&self) -> usize {
        std::mem::size_of_val(&self)
            + match &self.repr {
                Repr::Internal { children } => children
                    .iter()
                    .filter_map(Option::as_ref)
                    .map(Node::storage_bytes)
                    .sum::<usize>(),
                Repr::Leaf { .. } => 0,
            }
    }

    pub fn storage_util(&self) -> (usize, usize) {
        match &self.repr {
            Repr::Internal { children } => children
                .iter()
                .filter_map(Option::as_ref)
                .map(Node::storage_util)
                .reduce(|acc, it| (acc.0 + it.0, acc.1 + it.1))
                .unwrap_or_default(),
            Repr::Leaf { keys, .. } => (keys.len(), self.len),
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
}

impl<K: Key, V, const B: usize> Node<K, V, B> {
    fn assert_depth(&self, depth: usize) {
        match &self.repr {
            Repr::Internal { .. } => {
                debug_assert_ne!(
                    depth,
                    K::max_depth(B),
                    "internal nodes can only be below max depth"
                );
            }
            Repr::Leaf { .. } => {
                debug_assert_eq!(depth, K::max_depth(B), "leafs can only be at max depth");
            }
        }
    }

    pub fn insert(&mut self, depth: usize, key: K, value: V) -> Option<(K, V)> {
        self.assert_depth(depth);

        match &mut self.repr {
            Repr::Internal { children } => match &mut children[key.index_at(B, depth)] {
                Some(child) => {
                    self.len += 1;
                    child.insert(depth + 1, key, value)
                }
                child @ None => {
                    let mut new = if depth + 1 == K::max_depth(B) {
                        Self::new_leaf()
                    } else {
                        Self::new_internal()
                    };

                    new.insert(depth + 1, key, value);
                    *child = Some(new);
                    self.len += 1;
                    None
                }
            },
            Repr::Leaf { .. } => {
                self.replace_key_value_at(key.index_at(B, depth), Some((key, value)))
            }
        }
    }

    pub fn remove(&mut self, depth: usize, key: &K) -> Option<(K, V)> {
        self.assert_depth(depth);

        let idx = key.index_at(B, depth);
        match &mut self.repr {
            Repr::Internal { children } => match &mut children[idx] {
                Some(child) => {
                    let old_len = child.len;
                    let old = child.remove(depth + 1, key);
                    let new_len = child.len;

                    if new_len == 0 {
                        // TODO: i think this may actually increase the complexity by insane an measure
                        children[idx] = None;
                    } else if old_len != new_len {
                        self.len -= 1;
                    }

                    old
                }
                None => None,
            },
            Repr::Leaf { .. } => self.replace_key_value_at(idx, None),
        }
    }

    pub fn get(&self, depth: usize, key: &K) -> Option<(&K, &V)> {
        self.assert_depth(depth);

        let idx = key.index_at(B, depth);
        match &self.repr {
            Repr::Internal { children } => children
                .get(idx)
                .and_then(Option::as_ref)
                .and_then(|child| child.get(depth + 1, key)),
            Repr::Leaf { keys, values } => Option::zip(
                keys.get(idx).and_then(Option::as_ref),
                values.get(idx).and_then(Option::as_ref),
            ),
        }
    }

    pub fn get_mut(&mut self, depth: usize, key: &K) -> Option<(&mut K, &mut V)> {
        self.assert_depth(depth);

        let idx = key.index_at(B, depth);
        match &mut self.repr {
            Repr::Internal { children } => children
                .get_mut(idx)
                .and_then(Option::as_mut)
                .and_then(|child| child.get_mut(depth + 1, key)),
            Repr::Leaf { keys, values } => Option::zip(
                keys.get_mut(idx).and_then(Option::as_mut),
                values.get_mut(idx).and_then(Option::as_mut),
            ),
        }
    }

    pub fn first(&self) -> Option<(&K, &V)> {
        match &self.repr {
            Repr::Internal { children } => {
                get_non_empty::<K, V, B, true>(&**children).and_then(Node::first)
            }
            // TODO: double iteration
            Repr::Leaf { keys, values } => Option::zip(
                keys.iter().filter_map(Option::as_ref).next(),
                values.iter().filter_map(Option::as_ref).next(),
            ),
        }
    }

    pub fn first_mut(&mut self) -> Option<(&mut K, &mut V)> {
        match &mut self.repr {
            Repr::Internal { children } => {
                get_non_empty_mut::<K, V, B, true>(&mut **children).and_then(Node::first_mut)
            }
            // TODO: see above
            Repr::Leaf { keys, values } => Option::zip(
                keys.iter_mut().filter_map(Option::as_mut).next(),
                values.iter_mut().filter_map(Option::as_mut).next(),
            ),
        }
    }

    pub fn last(&self) -> Option<(&K, &V)> {
        match &self.repr {
            Repr::Internal { children } => {
                get_non_empty::<K, V, B, false>(&**children).and_then(Node::last)
            }
            // TODO: see above
            Repr::Leaf { keys, values } => Option::zip(
                keys.iter().rev().filter_map(Option::as_ref).next(),
                values.iter().rev().filter_map(Option::as_ref).next(),
            ),
        }
    }

    pub fn last_mut(&mut self) -> Option<(&mut K, &mut V)> {
        match &mut self.repr {
            Repr::Internal { children } => {
                get_non_empty_mut::<K, V, B, false>(&mut **children).and_then(Node::last_mut)
            }
            // TODO: see above
            Repr::Leaf { keys, values } => Option::zip(
                keys.iter_mut().rev().flat_map(Option::as_mut).next(),
                values.iter_mut().rev().flat_map(Option::as_mut).next(),
            ),
        }
    }
}

impl<K: Debug, V: Debug, const B: usize> Debug for Node<K, V, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        self.pairs().for_each(|(k, v)| {
            map.entry(k, v);
        });
        map.finish()
    }
}
