use tree::set::SrbTreeSet;

mod key;
mod tree;

fn main() {
    let mut set: SrbTreeSet<_> = (0..100_000).step_by(31).collect();
    let util = set.storage();
    let bytes = set.storage_bytes();

    eprintln!(
        "storage util: {:.2}% ({}/{} bytes used)",
        util * 100.0,
        (bytes as f32 * util) as usize,
        bytes,
    );

    set = (0..100).step_by(10).collect();

    let mut iter = set.iter();
    assert_eq!(iter.next(), Some(&0));
    assert_eq!(iter.next_back(), Some(&90));
    assert_eq!(iter.next(), Some(&10));
    assert_eq!(iter.next_back(), Some(&80));
    assert_eq!(iter.next(), Some(&20));

    assert_eq!(iter.next_back(), Some(&70));
    assert_eq!(iter.next(), Some(&30));
    assert_eq!(iter.next_back(), Some(&60));
    assert_eq!(iter.next(), Some(&40));
    assert_eq!(iter.next_back(), Some(&50));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}
