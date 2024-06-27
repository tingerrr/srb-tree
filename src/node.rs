use std::array;
use std::fmt::Debug;

use crate::key::Key;

#[derive(Debug)]
pub enum Node<K, V = (), const B: usize = 32> {
    Internal(Box<[Option<Self>; B]>),
    Leaf(K, V),
}

impl<K, V, const B: usize> Node<K, V, B> {
    pub fn new_internal() -> Self {
        Self::Internal(Box::new(array::from_fn(|_| None)))
    }

    fn insert_child_at(&mut self, idx: usize, child: Self) {
        match self {
            Node::Internal(children) => {
                children[idx] = Some(child);
            }
            Node::Leaf(_, _) => panic!("can't insert child into leaf node"),
        }
    }

    fn remove_child_at(&mut self, idx: usize) -> Option<Self> {
        match self {
            Node::Internal(children) => children.get_mut(idx).map(Option::take).flatten(),
            Node::Leaf(_, _) => panic!("can't insert child into leaf node"),
        }
    }

    fn children(&self) -> Option<impl DoubleEndedIterator<Item = &Node<K, V, B>>> {
        match self {
            Node::Internal(children) => Some(children.iter().filter_map(Option::as_ref)),
            Node::Leaf(_, _) => None,
        }
    }

    fn children_mut(&mut self) -> Option<impl DoubleEndedIterator<Item = &mut Node<K, V, B>>> {
        match self {
            Node::Internal(children) => Some(children.iter_mut().filter_map(Option::as_mut)),
            Node::Leaf(_, _) => None,
        }
    }

    fn keys(&self) -> impl DoubleEndedIterator<Item = &K> {
        match self {
            Node::Internal(children) => Box::new(
                children
                    .iter()
                    .filter_map(Option::as_ref)
                    .flat_map(Node::keys),
            ) as Box<dyn DoubleEndedIterator<Item = &K>>,
            Node::Leaf(key, _) => Box::new(std::iter::once(key)),
        }
    }

    fn keys_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut K> {
        match self {
            Node::Internal(children) => Box::new(
                children
                    .iter_mut()
                    .filter_map(Option::as_mut)
                    .flat_map(Node::keys_mut),
            )
                as Box<dyn DoubleEndedIterator<Item = &mut K>>,
            Node::Leaf(key, _) => Box::new(std::iter::once(key)),
        }
    }

    fn values(&self) -> impl DoubleEndedIterator<Item = &V> {
        match self {
            Node::Internal(children) => Box::new(
                children
                    .iter()
                    .filter_map(Option::as_ref)
                    .flat_map(Node::values),
            ) as Box<dyn DoubleEndedIterator<Item = &V>>,
            Node::Leaf(_, value) => Box::new(std::iter::once(value)),
        }
    }

    fn values_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut V> {
        match self {
            Node::Internal(children) => Box::new(
                children
                    .iter_mut()
                    .filter_map(Option::as_mut)
                    .flat_map(Node::values_mut),
            )
                as Box<dyn DoubleEndedIterator<Item = &mut V>>,
            Node::Leaf(_, value) => Box::new(std::iter::once(value)),
        }
    }

    fn pairs(&self) -> impl DoubleEndedIterator<Item = (&K, &V)> {
        match self {
            Node::Internal(children) => Box::new(
                children
                    .iter()
                    .filter_map(Option::as_ref)
                    .flat_map(Node::pairs),
            )
                as Box<dyn DoubleEndedIterator<Item = (&K, &V)>>,
            Node::Leaf(key, value) => Box::new(std::iter::once((key, value))),
        }
    }

    fn pairs_mut(&mut self) -> impl DoubleEndedIterator<Item = (&K, &mut V)> {
        match self {
            Node::Internal(children) => Box::new(
                children
                    .iter_mut()
                    .filter_map(Option::as_mut)
                    .flat_map(Node::pairs_mut),
            )
                as Box<dyn DoubleEndedIterator<Item = (&K, &mut V)>>,
            Node::Leaf(key, value) => Box::new(std::iter::once((&*key, value))),
        }
    }
}

impl<K: Key, V, const B: usize> Node<K, V, B> {
    pub fn insert(&mut self, depth: usize, key: K, value: V) -> Option<(K, V)> {
        match self {
            Node::Internal(children) => {
                debug_assert_ne!(
                    depth,
                    K::max_depth(B),
                    "internal nodes can only be below max depth"
                );

                let child = &mut children[key.index_at(B, depth)];
                match child {
                    Some(child) => child.insert(depth + 1, key, value),
                    None => {
                        let new = if depth + 1 == K::max_depth(B) {
                            Self::Leaf(key, value)
                        } else {
                            Self::new_internal()
                        };

                        *child = Some(new);
                        None
                    }
                }
            }
            Node::Leaf(k, v) => {
                debug_assert_eq!(depth, K::max_depth(B), "leafs can only be at max depth");
                Some((std::mem::replace(k, key), std::mem::replace(v, value)))
            }
        }
    }
}

// impl<K: Debug, V: Debug, const B: usize> Debug for Node<K, V, B> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Node::Internal(children) => {
//                 let mut set = f.debug_set();

//                 for c in children.iter().flat_map(Option::as_ref) {
//                     set.entry(c);
//                 }

//                 set.finish()
//             }
//             Node::Leaf(k, v) => f.debug_tuple("").field(k).field(v).finish(),
//         }
//     }
// }
