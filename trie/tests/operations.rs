use std::collections::HashSet;
use trie::{Letter, Trie, WordError};

#[test]
fn empty_trie_queries() {
    let trie = Trie::new();
    assert_eq!(trie.len(), 0);
    assert!(trie.is_empty());
    assert!(!trie.contains(""));
    assert!(!trie.contains("A"));
    assert!(!trie.contains("HELLO"));
    assert!(trie.has_prefix(""));
    assert!(!trie.has_prefix("A"));
    assert_eq!(trie.try_contains(""), Err(WordError::EmptyWord));
    assert!(trie.check_invariants().is_ok());

    let mut words = Vec::new();
    trie.visit_words(|w| words.push(w.to_string()));
    assert!(words.is_empty());
}

#[test]
fn single_word_lifecycle() {
    let mut trie = Trie::new();
    let inserted = trie.insert("HELLO").expect("valid insert");
    assert!(inserted);
    assert_eq!(trie.len(), 1);
    assert!(!trie.is_empty());
    assert!(trie.contains("HELLO"));
    assert!(trie.contains("hello")); // case-insensitive query
    assert!(trie.contains("HeLLo"));
    assert!(!trie.contains("HELL"));
    assert!(!trie.contains("HELLOS"));
    assert!(trie.has_prefix(""));
    assert!(trie.has_prefix("H"));
    assert!(trie.has_prefix("HE"));
    assert!(trie.has_prefix("HEL"));
    assert!(trie.has_prefix("HELL"));
    assert!(trie.has_prefix("HELLO"));
    assert!(!trie.has_prefix("HELLOS"));
    assert!(!trie.has_prefix("HA"));
    assert!(trie.check_invariants().is_ok());

    // Duplicate insert returns false and does not change counts
    let stats_before = trie.stats();
    let reinserted = trie.insert("hello").expect("duplicate insert");
    assert!(!reinserted);
    assert_eq!(trie.stats(), stats_before);
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn overlapping_branches_and_subprefixes() {
    let mut trie = Trie::new();
    // Insert CAR, CART, CAT, CARE
    assert!(trie.insert("CAR").unwrap());
    assert!(trie.insert("CART").unwrap());
    assert!(trie.insert("CAT").unwrap());
    assert!(trie.insert("CARE").unwrap());
    assert_eq!(trie.len(), 4);

    assert!(trie.contains("CAR"));
    assert!(trie.contains("CART"));
    assert!(trie.contains("CAT"));
    assert!(trie.contains("CARE"));

    assert!(!trie.contains("C"));
    assert!(!trie.contains("CA"));
    assert!(!trie.contains("CARTS"));

    assert!(trie.has_prefix("C"));
    assert!(trie.has_prefix("CA"));
    assert!(trie.has_prefix("CAR"));
    assert!(trie.has_prefix("CART"));
    assert!(trie.has_prefix("CAT"));
    assert!(trie.has_prefix("CARE"));

    assert!(trie.check_invariants().is_ok());

    // Subtree words at root should be 4
    let cursor = trie.cursor();
    assert_eq!(cursor.subtree_words(), 4);

    // Advance cursor C -> A -> R
    let c = cursor.advance(Letter::from_char('C').unwrap()).unwrap();
    assert_eq!(c.subtree_words(), 4);
    assert!(!c.is_word());

    let ca = c.advance(Letter::from_char('A').unwrap()).unwrap();
    assert_eq!(ca.subtree_words(), 4);
    assert!(!ca.is_word());

    let car = ca.advance(Letter::from_char('R').unwrap()).unwrap();
    assert_eq!(car.subtree_words(), 3); // CAR, CART, CARE
    assert!(car.is_word()); // CAR is a word

    let cart = car.advance(Letter::from_char('T').unwrap()).unwrap();
    assert_eq!(cart.subtree_words(), 1); // CART
    assert!(cart.is_word());
    assert!(!cart.has_children());
}

#[test]
fn prefix_itself_becomes_word_later() {
    let mut trie = Trie::new();
    assert!(trie.insert("CART").unwrap());
    assert_eq!(trie.len(), 1);
    assert!(!trie.contains("CAR"));
    let nodes_count = trie.stats().live_nodes;

    // Now insert CAR (which is a prefix of CART)
    assert!(trie.insert("CAR").unwrap());
    assert_eq!(trie.len(), 2);
    assert!(trie.contains("CAR"));
    assert!(trie.contains("CART"));
    // No new nodes allocated because path C-A-R already existed!
    assert_eq!(trie.stats().live_nodes, nodes_count);
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn invalid_input_does_not_mutate() {
    let mut trie = Trie::new();
    trie.insert("GOOD").unwrap();
    let stats_before = trie.stats();

    assert!(trie.insert("").is_err());
    assert!(trie.insert("bad-word").is_err());
    assert!(trie.insert("space word").is_err());
    assert!(trie.insert("word1").is_err());
    assert!(trie.insert("café").is_err());

    assert_eq!(trie.stats(), stats_before);
    assert_eq!(trie.len(), 1);
    assert!(trie.contains("GOOD"));
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn lexicographic_iteration_matches_sorted_oracle() {
    let mut trie = Trie::new();
    let words = vec![
        "ZEBRA",
        "APPLE",
        "BALL",
        "CAT",
        "DOG",
        "BAT",
        "APPLICATION",
        "APPLY",
        "AXE",
    ];

    let mut oracle = HashSet::new();
    for &w in &words {
        trie.insert(w).unwrap();
        oracle.insert(w.to_string());
    }

    let mut iterated = Vec::new();
    trie.visit_words(|w| iterated.push(w.to_string()));

    let mut sorted_oracle: Vec<String> = oracle.into_iter().collect();
    sorted_oracle.sort();

    assert_eq!(iterated, sorted_oracle);
}

#[test]
fn contains_does_not_mutate() {
    let mut trie = Trie::new();
    trie.insert("TEST").unwrap();
    let stats_before = trie.stats();

    assert!(trie.contains("TEST"));
    assert!(!trie.contains("NOTPRESENT"));
    assert!(!trie.contains(""));
    assert!(!trie.contains("invalid!"));

    assert_eq!(trie.stats(), stats_before);
}
