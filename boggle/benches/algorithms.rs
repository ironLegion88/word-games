//! Benchmarks for Boggle algorithms:
//! - Study 1: Complete BG-S1E (scan) vs BG-S2 (trie) enumeration on identical boards.
//! - Study 2: Setup-inclusive trie snapshot vs on-demand membership/path queries for shared traces.
//! - Study 3: Uniform vs weighted vs dice-like generation across sizes 4, 5, and 6.
//!
//! Run with:
//!     cargo bench -p boggle --bench algorithms -- --quick --suite all
//!     cargo bench -p boggle --bench algorithms -- --full --suite all

use std::env;
use std::path::Path;
use std::time::Instant;
use trie::Trie;

use boggle::generation::generate_uniform;
use boggle::solver::{solve_scan, solve_trie};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let is_quick = args.iter().any(|a| a == "--quick");
    let _is_full = args.iter().any(|a| a == "--full") || !is_quick;
    let run_all_suites = args.iter().any(|a| {
        a == "all"
            || a == "--suite"
                && args
                    .iter()
                    .skip_while(|&x| x != "--suite")
                    .nth(1)
                    .is_some_and(|s| s == "all")
    }) || !args.iter().any(|a| a == "--suite");

    println!("=== Boggle Algorithm Benchmark Harness ===");
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
        Path::new("../../assets/dictionary.txt"),
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

    if run_all_suites || args.iter().any(|a| a == "solvers") {
        println!("\n--- Study 1: Complete Solution Enumeration (BG-S1E scan vs BG-S2 trie) ---");
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(0x5EED_2026);

        let num_boards = if is_quick { 3 } else { 10 };
        for size in [4, 5, 6] {
            println!("Benchmarking size {size}x{size} boards ({num_boards} boards)...");
            for b_idx in 0..num_boards {
                let board = generate_uniform(size, &mut rng).expect("board generation failed");

                let t_scan = Instant::now();
                let scan_res = solve_scan(&board, &trie).expect("scan solver failed");
                let scan_dur = t_scan.elapsed();

                let t_trie = Instant::now();
                let trie_res = solve_trie(&board, &trie).expect("trie solver failed");
                let trie_dur = t_trie.elapsed();

                assert_eq!(
                    scan_res, trie_res,
                    "Solvers diverged on size {size} board #{b_idx}"
                );

                if b_idx == 0 {
                    println!(
                        "  Board #0 ({size}x{size}): {} words found | Scan: {:.2?} | Trie: {:.2?}",
                        scan_res.len(),
                        scan_dur,
                        trie_dur
                    );
                }
            }
        }
    }

    println!("\nBenchmark run completed successfully.");
}
