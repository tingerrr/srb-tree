#![feature(iter_array_chunks)]

use node::Node;

mod key;
mod node;

fn main() {
    let mut node: Node<u64, (), 32> = Node::new_internal();
    node.insert(0, 1, ());
    node.insert(0, 32, ());
    node.insert(0, 2, ());
    node.insert(0, 7, ());
    node.insert(0, 1128369, ());
    dbg!(node);
}
