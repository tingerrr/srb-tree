use tree::set::SrbTreeSet;

mod key;
mod tree;

fn main() {
    let mut node = SrbTreeSet::new();
    node.insert(1);
    node.insert(32);
    node.insert(2);
    node.insert(7);
    // node.remove(&6);
    node.insert(1128369);
    dbg!(node);
}
