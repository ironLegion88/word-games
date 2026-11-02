use rand::rngs::StdRng;
use rand::SeedableRng;
use spelling_bee::{
    generate_puzzle, AnchorIndex, DifficultyPreset, GenerationConfig, GenerationError,
};
use std::path::Path;
use trie::Trie;

fn load_dict_or_skip() -> Option<Trie> {
    let dict_paths = [
        Path::new("assets/dictionary.txt"),
        Path::new("word-games/assets/dictionary.txt"),
        Path::new("../assets/dictionary.txt"),
    ];
    let dict_path = dict_paths.iter().find(|p| p.exists())?;
    Trie::from_file(dict_path).ok()
}

#[test]
fn test_anchor_index_build_and_properties() {
    let trie = match load_dict_or_skip() {
        Some(t) => t,
        None => return,
    };

    let index = AnchorIndex::build(&trie).expect("build index");
    assert!(!index.is_empty());
    assert!(index.len() > 1000);

    // Every anchor must have exactly 7 distinct letters
    for anchor in index.anchors().iter().take(50) {
        let mut mask = 0u32;
        for b in anchor.bytes() {
            mask |= 1 << (b - b'A');
        }
        assert_eq!(
            mask.count_ones(),
            7,
            "anchor {anchor} does not have 7 distinct letters"
        );
    }
}

#[test]
fn test_seed_reproducibility() {
    let trie = match load_dict_or_skip() {
        Some(t) => t,
        None => return,
    };
    let index = AnchorIndex::build(&trie).expect("build index");

    let seed = 0x1234_5678_u64;
    let config = GenerationConfig::default();

    let mut rng1 = StdRng::seed_from_u64(seed);
    let (p1, s1) = generate_puzzle(&index, &trie, &config, &mut rng1).unwrap();

    let mut rng2 = StdRng::seed_from_u64(seed);
    let (p2, s2) = generate_puzzle(&index, &trie, &config, &mut rng2).unwrap();

    assert_eq!(p1, p2, "same seed must produce identical puzzle");
    assert_eq!(s1, s2, "same seed must produce identical solutions");
}

#[test]
fn test_generated_puzzle_has_pangram() {
    let trie = match load_dict_or_skip() {
        Some(t) => t,
        None => return,
    };
    let index = AnchorIndex::build(&trie).expect("build index");

    let seeds = [0x1111_u64, 0x2222_u64, 0x3333_u64];
    let config = GenerationConfig::default();

    for seed in seeds {
        let mut rng = StdRng::seed_from_u64(seed);
        let (puzzle, solutions) = generate_puzzle(&index, &trie, &config, &mut rng).unwrap();

        let pangram_count = solutions.iter().filter(|w| puzzle.is_pangram(w)).count();
        assert!(
            pangram_count >= 1,
            "generated puzzle {} must contain at least one pangram",
            puzzle.display_letters()
        );
    }
}

#[test]
fn test_difficulty_preset_bounds_and_intersection() {
    let trie = match load_dict_or_skip() {
        Some(t) => t,
        None => return,
    };
    let index = AnchorIndex::build(&trie).expect("build index");

    // Easy preset: 15..=125 words
    let config_easy = GenerationConfig::default()
        .with_preset(DifficultyPreset::Easy)
        .unwrap();

    let mut rng = StdRng::seed_from_u64(0x42_u64);
    let (_p, s_easy) = generate_puzzle(&index, &trie, &config_easy, &mut rng).unwrap();
    assert!(
        s_easy.len() >= 15 && s_easy.len() <= 125,
        "easy puzzle has {} words, expected 15..=125",
        s_easy.len()
    );

    // Empty intersection error
    let invalid_config = GenerationConfig {
        min_words: 200,
        max_words: 300,
        ..Default::default()
    }
    .with_preset(DifficultyPreset::Easy);

    assert!(matches!(
        invalid_config,
        Err(GenerationError::InvalidBounds { .. })
    ));
}

#[test]
fn test_center_override() {
    let trie = match load_dict_or_skip() {
        Some(t) => t,
        None => return,
    };
    let index = AnchorIndex::build(&trie).expect("build index");

    let config = GenerationConfig {
        center_override: Some('E'),
        ..Default::default()
    };

    let mut rng = StdRng::seed_from_u64(0x9999_u64);
    let (puzzle, _sols) = generate_puzzle(&index, &trie, &config, &mut rng).unwrap();
    assert_eq!(puzzle.center().to_char(), 'E');
}

#[test]
fn test_retry_exhaustion_on_impossible_bounds() {
    let trie = match load_dict_or_skip() {
        Some(t) => t,
        None => return,
    };
    let index = AnchorIndex::build(&trie).expect("build index");

    let config = GenerationConfig {
        min_words: 999_999,
        max_words: 999_999,
        max_retries: 5,
        ..Default::default()
    };

    let mut rng = StdRng::seed_from_u64(0x1_u64);
    let err = generate_puzzle(&index, &trie, &config, &mut rng).unwrap_err();
    assert_eq!(err, GenerationError::NoSuitablePuzzle { retries: 5 });
}
