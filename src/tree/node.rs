use std::array;
use std::fmt::Debug;

use crate::key::Key;

pub(super) enum Node<K, V = (), const B: usize = 32> {
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

    pub fn children(&self) -> Option<impl DoubleEndedIterator<Item = &Node<K, V, B>>> {
        match self {
            Node::Internal { children } => Some(children.iter().filter_map(Option::as_ref)),
            Node::Leaf { keys: _, values: _ } => None,
        }
    }

    pub fn children_mut(&mut self) -> Option<impl DoubleEndedIterator<Item = &mut Node<K, V, B>>> {
        match self {
            Node::Internal { children } => Some(children.iter_mut().filter_map(Option::as_mut)),
            Node::Leaf { keys: _, values: _ } => None,
        }
    }

    pub fn keys(&self) -> impl DoubleEndedIterator<Item = &K> {
        match self {
            Node::Internal { children } => Box::new(
                children
                    .iter()
                    .filter_map(Option::as_ref)
                    .flat_map(Node::keys),
            ) as Box<dyn DoubleEndedIterator<Item = &K>>,
            Node::Leaf { keys, .. } => Box::new(keys.iter().flat_map(Option::as_ref)),
        }
    }

    pub fn keys_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut K> {
        match self {
            Node::Internal { children } => Box::new(
                children
                    .iter_mut()
                    .filter_map(Option::as_mut)
                    .flat_map(Node::keys_mut),
            )
                as Box<dyn DoubleEndedIterator<Item = &mut K>>,
            Node::Leaf { keys, .. } => Box::new(keys.iter_mut().flat_map(Option::as_mut)),
        }
    }

    pub fn values(&self) -> impl DoubleEndedIterator<Item = &V> {
        match self {
            Node::Internal { children } => Box::new(
                children
                    .iter()
                    .filter_map(Option::as_ref)
                    .flat_map(Node::values),
            ) as Box<dyn DoubleEndedIterator<Item = &V>>,
            Node::Leaf { values, .. } => Box::new(values.iter().flat_map(Option::as_ref)),
        }
    }

    pub fn values_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut V> {
        match self {
            Node::Internal { children } => Box::new(
                children
                    .iter_mut()
                    .filter_map(Option::as_mut)
                    .flat_map(Node::values_mut),
            )
                as Box<dyn DoubleEndedIterator<Item = &mut V>>,
            Node::Leaf { values, .. } => Box::new(values.iter_mut().flat_map(Option::as_mut)),
        }
    }

    pub fn pairs(&self) -> impl DoubleEndedIterator<Item = (&K, &V)> {
        match self {
            Node::Internal { children } => Box::new(
                children
                    .iter()
                    .filter_map(Option::as_ref)
                    .flat_map(Node::pairs),
            )
                as Box<dyn DoubleEndedIterator<Item = (&K, &V)>>,
            Node::Leaf { keys, values } => Box::new(Iterator::zip(
                keys.iter().flat_map(Option::as_ref),
                values.iter().flat_map(Option::as_ref),
            )),
        }
    }

    // pub fn pairs_mut(&mut self) -> impl DoubleEndedIterator<Item = (&mut K, &mut V)> {
    //     match self {
    //         Node::Internal { children } => Box::new(
    //             children
    //                 .iter_mut()
    //                 .filter_map(Option::as_mut)
    //                 .flat_map(Node::pairs_mut),
    //         )
    //             as Box<dyn DoubleEndedIterator<Item = (&mut K, &mut V)>>,
    //         Node::Leaf { keys, values } => Box::new(Iterator::zip(
    //             keys.iter_mut().flat_map(Option::as_mut),
    //             values.iter_mut().flat_map(Option::as_mut),
    //         )),
    //     }
    // }

    pub fn traverse<F>(&self, mut f: F)
    where
        F: FnMut(&K, &V),
    {
        // self.pairs().for_each(|(k, v)| f(k, v))
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

        match self {
            Node::Internal { children } => {
                //
                todo!()
            }
            Node::Leaf { .. } => self.replace_key_value_at(key.index_at(B, depth), None),
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

enum NodeIter<'n, K, V, const B: usize> {
    Internal(std::slice::Iter<'n, Node<K, V, B>>),
    Leaf(std::slice::Iter<'n, (K, V)>),
}

struct Keys<'n, K, V, const B: usize> {
    path: Vec<NodeIter<'n, K, V, B>>,
    current: Option<std::slice::Iter<'n, (K, V)>>,
}

impl<'n, K, V, const B: usize> Iterator for Keys<'n, K, V, B> {
    type Item = &'n K;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.as_mut()?
    }
}
