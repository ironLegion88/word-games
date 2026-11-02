//! Tests for BG-S2 trie-guided solver and differential equality against BG-S1E.

use rand::SeedableRng;
use std::path::Path;
use trie::Trie;

use boggle::board::Board;
use boggle::error::SolverError;
use boggle::generation::generate_uniform;
use boggle::solver::{solve_scan, solve_trie, solve_trie_with_limit};

fn load_test_dictionary() -> Option<Trie> {
    let dict_paths = [
        Path::new("assets/dictionary.txt"),
        Path::new("../assets/dictionary.txt"),
        Path::new("../../assets/dictionary.txt"),
    ];
    let path = dict_paths.iter().find(|p| p.exists())?;
    Trie::from_file(path).ok()
}

#[test]
fn test_solve_trie_tiny_fixture() {
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
    let trie_solutions = solve_trie(&board, &trie).expect("solve_trie succeeds");
    let scan_solutions = solve_scan(&board, &trie).expect("solve_scan succeeds");

    assert_eq!(trie_solutions, scan_solutions);
}

#[test]
fn test_solve_trie_zero_solutions() {
    let mut trie = Trie::new();
    trie.insert("CAT").unwrap();
    trie.insert("DOG").unwrap();

    let board = Board::from_str_tiles(4, "ZZZZZZZZZZZZZZZZ").unwrap();
    let trie_solutions = solve_trie(&board, &trie).expect("solve_trie succeeds");
    let scan_solutions = solve_scan(&board, &trie).expect("solve_scan succeeds");

    assert!(trie_solutions.is_empty());
    assert_eq!(trie_solutions, scan_solutions);
}

#[test]
fn test_solve_trie_pathological_repeated_letters() {
    let mut trie = Trie::new();
    trie.insert("AAA").unwrap();
    trie.insert("AAAA").unwrap();
    trie.insert("AAAAA").unwrap();
    trie.insert(&"A".repeat(16)).unwrap();
    trie.insert(&"B".repeat(5)).unwrap();

    let board = Board::from_str_tiles(4, "AAAAAAAAAAAAAAAA").unwrap();
    let trie_solutions = solve_trie(&board, &trie).expect("solve_trie succeeds");
    let scan_solutions = solve_scan(&board, &trie).expect("solve_scan succeeds");

    assert_eq!(trie_solutions, scan_solutions);
    assert!(trie_solutions.contains(&"AAA".to_string()));
    assert!(trie_solutions.contains(&"AAAA".to_string()));
    assert!(trie_solutions.contains(&"AAAAA".to_string()));
    assert!(trie_solutions.contains(&"A".repeat(16)));
}

#[test]
fn test_solve_trie_qu_board_words() {
    let mut trie = Trie::new();
    trie.insert("QUIET").unwrap();
    trie.insert("QUICK").unwrap();
    trie.insert("QUIT").unwrap();
    trie.insert("BIRD").unwrap();

    // 4x4 board containing Qu tile
    let input = "
        Qu I C K
        U  E T A
        B  I R D
        X  Y Z W
    ";
    let board = Board::from_str_tiles(4, input).unwrap();
    let trie_solutions = solve_trie(&board, &trie).unwrap();
    let scan_solutions = solve_scan(&board, &trie).unwrap();

    assert_eq!(trie_solutions, scan_solutions);
    assert!(trie_solutions.contains(&"QUICK".to_string()));
    assert!(trie_solutions.contains(&"QUIT".to_string()));
    assert!(trie_solutions.contains(&"BIRD".to_string()));
}

#[test]
fn test_search_limit_exceeded_error() {
    let mut trie = Trie::new();
    trie.insert("CATS").unwrap();
    trie.insert("DOGS").unwrap();

    let board = Board::from_str_tiles(4, "CATSDOGSFROGBIRD").unwrap();

    // Extremely small expansion limit of 2
    let res = solve_trie_with_limit(&board, &trie, Some(2));
    assert!(matches!(
        res,
        Err(SolverError::SearchLimitExceeded {
            expansions: 3,
            limit: 2
        })
    ));
}

#[test]
fn test_differential_seeded_boards_full_corpus() {
    let trie = match load_test_dictionary() {
        Some(t) => t,
        None => {
            eprintln!("Skipping corpus test: dictionary.txt not found");
            return;
        }
    };

    // Test across 3 recorded seeds for 4x4, 5x5, 6x6
    let seeds = [0x5EED_2026, 0xC0FFEE, 0xBADC0DE];

    for &seed in &seeds {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        for size in [4, 5, 6] {
            let board = generate_uniform(size, &mut rng).unwrap();

            let scan_res = solve_scan(&board, &trie).expect("scan solver succeeds");
            let trie_res = solve_trie(&board, &trie).expect("trie solver succeeds");

            assert_eq!(
                scan_res, trie_res,
                "Solvers diverged for size {size}x{size} with seed {seed:#x}"
            );
        }
    }
}
