use spelling_bee::{solve_filter, SpellingBeePuzzle};
use trie::Trie;

#[test]
fn test_filter_solver_tiny_fixture() {
    let mut trie = Trie::new();
    let words = [
        "AROMATIC", // pangram (uses C,A,I,M,O,R,T)
        "CITRIC",   // valid (6 letters, center C)
        "CRITIC",   // valid (6 letters, center C)
        "ATOM",     // invalid: missing center C
        "ATOMIC",   // valid (6 letters, center C)
        "ACAI",     // valid (4 letters, center C)
        "MICRO",    // valid (5 letters, center C)
        "CAT",      // invalid: < 4 letters
        "DOG",      // invalid: letters not in puzzle
        "ROCOCO",   // valid: uses R, O, C (all in puzzle, has C)
    ];

    for w in words {
        trie.insert(w).unwrap();
    }

    let puzzle = SpellingBeePuzzle::from_raw("CAIMORT").unwrap();
    let solutions = solve_filter(&puzzle, &trie);

    let expected = vec![
        "ACAI".to_string(),
        "AROMATIC".to_string(),
        "ATOMIC".to_string(),
        "CITRIC".to_string(),
        "CRITIC".to_string(),
        "MICRO".to_string(),
        "ROCOCO".to_string(),
    ];

    assert_eq!(solutions, expected);
}

#[test]
fn test_filter_solver_empty_trie() {
    let trie = Trie::new();
    let puzzle = SpellingBeePuzzle::from_raw("CAIMORT").unwrap();
    let solutions = solve_filter(&puzzle, &trie);
    assert!(solutions.is_empty());
}

#[test]
fn test_filter_solver_zero_solutions() {
    let mut trie = Trie::new();
    trie.insert("ZEBRA").unwrap();
    trie.insert("QUEEN").unwrap();

    let puzzle = SpellingBeePuzzle::from_raw("CAIMORT").unwrap();
    let solutions = solve_filter(&puzzle, &trie);
    assert!(solutions.is_empty());
}
