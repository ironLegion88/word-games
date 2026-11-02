# Wordle Feedback Algorithm Comparative Benchmark Report

- **Run ID:** `run-20261102`
- **Profile Mode:** Full Verification
- **Execution Date:** 2026-11-02 (UTC)
- **Benchmark Seed:** `0x5EED2026`
- **Batches per Stratum:** 30 batches (1000 pairs/batch)

## 1. Executive Summary & Algorithmic Comparison

| Stratum | WL-F1 Marker (ns/op) | WL-F2 Freq (ns/op) | Ratio (WL-F1 / WL-F2) | Position Equality |
|---|---|---|---|---|
| `all_matches` | 10.14 ± 2.45 | 10.86 ± 3.81 | 0.93x | 100% (Identical) |
| `no_matches` | 12.04 ± 3.13 | 13.25 ± 5.27 | 0.91x | 100% (Identical) |
| `exact_partial_mixed` | 14.09 ± 0.43 | 9.79 ± 0.46 | 1.44x | 100% (Identical) |
| `repeated_target_letters` | 16.68 ± 1.01 | 9.25 ± 1.28 | 1.80x | 100% (Identical) |
| `repeated_guess_letters` | 18.40 ± 1.54 | 15.57 ± 7.27 | 1.18x | 100% (Identical) |
| `excess_occurrences` | 21.30 ± 10.75 | 17.02 ± 12.38 | 1.25x | 100% (Identical) |
| `corpus_random_pairs` | 35.32 ± 9.31 | 31.25 ± 8.45 | 1.13x | 100% (Identical) |

## 2. Invariant & Fairness Analysis

1. **Identical Input & Output Contract:** Both algorithms accept identical validated `Word` references and produce full `[LetterFeedback; 5]` arrays.
2. **Zero Heap Allocation:** WL-F1 replaces Mastermind's `Vec<bool>` with stack `[bool; 5]`. WL-F2 uses stack `[u8; 26]`. Neither allocates heap memory inside the evaluation kernel.
3. **100% Agreement:** All test pairs across all strata produce identical positional feedback arrays.
4. **Real Five-Letter Cost:** At length 5, both algorithms execute in the 5–25 nanosecond range. The frequency counter provides predictable $O(L + 26)$ performance across all duplicate letter distributions.
