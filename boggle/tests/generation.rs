//! Tests for Boggle board generation: uniform generator and normal play policy (Milestone BG-G1).

use rand::SeedableRng;
use std::path::Path;
use trie::Trie;

use boggle::board::{Board, MAX_BOARD_SIZE, MIN_BOARD_SIZE};
use boggle::error::GenerationError;
use boggle::generation::{generate_playable_board, generate_uniform, BoardGenerator};
use boggle::solver::{solve, SolverStrategy};

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
fn test_uniform_seed_reproducibility() {
    let seed = 0x5EED_2026;
    let mut rng1 = rand::rngs::StdRng::seed_from_u64(seed);
    let mut rng2 = rand::rngs::StdRng::seed_from_u64(seed);

    for size in MIN_BOARD_SIZE..=MAX_BOARD_SIZE {
        let b1 = generate_uniform(size, &mut rng1).unwrap();
        let b2 = generate_uniform(size, &mut rng2).unwrap();
        assert_eq!(b1, b2, "Identical seeds must produce identical boards");
    }
}

#[test]
fn test_different_seeds_produce_distinct_boards() {
    let mut rng1 = rand::rngs::StdRng::seed_from_u64(111);
    let mut rng2 = rand::rngs::StdRng::seed_from_u64(222);

    let b1 = generate_uniform(4, &mut rng1).unwrap();
    let b2 = generate_uniform(4, &mut rng2).unwrap();
    assert_ne!(b1, b2);
}

#[test]
fn test_generation_size_validation() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    for invalid in [0, 1, 2, 3, 7, 8] {
        let res = generate_uniform(invalid, &mut rng);
        assert!(matches!(res, Err(GenerationError::InvalidSize { .. })));
    }
}

#[test]
fn test_playable_board_guarantees_nonempty_solutions() {
    let trie = match load_test_dictionary() {
        Some(t) => t,
        None => return,
    };

    let mut rng = rand::rngs::StdRng::seed_from_u64(0xC0FFEE);
    for size in [4, 5, 6] {
        let (board, solutions) = generate_playable_board(
            size,
            BoardGenerator::Uniform,
            &trie,
            SolverStrategy::Trie,
            &mut rng,
            10,
        )
        .expect("playable board generation succeeds");

        assert_eq!(board.size(), size);
        assert!(
            !solutions.is_empty(),
            "Playable board must contain at least one solution"
        );
    }
}

#[test]
fn test_retry_exhaustion_on_empty_dictionary() {
    let empty_trie = Trie::new();
    let mut rng = rand::rngs::StdRng::seed_from_u64(999);

    let res = generate_playable_board(
        4,
        BoardGenerator::Uniform,
        &empty_trie,
        SolverStrategy::Trie,
        &mut rng,
        5,
    );

    assert_eq!(
        res.unwrap_err(),
        GenerationError::NoPlayableBoard { retries: 5 }
    );
}

#[test]
fn test_fixed_empty_board_retained_for_tests() {
    let mut trie = Trie::new();
    trie.insert("APPLE").unwrap();

    // Board with letters not in APPLE
    let fixed_empty = Board::from_str_tiles(4, "ZZZZZZZZZZZZZZZZ").unwrap();
    let solutions = solve(&fixed_empty, &trie, SolverStrategy::Trie).unwrap();
    assert!(solutions.is_empty());
}
