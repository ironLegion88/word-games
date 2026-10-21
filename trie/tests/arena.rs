use trie::node::{Node, NO_CHILD};
use trie::Trie;

#[test]
fn fresh_trie_arena_invariants() {
    let trie = Trie::new();
    assert_eq!(trie.stats().words, 0);
    assert_eq!(trie.stats().live_nodes, 1);
    assert_eq!(trie.stats().total_slots, 1);
    assert_eq!(trie.stats().free_slots, 0);
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn node_struct_size_and_sentinel() {
    // 104 bytes children + 1 byte is_word + 3 padding + 4 bytes subtree_words = 112 bytes
    assert_eq!(std::mem::size_of::<Node>(), 112);
    let node = Node::empty();
    assert_eq!(node.degree(), 0);
    assert!(!node.has_children());
    assert!(!node.is_word);
    assert_eq!(node.subtree_words, 0);
    for &c in &node.children {
        assert_eq!(c, NO_CHILD);
    }
}
