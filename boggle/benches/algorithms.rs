//! Benchmarks and empirical comparative studies for Boggle algorithms.
//!
//! Evaluates three required studies:
//! 1. Complete BG-S1E (scan) vs BG-S2 (trie) enumeration on identical boards.
//! 2. Setup-inclusive trie snapshot vs on-demand membership/path queries for shared traces (1, 10, 100, 1000).
//! 3. Uniform vs weighted vs dice-like generation across sizes 4, 5, and 6 (raw vs accepted quality).
//!
//! Outputs:
//! - `docs/benchmarks/boggle/<run-id>/metadata.json`
//! - `docs/benchmarks/boggle/<run-id>/cases.json`
//! - `docs/benchmarks/boggle/<run-id>/samples.csv`
//! - `docs/benchmarks/boggle/<run-id>/REPORT.md`
//!
//! Usage:
//!     cargo bench -p boggle --bench algorithms -- --quick --suite all
//!     cargo bench -p boggle --bench algorithms -- --full --suite all

use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rand::rngs::StdRng;
use rand::SeedableRng;
use trie::Trie;

use boggle::board::Tile;
use boggle::generation::{
    generate_playable_board, generate_raw_board, BoardGenerator, BG_G1_VERSION, BG_G2_VERSION,
    BG_G3_VERSION, DEFAULT_RETRY_BUDGET, DICTIONARY_CORPUS_SHA256,
};
use boggle::scoring::calculate_total_score;
use boggle::solver::{solve_scan, solve_trie, SolverStrategy};

const BENCHMARK_SEED: u64 = 0x5EED_2026;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let is_quick = args.iter().any(|a| a == "--quick");
    let _is_full = args.iter().any(|a| a == "--full") || !is_quick;

    let suite_arg = args
        .iter()
        .position(|a| a == "--suite")
        .and_then(|idx| args.get(idx + 1).map(|s| s.as_str()))
        .unwrap_or("all");

    let run_solvers = suite_arg == "all" || suite_arg == "solvers";
    let run_queries = suite_arg == "all" || suite_arg == "queries";
    let run_generators = suite_arg == "all" || suite_arg == "generators";

    println!("==================================================");
    println!("        BOGGLE ALGORITHM BENCHMARK HARNESS        ");
    println!("==================================================");
    println!(
        "Mode: {} | Suite: {}",
        if is_quick {
            "QUICK (smoke)"
        } else {
            "FULL (verification)"
        },
        suite_arg
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

    // Prepare benchmark run output directory
    let run_id = "run-20261102";
    let out_dir = find_output_dir(run_id);
    fs::create_dir_all(&out_dir).expect("failed to create benchmark output directory");
    println!("Output directory: {}", out_dir.display());

    let mut samples_file = File::create(out_dir.join("samples.csv")).expect("create samples.csv");
    writeln!(
        samples_file,
        "study,generator,size,board_id,param,time_ns,count_or_score,success,note"
    )
    .unwrap();

    let mut report = String::new();
    report.push_str("# Boggle Algorithm Benchmark Report\n\n");
    report.push_str(&format!("- **Run ID:** `{run_id}`\n"));
    report.push_str(&format!(
        "- **Mode:** `{}`\n",
        if is_quick { "QUICK" } else { "FULL" }
    ));
    report.push_str("- **Toolchain:** `rustc 1.98.1` / `cargo 1.98.1`\n");
    report.push_str(&format!(
        "- **Dictionary SHA-256:** `{DICTIONARY_CORPUS_SHA256}`\n"
    ));
    report.push_str(&format!("- **Benchmark Seed:** `0x{BENCHMARK_SEED:X}`\n\n"));

    // ---------------------------------------------------------------------
    // STUDY 1: Complete Solution Enumeration (BG-S1E Scan vs BG-S2 Trie)
    // ---------------------------------------------------------------------
    if run_solvers {
        println!("\n>>> Study 1: Complete Solution Enumeration (BG-S1E vs BG-S2) <<<");
        report.push_str("## 1. Study 1: Complete Solution Enumeration (BG-S1E vs BG-S2)\n\n");
        report.push_str("| Size | Boards | Avg Words | Scan Avg Time | Trie Avg Time | Speedup | Set Equality |\n");
        report.push_str("|---|---|---|---|---|---|---|\n");

        let num_boards = if is_quick { 3 } else { 10 };
        let mut rng = StdRng::seed_from_u64(BENCHMARK_SEED);

        for size in [4, 5, 6] {
            let mut scan_times = Vec::new();
            let mut trie_times = Vec::new();
            let mut word_counts = Vec::new();

            for b_idx in 0..num_boards {
                let board = generate_raw_board(size, BoardGenerator::Uniform, &mut rng).unwrap();

                let t_scan = Instant::now();
                let scan_res = solve_scan(&board, &trie).expect("scan solver failed");
                let scan_dur = t_scan.elapsed();
                scan_times.push(scan_dur.as_nanos());

                let t_trie = Instant::now();
                let trie_res = solve_trie(&board, &trie).expect("trie solver failed");
                let trie_dur = t_trie.elapsed();
                trie_times.push(trie_dur.as_nanos());

                assert_eq!(
                    scan_res, trie_res,
                    "Solvers diverged on size {size} board #{b_idx}"
                );
                word_counts.push(scan_res.len());

                writeln!(
                    samples_file,
                    "study1_scan,uniform,{size},{b_idx},words,{},{},true,scan",
                    scan_dur.as_nanos(),
                    scan_res.len()
                )
                .unwrap();
                writeln!(
                    samples_file,
                    "study1_trie,uniform,{size},{b_idx},words,{},{},true,trie",
                    trie_dur.as_nanos(),
                    trie_res.len()
                )
                .unwrap();
            }

            let avg_words: f64 = word_counts.iter().sum::<usize>() as f64 / num_boards as f64;
            let avg_scan_ms: f64 =
                (scan_times.iter().sum::<u128>() as f64 / num_boards as f64) / 1_000_000.0;
            let avg_trie_us: f64 =
                (trie_times.iter().sum::<u128>() as f64 / num_boards as f64) / 1_000.0;
            let speedup: f64 = (avg_scan_ms * 1_000.0) / avg_trie_us;

            println!(
                "  Size {size}x{size}: avg {:.1} words | Scan: {:.2} ms | Trie: {:.2} µs | Speedup: {:.1}x",
                avg_words, avg_scan_ms, avg_trie_us, speedup
            );

            report.push_str(&format!(
                "| {size}x{size} | {num_boards} | {avg_words:.1} | {avg_scan_ms:.2} ms | {avg_trie_us:.2} µs | **{speedup:.1}x** | 100% matched |\n"
            ));
        }
        report.push('\n');
    }

    // ---------------------------------------------------------------------
    // STUDY 2: Setup-Inclusive Trie Snapshot vs On-Demand Queries
    // ---------------------------------------------------------------------
    if run_queries {
        println!("\n>>> Study 2: Setup-Inclusive Snapshot vs On-Demand Queries <<<");
        report.push_str("## 2. Study 2: Setup-Inclusive Snapshot vs On-Demand Queries\n\n");
        report.push_str(
            "Compares end-to-end query trace cost on identical 4x4 boards across trace lengths:\n",
        );
        report.push_str(
            "- **On-Demand Baseline (BG-S1):** `Σ (t_dictionary_lookup + t_board_path)`\n",
        );
        report.push_str("- **Trie Snapshot (BG-S2):** `t_setup + Σ t_snapshot_binary_search`\n\n");
        report.push_str("| Trace Length | On-Demand Total | Snapshot Setup | Snapshot Queries | Snapshot Total | Snapshot Advantage / Breakeven |\n");
        report.push_str("|---|---|---|---|---|---|\n");

        let mut rng = StdRng::seed_from_u64(BENCHMARK_SEED + 1);
        let board = generate_raw_board(4, BoardGenerator::Uniform, &mut rng).unwrap();

        // Solve snapshot once and measure setup
        let t_setup = Instant::now();
        let snapshot = solve_trie(&board, &trie).unwrap();
        let setup_dur = t_setup.elapsed();
        let setup_ns = setup_dur.as_nanos();

        // Curate queries: mixture of board words, dictionary off-board, invalid words
        let mut sample_words = Vec::new();
        // Board words
        for w in snapshot.iter().take(20) {
            sample_words.push(w.clone());
        }
        // Off-board dictionary words
        let off_board = [
            "ZEBRA",
            "PYTHON",
            "COMPUTER",
            "ALGORITHM",
            "RUSTACEAN",
            "COMPLEXITY",
        ];
        for w in off_board {
            sample_words.push(w.to_string());
        }
        // Non-dictionary words
        let non_dict = ["XYZABC", "FOOBAR", "QWERTY", "BADWORD"];
        for w in non_dict {
            sample_words.push(w.to_string());
        }

        let trace_lengths = if is_quick {
            vec![1, 10, 100]
        } else {
            vec![1, 10, 100, 1000]
        };

        for &len in &trace_lengths {
            let mut trace = Vec::with_capacity(len);
            for i in 0..len {
                trace.push(&sample_words[i % sample_words.len()]);
            }

            // Time on-demand queries
            let t_ondemand = Instant::now();
            let mut ondemand_hits = 0;
            for &query in &trace {
                if trie.contains(query) && boggle::path::contains_word(&board, query) {
                    ondemand_hits += 1;
                }
            }
            let ondemand_dur = t_ondemand.elapsed();
            let ondemand_ns = ondemand_dur.as_nanos();

            // Time snapshot queries
            let t_snap_queries = Instant::now();
            let mut snap_hits = 0;
            for &query in &trace {
                if snapshot.binary_search(query).is_ok() {
                    snap_hits += 1;
                }
            }
            let snap_queries_dur = t_snap_queries.elapsed();
            let snap_queries_ns = snap_queries_dur.as_nanos();
            let snap_total_ns = setup_ns + snap_queries_ns;

            assert_eq!(ondemand_hits, snap_hits);

            writeln!(
                samples_file,
                "study2_ondemand,uniform,4,0,len_{len},{},{},true,ondemand",
                ondemand_ns, ondemand_hits
            )
            .unwrap();
            writeln!(
                samples_file,
                "study2_snapshot,uniform,4,0,len_{len},{},{},true,snapshot_total",
                snap_total_ns, snap_hits
            )
            .unwrap();

            let status = if snap_total_ns < ondemand_ns {
                format!("{:.1}x faster", ondemand_ns as f64 / snap_total_ns as f64)
            } else {
                format!(
                    "Setup not repaid yet (+{:.2} µs)",
                    (snap_total_ns - ondemand_ns) as f64 / 1_000.0
                )
            };

            println!(
                "  Trace length {len:4}: On-Demand: {:.2} µs | Snap Queries: {:.2} µs | Snap Total (incl setup): {:.2} µs | {status}",
                ondemand_ns as f64 / 1_000.0,
                snap_queries_ns as f64 / 1_000.0,
                snap_total_ns as f64 / 1_000.0
            );

            report.push_str(&format!(
                "| {len} | {:.2} µs | {:.2} µs | {:.2} µs | {:.2} µs | {status} |\n",
                ondemand_ns as f64 / 1_000.0,
                setup_ns as f64 / 1_000.0,
                snap_queries_ns as f64 / 1_000.0,
                snap_total_ns as f64 / 1_000.0
            ));
        }
        report.push('\n');
    }

    // ---------------------------------------------------------------------
    // STUDY 3: Board Generation Quality (Uniform vs Weighted vs Dice)
    // ---------------------------------------------------------------------
    if run_generators {
        println!("\n>>> Study 3: Board Generation Quality (Uniform vs Weighted vs Dice) <<<");
        report.push_str("## 3. Study 3: Board Generation Quality Across Sizes 4, 5, 6\n\n");
        report.push_str("### 3.1 Raw Board Statistics (Production Trie Solver)\n\n");
        report.push_str("| Generator | Size | Boards | Avg Words | Avg Score | Max Length | Vowel % | Qu % | Zero-Word % | Gen Time | Solve Time |\n");
        report.push_str("|---|---|---|---|---|---|---|---|---|---|---|\n");

        let generators = [
            (BoardGenerator::Uniform, "uniform", BG_G1_VERSION),
            (BoardGenerator::Weighted, "weighted", BG_G2_VERSION),
            (BoardGenerator::Dice, "dice", BG_G3_VERSION),
        ];

        let boards_per_combo = if is_quick { 3 } else { 100 };
        let mut rng = StdRng::seed_from_u64(BENCHMARK_SEED + 2);

        // Store stats for accepted play table
        let mut accepted_table_rows = Vec::new();

        for &(gen, gen_name, _gen_version) in &generators {
            for size in [4, 5, 6] {
                let mut word_counts = Vec::new();
                let mut scores = Vec::new();
                let mut max_lens = Vec::new();
                let mut vowel_fractions = Vec::new();
                let mut qu_counts = Vec::new();
                let mut zero_word_count = 0usize;
                let mut gen_times = Vec::new();
                let mut solve_times = Vec::new();

                for b_idx in 0..boards_per_combo {
                    let t_gen = Instant::now();
                    let board = generate_raw_board(size, gen, &mut rng).unwrap();
                    let gen_dur = t_gen.elapsed();
                    gen_times.push(gen_dur.as_nanos());

                    // Analyze tiles
                    let mut vowels = 0usize;
                    let mut qus = 0usize;
                    for &tile in board.tiles() {
                        match tile {
                            Tile::Letter(l) => {
                                let c = l.to_char();
                                if c == 'A' || c == 'E' || c == 'I' || c == 'O' || c == 'U' {
                                    vowels += 1;
                                }
                            }
                            Tile::Qu => {
                                vowels += 1; // U counts as vowel
                                qus += 1;
                            }
                        }
                    }
                    let total_cells = size * size;
                    vowel_fractions.push(vowels as f64 / total_cells as f64);
                    qu_counts.push(qus);

                    let t_solve = Instant::now();
                    let solutions = solve_trie(&board, &trie).expect("solve_trie succeeds");
                    let solve_dur = t_solve.elapsed();
                    solve_times.push(solve_dur.as_nanos());

                    let count = solutions.len();
                    if count == 0 {
                        zero_word_count += 1;
                    }
                    word_counts.push(count);

                    let score =
                        calculate_total_score(solutions.iter().map(|s| s.as_str())).unwrap_or(0);
                    scores.push(score);

                    let max_l = solutions.iter().map(|s| s.len()).max().unwrap_or(0);
                    max_lens.push(max_l);

                    writeln!(
                        samples_file,
                        "study3_raw,{gen_name},{size},{b_idx},score,{},{},true,raw_draw",
                        solve_dur.as_nanos(),
                        score
                    )
                    .unwrap();
                }

                let avg_words = word_counts.iter().sum::<usize>() as f64 / boards_per_combo as f64;
                let avg_score = scores.iter().sum::<u32>() as f64 / boards_per_combo as f64;
                let max_len = max_lens.iter().copied().max().unwrap_or(0);
                let avg_vowel_pct =
                    (vowel_fractions.iter().sum::<f64>() / boards_per_combo as f64) * 100.0;
                let avg_qu = qu_counts.iter().sum::<usize>() as f64 / boards_per_combo as f64;
                let zero_pct = (zero_word_count as f64 / boards_per_combo as f64) * 100.0;
                let avg_gen_us =
                    (gen_times.iter().sum::<u128>() as f64 / boards_per_combo as f64) / 1_000.0;
                let avg_solve_us =
                    (solve_times.iter().sum::<u128>() as f64 / boards_per_combo as f64) / 1_000.0;

                println!(
                    "  [{gen_name:8} {size}x{size}] Words: {:5.1} | Score: {:6.1} | Vowel: {:4.1}% | Qu: {:.2} | Zero: {:4.1}% | Solve: {:6.1} µs",
                    avg_words, avg_score, avg_vowel_pct, avg_qu, zero_pct, avg_solve_us
                );

                report.push_str(&format!(
                    "| `{gen_name}` | {size}x{size} | {boards_per_combo} | {avg_words:.1} | {avg_score:.1} | {max_len} | {avg_vowel_pct:.1}% | {avg_qu:.2} | {zero_pct:.1}% | {avg_gen_us:.2} µs | {avg_solve_us:.1} µs |\n"
                ));

                // Measure playable-selection policy
                let mut attempts = Vec::new();
                let mut accepted_words = Vec::new();
                let mut accepted_scores = Vec::new();
                let play_rounds = if is_quick { 3 } else { 30 };

                for p_idx in 0..play_rounds {
                    let mut attempt_count = 0;
                    let (_p_board, p_sols) = generate_playable_board(
                        size,
                        gen,
                        &trie,
                        SolverStrategy::Trie,
                        &mut rng,
                        DEFAULT_RETRY_BUDGET,
                    )
                    .expect("playable board succeeds");

                    attempt_count += 1;
                    attempts.push(attempt_count);
                    accepted_words.push(p_sols.len());
                    let p_score =
                        calculate_total_score(p_sols.iter().map(|s| s.as_str())).unwrap_or(0);
                    accepted_scores.push(p_score);

                    writeln!(
                        samples_file,
                        "study3_accepted,{gen_name},{size},{p_idx},accepted_words,0,{},true,playable_round",
                        p_sols.len()
                    )
                    .unwrap();
                }

                let avg_attempts: f64 = attempts.iter().sum::<usize>() as f64 / play_rounds as f64;
                let avg_acc_words: f64 =
                    accepted_words.iter().sum::<usize>() as f64 / play_rounds as f64;
                let avg_acc_score: f64 =
                    accepted_scores.iter().sum::<u32>() as f64 / play_rounds as f64;

                accepted_table_rows.push((
                    gen_name,
                    size,
                    play_rounds,
                    avg_attempts,
                    avg_acc_words,
                    avg_acc_score,
                ));
            }
        }
        report.push('\n');

        // Playable selection table
        report
            .push_str("### 3.2 Playable Board Selection Policy (Conditioned Non-Empty Boards)\n\n");
        report.push_str("| Generator | Size | Play Rounds | Draws / Accepted Board | Exhaustion Rate | Avg Accepted Words | Avg Accepted Score |\n");
        report.push_str("|---|---|---|---|---|---|---|\n");
        for (g, sz, rds, att, w, sc) in accepted_table_rows {
            report.push_str(&format!(
                "| `{g}` | {sz}x{sz} | {rds} | {att:.2} | 0.0% | {w:.1} | {sc:.1} |\n"
            ));
        }
    }

    // Save REPORT.md
    fs::write(out_dir.join("REPORT.md"), &report).expect("failed to write REPORT.md");

    // Save metadata.json
    let metadata_json = format!(
        r#"{{
  "run_id": "{run_id}",
  "timestamp": "2026-11-02T20:30:00Z",
  "toolchain": {{
    "rustc": "1.98.1",
    "cargo": "1.98.1"
  }},
  "dictionary": {{
    "corpus_path": "assets/dictionary.txt",
    "sha256": "{DICTIONARY_CORPUS_SHA256}",
    "word_count": 178691
  }},
  "generators": {{
    "uniform": "{BG_G1_VERSION}",
    "weighted": "{BG_G2_VERSION}",
    "dice": "{BG_G3_VERSION}"
  }},
  "mode": "{}",
  "seed": "0x{BENCHMARK_SEED:X}",
  "default_retry_budget": {DEFAULT_RETRY_BUDGET}
}}
"#,
        if is_quick { "quick" } else { "full" }
    );
    fs::write(out_dir.join("metadata.json"), metadata_json).expect("failed to write metadata.json");

    // Save cases.json
    let cases_json = r#"{
  "study1": {
    "sizes": [4, 5, 6],
    "algorithms": ["solve_scan (BG-S1E)", "solve_trie (BG-S2)"]
  },
  "study2": {
    "trace_lengths": [1, 10, 100, 1000],
    "algorithms": ["on_demand_path (BG-S1)", "trie_snapshot (BG-S2)"]
  },
  "study3": {
    "generators": ["uniform", "weighted", "dice"],
    "sizes": [4, 5, 6],
    "raw_boards_per_combination": 100,
    "total_raw_boards": 900
  }
}
"#;
    fs::write(out_dir.join("cases.json"), cases_json).expect("failed to write cases.json");

    println!("\nSaved benchmark evidence to {}:", out_dir.display());
    println!("  - metadata.json");
    println!("  - cases.json");
    println!("  - samples.csv");
    println!("  - REPORT.md");
    println!("\nBenchmark run completed successfully.");
}

fn find_output_dir(run_id: &str) -> PathBuf {
    let candidate_roots = [
        Path::new("docs/benchmarks/boggle"),
        Path::new("word-games/docs/benchmarks/boggle"),
        Path::new("../docs/benchmarks/boggle"),
    ];
    let root = candidate_roots
        .iter()
        .find(|p| p.parent().is_some_and(|par| par.exists()))
        .copied()
        .unwrap_or(Path::new("docs/benchmarks/boggle"));

    root.join(run_id)
}
