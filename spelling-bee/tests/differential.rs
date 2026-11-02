use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use spelling_bee::{solve_filter, solve_trie_pruned, AnchorIndex, SpellingBeePuzzle};
use std::collections::BTreeSet;
use std::path::Path;
use trie::{Letter, Trie};

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
fn test_differential_tiny_fixture() {
    let mut trie = Trie::new();
    let words = [
        "AROMATIC",
        "CITRIC",
        "CRITIC",
        "ATOM",
        "ATOMIC",
        "ACAI",
        "MICRO",
        "CAT",
        "DOG",
        "ROCOCO",
        "TRACTOR",
        "TRACT",
        "ATTRACT",
        "ATTRACTOR",
        "TOCCATA",
        "MACRO",
        "COMMA",
    ];
    for &w in &words {
        trie.insert(w).unwrap();
    }

    let stats_before = trie.stats();
    let puzzle = SpellingBeePuzzle::from_raw("CAIMORT").unwrap();

    let res_filter = solve_filter(&puzzle, &trie);
    let res_pruned = solve_trie_pruned(&puzzle, &trie);

    assert_eq!(
        res_filter, res_pruned,
        "SB-S1 and SB-S2 must produce identical complete sorted solution vectors"
    );

    let stats_after = trie.stats();
    assert_eq!(
        stats_before, stats_after,
        "trie must remain completely unchanged"
    );
}

#[test]
fn test_differential_zero_solutions() {
    let mut trie = Trie::new();
    trie.insert("ZEBRA").unwrap();
    trie.insert("QUEEN").unwrap();
    trie.insert("XYLOPHONE").unwrap();

    let puzzle = SpellingBeePuzzle::from_raw("CAIMORT").unwrap();

    let res_filter = solve_filter(&puzzle, &trie);
    let res_pruned = solve_trie_pruned(&puzzle, &trie);

    assert!(res_filter.is_empty());
    assert_eq!(res_filter, res_pruned);
}

#[test]
fn test_differential_seeded_random_letter_sets() {
    let mut trie = Trie::new();
    let words = [
        "CARE", "CART", "CRATE", "TRACE", "RACE", "ACE", "ACRE", "CATER", "REACT", "TEAR", "RATE",
        "STARE", "CASTER", "CRATES", "TRACES", "REACTION", "CREATION",
    ];
    for &w in &words {
        trie.insert(w).unwrap();
    }

    let seeds = [0x1111_u64, 0x2222_u64, 0x3333_u64, 0x4444_u64, 0x5555_u64];
    for seed in seeds {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut letters = BTreeSet::new();
        while letters.len() < 7 {
            let ch = (b'A' + rng.gen_range(0..26)) as char;
            letters.insert(ch);
        }

        let lvec: Vec<char> = letters.into_iter().collect();
        let center = lvec[0];
        let puzzle = SpellingBeePuzzle::new(center, &lvec[1..]).unwrap();

        let s1 = solve_filter(&puzzle, &trie);
        let s2 = solve_trie_pruned(&puzzle, &trie);
        assert_eq!(s1, s2, "seed {seed} failed differential comparison");
    }
}

#[test]
fn test_differential_corpus_all_seven_centers() {
    let trie = match load_dict_or_skip() {
        Some(t) => t,
        None => return,
    };

    let anchor_index = AnchorIndex::build(&trie).expect("anchor index build");
    let mut rng = StdRng::seed_from_u64(0x05EE_DBEE);

    // Pick 5 distinct anchor words from the dictionary
    for _ in 0..5 {
        let anchor = anchor_index.choose_anchor(&mut rng);
        let mut distinct = BTreeSet::new();
        for b in anchor.bytes() {
            if let Some(l) = Letter::from_char(b as char) {
                distinct.insert(l.to_char());
            }
        }
        assert_eq!(distinct.len(), 7);

        let dvec: Vec<char> = distinct.into_iter().collect();

        // Test all 7 possible centers for this 7-letter set!
        for &center in &dvec {
            let outer: Vec<char> = dvec.iter().copied().filter(|&c| c != center).collect();
            let puzzle = SpellingBeePuzzle::new(center, &outer).unwrap();

            let stats_before = trie.stats();
            let s1 = solve_filter(&puzzle, &trie);
            let s2 = solve_trie_pruned(&puzzle, &trie);
            let stats_after = trie.stats();

            assert_eq!(
                s1, s2,
                "divergence on anchor '{anchor}' with center '{center}'"
            );
            assert_eq!(stats_before, stats_after, "trie mutated during solving");
            assert!(
                !s1.is_empty(),
                "anchor word '{anchor}' must yield at least one solution"
            );
        }
    }
}
