//! Tests for Boggle board generation:
//! - BG-G1: Uniform letter sampling.
//! - BG-G2: Dictionary frequency-weighted letter sampling.
//! - BG-G3: Synthetic 6-face dice simulation for all board sizes (4, 5, 6).

use rand::SeedableRng;
use std::collections::HashMap;
use std::path::Path;
use trie::Trie;

use boggle::board::{Board, MAX_BOARD_SIZE, MIN_BOARD_SIZE};
use boggle::error::GenerationError;
use boggle::generation::{
    derive_dice_faces, derive_dictionary_weights, generate_dice, generate_playable_board,
    generate_uniform, generate_weighted, get_dice_for_size, BoardGenerator, BG_G1_VERSION,
    BG_G2_VERSION, BG_G2_WEIGHTS, BG_G3_VERSION, DICTIONARY_CORPUS_SHA256,
};
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

// -------------------------------------------------------------------------
// BG-G1: Uniform Generator Tests
// -------------------------------------------------------------------------

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
        let res_u = generate_uniform(invalid, &mut rng);
        assert!(matches!(res_u, Err(GenerationError::InvalidSize { .. })));

        let res_w = generate_weighted(invalid, &mut rng);
        assert!(matches!(res_w, Err(GenerationError::InvalidSize { .. })));

        let res_d = generate_dice(invalid, &mut rng);
        assert!(matches!(res_d, Err(GenerationError::InvalidSize { .. })));
    }
}

// -------------------------------------------------------------------------
// BG-G2: Frequency-Weighted Generator Tests
// -------------------------------------------------------------------------

#[test]
fn test_derive_weights_matches_versioned_constant() {
    let trie = match load_test_dictionary() {
        Some(t) => t,
        None => return,
    };

    let derived = derive_dictionary_weights(&trie);
    assert_eq!(
        derived, BG_G2_WEIGHTS,
        "Derived weights from corpus must exactly match BG_G2_WEIGHTS constant"
    );
    assert_eq!(BG_G1_VERSION, "BG-G1-UNIFORM-V1");
    assert_eq!(BG_G2_VERSION, "BG-G2-WEIGHTED-V1");
    assert_eq!(BG_G3_VERSION, "BG-G3-DICE-V1");
    assert_eq!(
        DICTIONARY_CORPUS_SHA256,
        "eda7bc8b86a534de8065a7d0cc091dc918d400475061749d05a5711afda12a3b"
    );
}

#[test]
fn test_weighted_seed_reproducibility() {
    let seed = 0xCAFE_BABE;
    let mut rng1 = rand::rngs::StdRng::seed_from_u64(seed);
    let mut rng2 = rand::rngs::StdRng::seed_from_u64(seed);

    for size in MIN_BOARD_SIZE..=MAX_BOARD_SIZE {
        let b1 = generate_weighted(size, &mut rng1).unwrap();
        let b2 = generate_weighted(size, &mut rng2).unwrap();
        assert_eq!(
            b1, b2,
            "Weighted generator must be reproducible with same seed"
        );
    }
}

#[test]
fn test_weighted_sampling_frequency_distribution() {
    // Generate many tiles and verify frequent letters (E, S, I, A) outnumber rare (Q, J, Z)
    let mut rng = rand::rngs::StdRng::seed_from_u64(0x1234_5678);
    let mut counts: HashMap<String, usize> = HashMap::new();

    for _ in 0..100 {
        let board = generate_weighted(5, &mut rng).unwrap();
        for &tile in board.tiles() {
            *counts.entry(tile.to_string()).or_default() += 1;
        }
    }

    let count_e = counts.get("E").copied().unwrap_or(0);
    let count_s = counts.get("S").copied().unwrap_or(0);
    let count_z = counts.get("Z").copied().unwrap_or(0);
    let count_q = counts.get("Qu").copied().unwrap_or(0);

    assert!(
        count_e > count_z * 5,
        "E count ({count_e}) must exceed Z count ({count_z})"
    );
    assert!(
        count_s > count_q * 5,
        "S count ({count_s}) must exceed Qu count ({count_q})"
    );
}

// -------------------------------------------------------------------------
// BG-G3: Synthetic 6-Face Dice Tests
// -------------------------------------------------------------------------

#[test]
fn test_dice_properties_all_board_sizes() {
    for size in MIN_BOARD_SIZE..=MAX_BOARD_SIZE {
        let dice = get_dice_for_size(size).expect("valid size dice");
        let expected_dice_count = size * size;
        assert_eq!(dice.len(), expected_dice_count);

        let mut total_face_count = 0;
        for (die_idx, die) in dice.iter().enumerate() {
            assert_eq!(die.len(), 6, "Die {die_idx} must have exactly 6 faces");
            for &face in die {
                total_face_count += 1;
                // Face must have valid spelled length (1 for letter, 2 for Qu)
                assert!(face.spelled_len() == 1 || face.spelled_len() == 2);
            }
        }
        assert_eq!(total_face_count, 6 * expected_dice_count);
    }
}

#[test]
fn test_derive_dice_faces_largest_remainder_conservation() {
    for size in [4, 5, 6] {
        let dice = derive_dice_faces(size, &BG_G2_WEIGHTS);
        let total_faces = dice.len() * 6;
        assert_eq!(total_faces, 6 * size * size);
    }
}

#[test]
fn test_dice_board_generation_reproducibility() {
    let seed = 0xD1CE_2026;
    let mut rng1 = rand::rngs::StdRng::seed_from_u64(seed);
    let mut rng2 = rand::rngs::StdRng::seed_from_u64(seed);

    for size in MIN_BOARD_SIZE..=MAX_BOARD_SIZE {
        let b1 = generate_dice(size, &mut rng1).unwrap();
        let b2 = generate_dice(size, &mut rng2).unwrap();
        assert_eq!(
            b1, b2,
            "Dice board generation must be reproducible with same seed"
        );
    }
}

// -------------------------------------------------------------------------
// Normal Play Policy and Retry Exhaustion Tests
// -------------------------------------------------------------------------

#[test]
fn test_playable_board_guarantees_nonempty_solutions_all_generators() {
    let trie = match load_test_dictionary() {
        Some(t) => t,
        None => return,
    };

    let generators = [
        BoardGenerator::Uniform,
        BoardGenerator::Weighted,
        BoardGenerator::Dice,
    ];

    for gen in generators {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0xFEED_BEEF);
        for size in [4, 5, 6] {
            let (board, solutions) =
                generate_playable_board(size, gen, &trie, SolverStrategy::Trie, &mut rng, 10)
                    .expect("playable board generation succeeds");

            assert_eq!(board.size(), size);
            assert!(
                !solutions.is_empty(),
                "Playable board for {:?} must contain at least one solution",
                gen
            );
        }
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

    let fixed_empty = Board::from_str_tiles(4, "ZZZZZZZZZZZZZZZZ").unwrap();
    let solutions = solve(&fixed_empty, &trie, SolverStrategy::Trie).unwrap();
    assert!(solutions.is_empty());
}
