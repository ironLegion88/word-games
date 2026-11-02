//! Tests for Boggle scoring rules, boundaries, and checked arithmetic.

use boggle::board::Board;
use boggle::path::find_word_path;
use boggle::scoring::{calculate_total_score, word_score};
use trie::Trie;

#[test]
fn test_scoring_boundaries() {
    // Lengths < 3 score 0
    assert_eq!(word_score("").unwrap(), 0);
    assert_eq!(word_score("A").unwrap(), 0);
    assert_eq!(word_score("AT").unwrap(), 0);

    // 3, 4 -> 1
    assert_eq!(word_score("CAT").unwrap(), 1);
    assert_eq!(word_score("CATS").unwrap(), 1);

    // 5 -> 2
    assert_eq!(word_score("CRANE").unwrap(), 2);

    // 6 -> 3
    assert_eq!(word_score("BEAUTY").unwrap(), 3);

    // 7 -> 5
    assert_eq!(word_score("COMPACT").unwrap(), 5);

    // 8 -> 11
    assert_eq!(word_score("BUILDING").unwrap(), 11);

    // 9..=12 -> 2 * length
    assert_eq!(word_score("BEAUTIFUL").unwrap(), 18); // 9 * 2
    assert_eq!(word_score("COMPUTERS").unwrap(), 18); // 9 * 2
    assert_eq!(word_score("CHALLENGES").unwrap(), 20); // 10 * 2
    assert_eq!(word_score("DEVELOPMENT").unwrap(), 22); // 11 * 2
    assert_eq!(word_score("DICTIONARIES").unwrap(), 24); // 12 * 2

    // 13..=14 -> 3 * length
    assert_eq!(word_score("DETERMINISTIC").unwrap(), 39); // 13 * 3
    assert_eq!(word_score("COMMUNICATIONS").unwrap(), 42); // 14 * 3

    // >= 15 -> 4 * length
    assert_eq!(word_score("REPRESENTATIONS").unwrap(), 60); // 15 * 4
    assert_eq!(word_score("CHARACTERIZATION").unwrap(), 64); // 16 * 4
    assert_eq!(word_score("INSTITUTIONALIZED").unwrap(), 68); // 17 * 4
}

#[test]
fn test_total_score_calculation() {
    let words = vec!["CAT", "CRANE", "BUILDING", "COMPUTERS"];
    // 1 + 2 + 11 + 18 = 32
    assert_eq!(calculate_total_score(words).unwrap(), 32);
}

#[test]
fn test_realizable_over_15_letter_fixture() {
    // A 16-letter word formed on a 4x4 board:
    // "UNCHARACTERISTIC" is 16 letters, scores 16 * 4 = 64 points.
    // Let's create a 4x4 board where a 16-letter word traverses every cell:
    // Row 0: U N C H
    // Row 1: E T C A (moving right to left: H -> A -> R -> A -> C -> T -> E -> R -> ...)
    // Simpler: a serpentine 16-letter snake path on a 4x4 board:
    // Cells: 0, 1, 2, 3, 7, 6, 5, 4, 8, 9, 10, 11, 15, 14, 13, 12
    // Let the word be "ABCDEFGHIJKLMNOP" (16 letters, custom dictionary word):
    let letters = "ABCDEFGHIJKLMNOP";
    let mut grid = vec![' '; 16];
    let path = [0, 1, 2, 3, 7, 6, 5, 4, 8, 9, 10, 11, 15, 14, 13, 12];
    for (i, &cell) in path.iter().enumerate() {
        grid[cell] = letters.chars().nth(i).unwrap();
    }
    let grid_str: String = grid.into_iter().collect();

    let board = Board::from_str_tiles(4, &grid_str).unwrap();

    // Verify word exists on board and can be found
    let found_path = find_word_path(&board, letters).expect("16-letter snake path must be found");
    assert_eq!(found_path, path.to_vec());

    // Score for 16-letter word: 16 * 4 = 64
    let score = word_score(letters).unwrap();
    assert_eq!(score, 64);

    // Verify in custom trie
    let mut trie = Trie::new();
    trie.insert(letters).unwrap();
    assert!(trie.contains(letters));
}
