use trie::Trie;

#[test]
fn delete_absent_word() {
    let mut trie = Trie::new();
    trie.insert("APPLE").unwrap();
    let stats_before = trie.stats();

    assert!(!trie.remove("BANANA").unwrap());
    assert_eq!(trie.stats(), stats_before);
    assert!(trie.contains("APPLE"));
    assert!(trie.check_invariants().is_ok());

    // Absent prefix of existing word
    assert!(!trie.remove("APP").unwrap());
    assert_eq!(trie.stats(), stats_before);
    assert!(trie.contains("APPLE"));
    assert!(trie.check_invariants().is_ok());

    // Absent extension of existing word
    assert!(!trie.remove("APPLES").unwrap());
    assert_eq!(trie.stats(), stats_before);
    assert!(trie.contains("APPLE"));
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn delete_leaf_prunes_branch() {
    let mut trie = Trie::new();
    trie.insert("CAR").unwrap();
    trie.insert("CART").unwrap();

    let stats_before = trie.stats();
    assert_eq!(stats_before.words, 2);
    // Root (0) -> C (1) -> A (2) -> R (3) -> T (4)
    assert_eq!(stats_before.live_nodes, 5);

    // Delete CART (leaf)
    assert!(trie.remove("CART").unwrap());
    assert_eq!(trie.len(), 1);
    assert!(!trie.contains("CART"));
    assert!(trie.contains("CAR"));

    let stats_after = trie.stats();
    assert_eq!(stats_after.words, 1);
    assert_eq!(stats_after.live_nodes, 4); // Node T was pruned!
    assert_eq!(stats_after.free_slots, 1); // Node T moved to free list!
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn delete_word_that_is_prefix_of_another() {
    let mut trie = Trie::new();
    trie.insert("CAR").unwrap();
    trie.insert("CART").unwrap();

    let live_nodes_before = trie.stats().live_nodes;

    // Delete CAR (prefix of CART)
    assert!(trie.remove("CAR").unwrap());
    assert_eq!(trie.len(), 1);
    assert!(!trie.contains("CAR"));
    assert!(trie.contains("CART"));

    // No nodes should be pruned because all nodes C-A-R are needed for CART
    assert_eq!(trie.stats().live_nodes, live_nodes_before);
    assert_eq!(trie.stats().free_slots, 0);
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn delete_child_while_keeping_siblings() {
    let mut trie = Trie::new();
    trie.insert("CAR").unwrap();
    trie.insert("CAT").unwrap();

    assert_eq!(trie.len(), 2);
    // Root -> C -> A -> R and C -> A -> T
    // Live nodes: Root, C, A, R, T = 5
    assert_eq!(trie.stats().live_nodes, 5);

    assert!(trie.remove("CAT").unwrap());
    assert_eq!(trie.len(), 1);
    assert!(!trie.contains("CAT"));
    assert!(trie.contains("CAR"));

    // T is pruned, but C and A remain for CAR
    assert_eq!(trie.stats().live_nodes, 4);
    assert_eq!(trie.stats().free_slots, 1);
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn delete_every_word_retains_root() {
    let mut trie = Trie::new();
    trie.insert("HELLO").unwrap();
    trie.insert("WORLD").unwrap();

    assert_eq!(trie.len(), 2);

    assert!(trie.remove("HELLO").unwrap());
    assert!(trie.check_invariants().is_ok());

    assert!(trie.remove("WORLD").unwrap());
    assert!(trie.check_invariants().is_ok());

    assert_eq!(trie.len(), 0);
    assert!(trie.is_empty());
    assert_eq!(trie.stats().words, 0);
    assert_eq!(trie.stats().live_nodes, 1); // Root retained!
    assert_eq!(trie.stats().free_slots, trie.stats().total_slots - 1);
}

#[test]
fn reinsert_deleted_word_reuses_free_slots() {
    let mut trie = Trie::new();
    trie.insert("TEST").unwrap();
    let initial_slots = trie.stats().total_slots;

    assert!(trie.remove("TEST").unwrap());
    assert_eq!(trie.stats().free_slots, 4); // T, E, S, T slots freed
    assert_eq!(trie.stats().total_slots, initial_slots);

    // Reinsert TEST
    assert!(trie.insert("TEST").unwrap());
    assert_eq!(trie.stats().free_slots, 0); // All 4 slots reused!
    assert_eq!(trie.stats().total_slots, initial_slots); // Arena did not grow!
    assert_eq!(trie.len(), 1);
    assert!(trie.contains("TEST"));
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn case_insensitive_delete() {
    let mut trie = Trie::new();
    trie.insert("Rust").unwrap();
    assert!(trie.remove("rust").unwrap());
    assert_eq!(trie.len(), 0);
    assert!(!trie.contains("RUST"));
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn duplicate_delete_returns_false() {
    let mut trie = Trie::new();
    trie.insert("WORD").unwrap();
    assert!(trie.remove("WORD").unwrap());
    assert!(!trie.remove("WORD").unwrap());
}

#[test]
fn invalid_delete_leaves_state_unchanged() {
    let mut trie = Trie::new();
    trie.insert("WORD").unwrap();
    let stats_before = trie.stats();

    assert!(trie.remove("").is_err());
    assert!(trie.remove("word!").is_err());
    assert!(trie.remove("w ord").is_err());

    assert_eq!(trie.stats(), stats_before);
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn compaction_remaps_indices_and_shrinks() {
    let mut trie = Trie::new();
    trie.insert("APPLE").unwrap();
    trie.insert("APRICOT").unwrap();
    trie.insert("BANANA").unwrap();
    trie.insert("CHERRY").unwrap();

    let total_before = trie.stats().total_slots;

    // Remove BANANA and CHERRY, creating a fragmented free list
    assert!(trie.remove("BANANA").unwrap());
    assert!(trie.remove("CHERRY").unwrap());

    assert!(trie.stats().free_slots > 0);
    assert!(trie.stats().total_slots == total_before);

    // Force compaction
    trie.compact();

    assert_eq!(trie.stats().free_slots, 0);
    assert_eq!(trie.stats().total_slots, trie.stats().live_nodes);
    assert!(trie.contains("APPLE"));
    assert!(trie.contains("APRICOT"));
    assert!(!trie.contains("BANANA"));
    assert!(!trie.contains("CHERRY"));
    assert!(trie.check_invariants().is_ok());

    let mut words = Vec::new();
    trie.visit_words(|w| words.push(w.to_string()));
    assert_eq!(words, vec!["APPLE", "APRICOT"]);
}

#[test]
fn churn_1000_operations_stress_test() {
    let mut trie = Trie::new();
    let word_list = [
        "ALPHA", "BRAVO", "CHARLIE", "DELTA", "ECHO", "FOXTROT", "GOLF", "HOTEL", "INDIA",
        "JULIETT", "KILO", "LIMA", "MIKE", "NOVEMBER", "OSCAR", "PAPA", "QUEBEC", "ROMEO",
        "SIERRA", "TANGO", "UNIFORM", "VICTOR", "WHISKEY", "XRAY", "YANKEE", "ZULU",
    ];

    // Churn: repeatedly insert, check, delete, check
    for cycle in 0..100 {
        for &w in &word_list {
            trie.insert(w).unwrap();
        }
        assert_eq!(trie.len(), word_list.len());

        for &w in &word_list {
            assert!(trie.contains(w));
        }

        // Remove half
        for &w in &word_list[..13] {
            assert!(trie.remove(w).unwrap());
        }
        assert_eq!(trie.len(), 13);

        // Compact on some cycles
        if cycle % 10 == 0 {
            trie.compact();
            assert!(trie.check_invariants().is_ok());
        }

        // Remove the rest
        for &w in &word_list[13..] {
            assert!(trie.remove(w).unwrap());
        }
        assert_eq!(trie.len(), 0);
        assert!(trie.check_invariants().is_ok());
    }

    assert_eq!(trie.len(), 0);
    assert_eq!(trie.stats().live_nodes, 1);
    assert!(trie.check_invariants().is_ok());
}
