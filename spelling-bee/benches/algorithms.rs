//! Benchmarks for Spelling Bee algorithms: SB-S1 (filtering) vs SB-S2 (trie-pruned).
//!
//! Run with:
//!     cargo bench -p spelling-bee --bench algorithms -- --quick
//!     cargo bench -p spelling-bee --bench algorithms -- --full

use spelling_bee::{
    generate_puzzle, solve_filter, solve_trie_pruned, AnchorIndex, GenerationConfig,
};
use std::env;
use std::hint::black_box;
use std::path::Path;
use std::time::Instant;
use trie::Trie;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let is_quick = args.iter().any(|a| a == "--quick");
    let _is_full = args.iter().any(|a| a == "--full") || !is_quick;

    println!("=== Spelling Bee Benchmark Harness ===");
    println!(
        "Mode: {}",
        if is_quick {
            "QUICK (smoke)"
        } else {
            "FULL (verification)"
        }
    );

    // Locate dictionary
    let dict_paths = [
        Path::new("assets/dictionary.txt"),
        Path::new("word-games/assets/dictionary.txt"),
        Path::new("../assets/dictionary.txt"),
    ];
    let dict_path = dict_paths
        .iter()
        .find(|p| p.exists())
        .expect("dictionary.txt must exist");

    println!("Loading dictionary from '{}'...", dict_path.display());
    let t0 = Instant::now();
    let trie = Trie::from_file(dict_path).expect("failed to load dictionary");
    let load_dur = t0.elapsed();
    println!("Loaded {} words in {:.2?}", trie.len(), load_dur);

    // Build anchor index
    let t_anchor = Instant::now();
    let anchor_index = AnchorIndex::build(&trie).expect("failed to build anchor index");
    let anchor_dur = t_anchor.elapsed();
    println!(
        "Built anchor index ({} eligible anchors) in {:.2?}",
        anchor_index.len(),
        anchor_dur
    );

    // Sample puzzles to benchmark
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(0x5EED_2026);

    let num_puzzles = if is_quick { 5 } else { 30 };
    let iterations_per_puzzle = if is_quick { 1 } else { 10 };

    println!("\nGenerating {num_puzzles} distinct benchmark puzzles...");
    let mut puzzles = Vec::new();

    let config = GenerationConfig {
        min_words: 15,
        max_words: 250,
        ..Default::default()
    };

    while puzzles.len() < num_puzzles {
        if let Ok((p, _)) = generate_puzzle(&anchor_index, &trie, &config, &mut rng) {
            puzzles.push(p);
        }
    }

    println!(
        "Benchmarking {} puzzles (iterations per puzzle: {})...",
        puzzles.len(),
        iterations_per_puzzle
    );

    let mut filter_total_ns = 0u128;
    let mut trie_total_ns = 0u128;
    let mut words_total = 0usize;

    for (idx, puzzle) in puzzles.iter().enumerate() {
        // Assert complete set equality on each benchmarked puzzle
        let res_filter = solve_filter(puzzle, &trie);
        let res_trie = solve_trie_pruned(puzzle, &trie);
        assert_eq!(
            res_filter,
            res_trie,
            "Filter and Trie solvers produced divergent results for puzzle {}",
            puzzle.display_letters()
        );

        words_total += res_filter.len();

        // Warm up
        black_box(solve_filter(puzzle, &trie));
        black_box(solve_trie_pruned(puzzle, &trie));

        // Time Filter (SB-S1)
        let start = Instant::now();
        for _ in 0..iterations_per_puzzle {
            black_box(solve_filter(puzzle, &trie));
        }
        filter_total_ns += start.elapsed().as_nanos();

        // Time Trie Pruned (SB-S2)
        let start = Instant::now();
        for _ in 0..iterations_per_puzzle {
            black_box(solve_trie_pruned(puzzle, &trie));
        }
        trie_total_ns += start.elapsed().as_nanos();

        if is_quick && idx < 3 {
            println!(
                "  Puzzle {}: [{}] => {} words",
                idx + 1,
                puzzle.display_letters(),
                res_filter.len()
            );
        }
    }

    let total_evals = (num_puzzles * iterations_per_puzzle) as f64;
    let filter_avg_us = (filter_total_ns as f64 / total_evals) / 1000.0;
    let trie_avg_us = (trie_total_ns as f64 / total_evals) / 1000.0;
    let avg_words = words_total as f64 / num_puzzles as f64;

    println!("\n=== Benchmark Results ===");
    println!("Puzzles evaluated:   {}", num_puzzles);
    println!("Average words/puzzle: {:.1}", avg_words);
    println!("SB-S1 (Filter):      {:.2} µs / solve", filter_avg_us);
    println!("SB-S2 (Trie Pruned): {:.2} µs / solve", trie_avg_us);

    if trie_avg_us > 0.0 {
        let speedup = filter_avg_us / trie_avg_us;
        println!("Speedup (S1 / S2):   {:.2}x", speedup);
    }
    println!("=========================\n");
}
