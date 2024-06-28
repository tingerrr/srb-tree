use tree::set::SrbTreeSet;

mod key;
mod tree;

fn main() {
    let set: SrbTreeSet<_> = (0..100_000).step_by(31).collect();
    let util = set.storage();
    let bytes = set.storage_bytes();

    eprintln!(
        "storage util: {:.2}% ({}/{} bytes used)",
        util * 100.0,
        (bytes as f32 * util) as usize,
        bytes,
    );
}
