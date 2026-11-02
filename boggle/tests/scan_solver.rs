//! Tests for BG-S1E dictionary scan solver baseline.

use boggle::board::Board;
use boggle::solver::solve_scan;
use trie::Trie;

#[test]
fn test_solve_scan_tiny_fixture() {
    let mut trie = Trie::new();
    trie.insert("CAT").unwrap();
    trie.insert("CATS").unwrap();
    trie.insert("DOG").unwrap();
    trie.insert("DOGS").unwrap();
    trie.insert("BIRD").unwrap();
    trie.insert("COED").unwrap();
    trie.insert("AT").unwrap(); // length 2 (< 3)
    trie.insert("ZEBRA").unwrap(); // not on board

    let input = "
        C A T S
        D O G S
        Qu R E W
        B I R D
    ";
    let board = Board::from_str_tiles(4, input).unwrap();
    let solutions = solve_scan(&board, &trie).expect("solve_scan succeeds");

    // All solutions must have length >= 3
    for s in &solutions {
        assert!(s.len() >= 3);
    }

    // Solutions must be sorted and strictly deduplicated
    for w in solutions.windows(2) {
        assert!(w[0] < w[1], "solutions not strictly ascending: {:?}", w);
    }

    // Expected words
    assert!(solutions.contains(&"BIRD".to_string()));
    assert!(solutions.contains(&"CAT".to_string()));
    assert!(solutions.contains(&"CATS".to_string()));
    assert!(solutions.contains(&"COED".to_string()));
    assert!(solutions.contains(&"DOG".to_string()));
    assert!(solutions.contains(&"DOGS".to_string()));

    // Excluded words
    assert!(!solutions.contains(&"AT".to_string()));
    assert!(!solutions.contains(&"ZEBRA".to_string()));
}

#[test]
fn test_solve_scan_zero_solutions() {
    let mut trie = Trie::new();
    trie.insert("CAT").unwrap();
    trie.insert("DOG").unwrap();

    // Board containing only Zs
    let board = Board::from_str_tiles(4, "ZZZZZZZZZZZZZZZZ").unwrap();
    let solutions = solve_scan(&board, &trie).expect("solve_scan succeeds");
    assert!(solutions.is_empty());
}

#[test]
fn test_solve_scan_pathological_repeated_letters() {
    let mut trie = Trie::new();
    trie.insert("AAA").unwrap();
    trie.insert("AAAA").unwrap();
    trie.insert("AAAAA").unwrap();
    trie.insert(&"A".repeat(16)).unwrap();
    trie.insert(&"B".repeat(5)).unwrap(); // absent

    let board = Board::from_str_tiles(4, "AAAAAAAAAAAAAAAA").unwrap();
    let solutions = solve_scan(&board, &trie).expect("solve_scan succeeds");

    assert!(solutions.contains(&"AAA".to_string()));
    assert!(solutions.contains(&"AAAA".to_string()));
    assert!(solutions.contains(&"AAAAA".to_string()));
    assert!(solutions.contains(&"A".repeat(16)));
    assert!(!solutions.contains(&"BBBBB".to_string()));
}
