use super::{Node, Repr};

#[derive(Debug)]
struct InternalIter<'n, K, V, const B: usize> {
    children: &'n [Option<Node<K, V, B>>],
}

impl<'n, K, V, const B: usize> InternalIter<'n, K, V, B> {
    fn yield_left(&mut self) -> Option<&'n Node<K, V, B>> {
        loop {
            if let Some(child) = self.next()? {
                if child.len != 0 {
                    return Some(child);
                }
            }
        }
    }

    fn yield_right(&mut self) -> Option<&'n Node<K, V, B>> {
        loop {
            if let Some(child) = self.next_back()? {
                if child.len != 0 {
                    return Some(child);
                }
            }
        }
    }
}

impl<'n, K: 'n, V: 'n, const B: usize> Iterator for InternalIter<'n, K, V, B> {
    type Item = &'n Option<Node<K, V, B>>;

    fn next(&mut self) -> Option<Self::Item> {
        let (child, rest) = self.children.split_first()?;
        self.children = rest;

        Some(child)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.children.len(), Some(self.children.len()))
    }
}

impl<'n, K: 'n, V: 'n, const B: usize> DoubleEndedIterator for InternalIter<'n, K, V, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (child, rest) = self.children.split_last()?;
        self.children = rest;

        Some(child)
    }
}

impl<'n, K: 'n, V: 'n, const B: usize> ExactSizeIterator for InternalIter<'n, K, V, B> {}

#[derive(Debug)]
struct LeafIter<'n, K, V, const B: usize> {
    keys: &'n [Option<K>],
    values: &'n [Option<V>],
}

impl<'n, K, V, const B: usize> LeafIter<'n, K, V, B> {
    fn yield_left(&mut self) -> Option<(&'n K, &'n V)> {
        loop {
            if let (Some(k), Some(v)) = self.next()? {
                return Some((k, v));
            }
        }
    }

    fn yield_right(&mut self) -> Option<(&'n K, &'n V)> {
        loop {
            if let (Some(k), Some(v)) = self.next_back()? {
                return Some((k, v));
            }
        }
    }
}

impl<'n, K, V, const B: usize> Iterator for LeafIter<'n, K, V, B> {
    type Item = (&'n Option<K>, &'n Option<V>);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, krest) = self.keys.split_first()?;
        let (v, vrest) = self.values.split_first()?;

        self.keys = krest;
        self.values = vrest;

        Some((k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.keys.len(), Some(self.keys.len()))
    }
}

impl<'n, K, V, const B: usize> DoubleEndedIterator for LeafIter<'n, K, V, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (k, krest) = self.keys.split_last()?;
        let (v, vrest) = self.values.split_last()?;

        self.keys = krest;
        self.values = vrest;

        Some((k, v))
    }
}

impl<'n, K, V, const B: usize> ExactSizeIterator for LeafIter<'n, K, V, B> {}

#[derive(Debug)]
pub(super) struct Iter<'n, K, V, const B: usize> {
    path: Vec<InternalIter<'n, K, V, B>>,
    leaf: LeafIter<'n, K, V, B>,
}

impl<'n, K, V, const B: usize> Iter<'n, K, V, B> {
    pub fn new(node: &'n Node<K, V, B>) -> Self {
        match &node.repr {
            Repr::Internal { children } => Self {
                path: vec![InternalIter {
                    children: &**children,
                }],
                leaf: LeafIter {
                    keys: &[],
                    values: &[],
                },
            },
            Repr::Leaf { keys, values } => Self {
                path: vec![InternalIter { children: &[] }],
                leaf: LeafIter {
                    keys: &**keys,
                    values: &**values,
                },
            },
        }
    }

    fn descend_left(&mut self, child: &'n Node<K, V, B>) {
        match &child.repr {
            Repr::Internal { children } => {
                let mut current = InternalIter {
                    children: &**children,
                };

                let child = current.yield_left();
                self.path.push(current);
                match child {
                    Some(child) => self.descend_left(child),
                    // these stumps appear after removals without cleanup
                    None => {}
                }
            }
            Repr::Leaf { keys, values } => {
                self.leaf = LeafIter {
                    keys: &**keys,
                    values: &**values,
                };
            }
        }
    }

    fn next_leaf_left(&mut self) -> Option<()> {
        loop {
            let mut last = self.path.pop()?;
            if let Some(child) = last.yield_left() {
                self.path.push(last);
                self.descend_left(child);
                break Some(());
            }
        }
    }

    fn next_leaf_right(&mut self) -> Option<()> {
        todo!()
    }
}

impl<'n, K, V, const B: usize> Iterator for Iter<'n, K, V, B> {
    type Item = (&'n K, &'n V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(kv) = self.leaf.yield_left() {
                return Some(kv);
            }

            self.next_leaf_left()?;
        }
    }
}

impl<'n, K, V, const B: usize> DoubleEndedIterator for Iter<'n, K, V, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(kv) = self.leaf.yield_right() {
                return Some(kv);
            }

            self.next_leaf_right()?;
        }
    }
}

#[derive(Debug)]
pub struct Keys<'n, K, V, const B: usize> {
    pub(super) inner: Iter<'n, K, V, B>,
}

impl<'n, K, V, const B: usize> Iterator for Keys<'n, K, V, B> {
    type Item = &'n K;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, _)| k)
    }
}

impl<'n, K, V, const B: usize> DoubleEndedIterator for Keys<'n, K, V, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(k, _)| k)
    }
}

#[derive(Debug)]
pub struct Values<'n, K, V, const B: usize> {
    pub(super) inner: Iter<'n, K, V, B>,
}

impl<'n, K, V, const B: usize> Iterator for Values<'n, K, V, B> {
    type Item = &'n V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, v)| v)
    }
}

impl<'n, K, V, const B: usize> DoubleEndedIterator for Values<'n, K, V, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(_, v)| v)
    }
}

#[derive(Debug)]
pub struct Pairs<'n, K, V, const B: usize> {
    pub(super) inner: Iter<'n, K, V, B>,
}

impl<'n, K, V, const B: usize> Iterator for Pairs<'n, K, V, B> {
    type Item = (&'n K, &'n V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'n, K, V, const B: usize> DoubleEndedIterator for Pairs<'n, K, V, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}
