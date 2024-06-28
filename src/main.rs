use tree::set::SrbTreeSet;

mod key;
mod tree;

fn main() {
    let set: SrbTreeSet<_> = (0..100_000).step_by(10).collect();
    eprintln!("storage util: {:.2}%", set.storage() * 100.0);
}
