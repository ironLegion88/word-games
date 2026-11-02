use rand::rngs::StdRng;
use rand::SeedableRng;
use spelling_bee::{solve_trie_pruned, AnchorIndex, SpellingBeePuzzle};
use std::collections::BTreeSet;
use std::path::Path;
use trie::{Letter, Trie};

#[test]
fn test_calibrate_difficulty_presets() {
    let dict_paths = [
        Path::new("assets/dictionary.txt"),
        Path::new("word-games/assets/dictionary.txt"),
        Path::new("../assets/dictionary.txt"),
    ];
    let dict_path = match dict_paths.iter().find(|p| p.exists()) {
        Some(p) => p,
        None => return,
    };

    let trie = Trie::from_file(dict_path).expect("load dict");
    let anchor_index = AnchorIndex::build(&trie).expect("build anchor index");

    let mut rng = StdRng::seed_from_u64(0xCA11_B2A7_2026);
    let mut sampled_letter_sets = BTreeSet::new();

    while sampled_letter_sets.len() < 256 {
        let anchor = anchor_index.choose_anchor(&mut rng);
        let mut letters = BTreeSet::new();
        for b in anchor.bytes() {
            if let Some(l) = Letter::from_char(b as char) {
                letters.insert(l.to_char());
            }
        }
        if letters.len() == 7 {
            let lstr: String = letters.into_iter().collect();
            sampled_letter_sets.insert(lstr);
        }
    }

    let mut all_solution_counts = Vec::new();

    for letter_str in &sampled_letter_sets {
        let chars: Vec<char> = letter_str.chars().collect();
        for &center in &chars {
            let outers: Vec<char> = chars.iter().copied().filter(|&c| c != center).collect();
            let puzzle = SpellingBeePuzzle::new(center, &outers).unwrap();
            let sols = solve_trie_pruned(&puzzle, &trie);
            all_solution_counts.push(sols.len());
        }
    }

    all_solution_counts.sort_unstable();
    let n = all_solution_counts.len();
    assert_eq!(n, 256 * 7); // 1792 total puzzle configurations

    let p33 = all_solution_counts[n / 3];
    let p67 = all_solution_counts[2 * n / 3];
    let min_c = all_solution_counts[0];
    let max_c = all_solution_counts[n - 1];

    println!(
        "Calibration result on 256 distinct sets (1792 puzzles): min={}, p33={}, p67={}, max={}",
        min_c, p33, p67, max_c
    );
}
