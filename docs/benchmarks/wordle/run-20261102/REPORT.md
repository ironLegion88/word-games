# Wordle Feedback Algorithm Comparative Benchmark Report

- **Run ID:** `run-20261102`
- **Profile Mode:** Full Verification
- **Execution Date:** 2026-11-02 (UTC)
- **Benchmark Seed:** `0x5EED2026`
- **Batches per Stratum:** 30 batches (1000 pairs/batch)

## 1. Executive Summary & Algorithmic Comparison

| Stratum | WL-F1 Marker (ns/op) | WL-F2 Freq (ns/op) | Ratio (WL-F1 / WL-F2) | Position Equality |
|---|---|---|---|---|
| `all_matches` | 6.91 ± 1.79 | 6.46 ± 1.26 | 1.07x | 100% (Identical) |
| `no_matches` | 9.60 ± 0.04 | 9.79 ± 0.60 | 0.98x | 100% (Identical) |
| `exact_partial_mixed` | 13.13 ± 0.76 | 9.27 ± 0.14 | 1.42x | 100% (Identical) |
| `repeated_target_letters` | 15.81 ± 0.46 | 8.86 ± 1.01 | 1.78x | 100% (Identical) |
| `repeated_guess_letters` | 18.42 ± 4.90 | 10.37 ± 3.01 | 1.78x | 100% (Identical) |
| `excess_occurrences` | 17.38 ± 0.61 | 9.25 ± 1.37 | 1.88x | 100% (Identical) |
| `corpus_random_pairs` | 19.23 ± 7.68 | 12.35 ± 6.35 | 1.56x | 100% (Identical) |

## 2. Invariant & Fairness Analysis

1. **Identical Input & Output Contract:** Both algorithms accept identical validated `Word` references and produce full `[LetterFeedback; 5]` arrays.
2. **Zero Heap Allocation:** WL-F1 replaces Mastermind's `Vec<bool>` with stack `[bool; 5]`. WL-F2 uses stack `[u8; 26]`. Neither allocates heap memory inside the evaluation kernel.
3. **100% Agreement:** All test pairs across all strata produce identical positional feedback arrays.
4. **Real Five-Letter Cost:** At length 5, both algorithms execute in the 5–25 nanosecond range. The frequency counter provides predictable $O(L + 26)$ performance across all duplicate letter distributions.
