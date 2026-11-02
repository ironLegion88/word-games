# Spelling Bee Benchmark Report

- **Date:** 2026-11-02
- **Run ID:** `run-20261102`
- **Compiler:** `rustc 1.98.1 (48a229cea 2026-09-01)`
- **Cargo:** `cargo 1.98.1 (797e8a9bc 2026-08-05)`
- **OS:** Windows x86_64
- **Dictionary Path:** `assets/dictionary.txt`
- **Dictionary SHA-256:** `eda7bc8b86a534de8065a7d0cc091dc918d400475061749d05a5711afda12a3b`
- **Words Loaded:** 178,691 words in 85.40 ms
- **Anchor Words Indexed:** 40,859 eligible 7-distinct-letter anchors in 17.31 ms

## Workload Configuration
- Benchmark Mode: `FULL` (30 distinct puzzles generated from seeded RNG `0x5EED_2026`, 10 iterations per puzzle = 300 solves per solver).
- Equality Check: 100% complete sorted solution set equality asserted between SB-S1 and SB-S2 on all 30 puzzles.
- Mean Words per Puzzle: 131.9 words

## Measurement Results

| Algorithm Version | Mean Latency per Solve | Relative Throughput |
|---|---|---|
| **SB-S1 (Exhaustive Filter)** | 12,121.08 µs (12.12 ms) | 1.00x (baseline) |
| **SB-S2 (Trie-Pruned DFS)** | 26.94 µs (0.027 ms) | 449.87x speedup |

## Retained Version Policy
Both solvers remain available in the final crate:
- `spelling_bee::solve_filter` (SB-S1)
- `spelling_bee::solve_trie_pruned` (SB-S2, default)
- Selectable via CLI `--solver filter` or `--solver trie`.
