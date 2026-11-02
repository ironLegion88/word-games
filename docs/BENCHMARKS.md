# Algorithm Versions and Reproducible Benchmarks

**Status — 2026-11-02:** Protocol and planned workloads only. The new games, alternative algorithms, calibration outputs and game measurements are **NOT IMPLEMENTED / NOT RUN** in this documentation update. Existing checkpoint-one observations remain in [TESTING.md](TESTING.md), with their original provenance. Do not promote its unsupported sub-20-ns lookup statement into a baseline.

## 1. Purpose and retained versions

Build correctness first, then compare implementations under the same inputs. Keep reference and optimized algorithms available at the final commit, even when a baseline is slower or an optimization produces no significant benefit. Version IDs identify algorithms/configurations, not crate major versions.

| Track | Baseline | Alternative | Default after required verification |
|---|---|---|---|
| Bee solutions | SB-S1: full dictionary filtering | SB-S2: allowed-edge trie traversal | SB-S2 |
| Bee puzzle controls | SB-P1: seeded pangram anchor | SB-P2: anchored generation with count bounds and calibrated presets | Anchored generation with requested controls |
| Boggle complete solution sets | BG-S1E: dictionary scan + independent path search | BG-S2: trie-guided board DFS | BG-S2 |
| Boggle query traces | BG-S1: membership + per-word path DFS | BG-S2 setup + snapshot binary search | Snapshot gameplay; report amortization |
| Boggle generation | BG-G1: uniform | BG-G2: weighted; BG-G3: synthetic dice-like | BG-G1 remains selectable/default |
| Wordle feedback | WL-F1: Mastermind-derived marker matching | WL-F2: 26-letter remaining frequencies | WL-F2 |

An alternative must have an independent implementation of the work being compared. Reusing common normalization, rules, scoring or output formatting is appropriate; implementing the reference solver by calling the production solver is not.

## 2. Harness and output contract

Use stable Rust benchmark executables under each engine's `benches/algorithms.rs`, configured with `harness = false`. Do not require nightly `#[bench]`. Minimize dependencies; a small `Instant`/`black_box` driver is adequate when its limitations are documented. A well-justified existing framework can be retained if already present, with matching commands updated everywhere.

The implementation target commands are:

```bash
cargo bench -p spelling-bee --bench algorithms -- --quick
cargo bench -p spelling-bee --bench algorithms -- --full
cargo bench -p boggle --bench algorithms -- --quick --suite all
cargo bench -p boggle --bench algorithms -- --full --suite all
cargo bench -p wordle --bench algorithms -- --quick
cargo bench -p wordle --bench algorithms -- --full
```

These commands become runnable only after the relevant branch implements the harness. `--quick` is a deterministic smoke test; `--full` is the required pre-merge comparison. Document any actual CLI adjustment consistently rather than leaving broken commands in the README or review instructions.

Write evidence under `docs/benchmarks/<game>/<run-id>/`:

- `metadata.json`: reviewed code commit, dirty-state flag, UTC timestamp, rustc/Cargo versions, target, OS/CPU/RAM where available, build profile/flags, RNG/lockfile versions, fixture/data hashes, algorithm and data-policy versions, seeds, warm-up/repetition/batch settings, limits and exclusions.
- `cases.json` or CSV: exact puzzles, boards, word pairs or guess traces used by every competing algorithm. Include generation/configuration IDs, not merely a seed.
- `samples.csv`: per-case/per-repeat time, sample/batch count, result size/checksum, expansion/retry counters where meaningful, success/failure and failure reason. Distinguish seconds/nanoseconds explicitly.
- `REPORT.md`: methods, correctness gate, summaries and uncertainty, setup versus repeated work, memory measurement method, failures and limitations, interpretation, and exact reproduction commands.

Commit concise raw datasets and reports needed to reproduce the claims, together with deterministic input generation. Do not commit target binaries, huge profiler dumps or user-specific paths. A script may regenerate large optional datasets, but record every input/hash and keep the primary comparison's raw numeric samples.

## 3. Measurement rules

1. Verify complete set/array equality **before** timing a case. A mismatch is a correctness failure, not a faster result. Save the failing case and stop promotion of that variant.
2. Construct/materialize shared fixtures and query traces before comparing algorithms. Separate dictionary parsing, loading, generation/indexing, solving and per-guess work; also measure their appropriate combined cost.
3. Use optimized builds. Warm up each variant (initial default: 5 batches), then measure repeated samples (full default: at least 30 independent timing samples for cheap kernels). Batch very small operations enough to exceed clock granularity; keep batch size in metadata. Black-box inputs **and** outputs or consume checksums so unused work is not eliminated.
4. Alternate or deterministically shuffle variant order per repetition. Use the same seed/case manifest and avoid random draws or console printing in the timed inner loop. Label hot versus cold/cache-inclusive measurements; do not call repeated hot lookup numbers universal latency.
5. Summarize median, distribution/quantiles and spread, with sample sizes. Compute per-case paired differences/ratios before aggregation and include uncertainty where practical. Avoid overstated p95 claims on tiny sample sets. Tiny score-kernel differences at five letters may be noise; say so.
6. Measure peak RSS only with actual process/platform facilities. Label Rust object/payload/capacity estimates separately; do not present them as peak process memory. Report shared dictionary cost once, per-game added memory separately, and unavailable metrics as `NOT MEASURED`.
7. Preserve limits, timeouts, errors, zero-solution cases and retry exhaustion in denominators. A resource-limit error is not an empty solution set. If a full baseline is too expensive, use a predeclared common subset/budget, report the excluded coverage and failure rates, and retain an exact small/representative comparison; do not silently shrink only the slower variant's workload.
8. Record machine/environment constraints. CI/VM load and CPU thermal conditions may add variance. Measurements support conclusions only about the recorded workload/environment. No benchmark requires the optimized version to win.

Quick-mode defaults can use 3 timing samples, tiny exact fixtures and a small corpus sample. Full runs should normally include at least 30 matched timed cases or samples per cheap solver/kernel comparison and the generator workload below. If this is infeasible, document a common reduced workload, why it is adequate/limited, and have the independent reviewer assess it; do not invent missing results or waive a required comparative run.

## 4. Spelling Bee workloads

- Tiny hand-enumerated fixtures: shared prefixes, repeated letters, no center, no solutions, one pangram, multiple pangrams and all-center variants.
- Deterministic random seven-letter sets, including low-solution sets; plus at least 30 anchored full-dictionary puzzle configurations spread across observed solution-count bands. Compare SB-S1 and SB-S2 on **identical letters and centers** and assert full sorted-set equality.
- Record visited dictionary words/nodes or equivalent instrumented work counts, solve latency, solution count, length histogram, total possible score and memory where measured. Keep instrumentation outside comparative timing or enable it equally and disclose the overhead.
- Measure anchor-index construction separately from per-puzzle solving. Compare unconstrained SB-P1 and count-controlled SB-P2 for selection/solve cost, retries, acceptance rate, repeated letter-set bias and reproducibility. Their output puzzles differ by design, so this is a generation-policy study, not a solver speedup test.
- Calibrate presets only after the trie solver is correct, using the design's sampled distinct sets and all seven centers. Save calibration cases, observed quantiles, tie handling and the resulting integer bands. Do not call these human-validated difficulty levels.

## 5. Boggle solver workloads

Use all board sizes, all generation modes after they exist, explicit diagonal/no-reuse/`Qu` fixtures, low-vowel boards, repeated-letter boards and a custom dictionary word longer than 15 letters. Verify BG-S1E versus BG-S2 on complete sets; for pathological exhaustive scans, include exact tiny dictionaries and budgeted representative full-corpus cases with honest failure accounting.

Measure two distinct experiments:

1. **Complete enumeration:** BG-S1E and BG-S2 both generate the same entire snapshot. Include dictionary filtering/board search that each actually performs, separate shared dictionary loading, and record search expansions and memory.
2. **Amortized queries:** identical trace lengths such as 1, 10, 100 and 1,000, with valid words, unknown words, dictionary-but-off-board words, repeats and malformed-input cases separately labelled. Compare `T_trie_setup + Σ T_snapshot_query` against `Σ (T_dictionary_lookup + T_BG-S1_path)` without charging the on-demand baseline for an unused complete snapshot. Report the number of queries at which any setup cost is repaid, or state that it was not repaid.

The on-demand experiment evaluates a strategy/API; the shipped session still owns its complete solution snapshot. Do not change the ownership contract merely to make the benchmark easier. Partition malformed-input rejection costs from search-only kernel timings.

## 6. Boggle board-generation quality workloads

Full-mode starting workload: 100 raw boards per `(generator, size)` combination, 3 generators × 3 sizes = 900 boards, using a saved seed/configuration manifest. Quick mode can use 3 per combination. Larger optional studies may increase this to 1,000 per combination, with the actual workload stated.

For every raw draw, use the **same production solver** and record:

| Metric | Meaning |
|---|---|
| Generation and solve time | Separate distribution cost from evaluation cost |
| Valid distinct word count | Complete solution-set size |
| Total reachable score | Same scoring table for every generator |
| Word-length distribution and longest word | Include spelled `QU` length consistently |
| Vowel fraction and `Qu` count | Diagnostic board properties, not quality guarantees |
| Zero-word rate | Raw generator failure to yield any playable word |
| Search failures/limits | Separate from valid zero-word boards |
| Seed/data hashes | Reconstruct every sampled board |

Then evaluate the playable-board retry policy with a fixed documented candidate budget. Report accepted-board metrics, attempted draws per accepted board, exhaustion probability and total setup cost. Conditioning on nonempty boards changes the distribution; do not publish only accepted boards as if they described raw generation.

BG-G2 weights derive from dictionary spellings with add-one smoothing; BG-G3 synthetic dice derive from these same versioned weights. Save raw counts, derived weights, dice data, derivation method and hashes. Dice tests verify one face per die, six faces per die, correct die count, valid tiles and a shuffled board assignment. Never identify these generated tables as official commercial dice.

## 7. Wordle workloads

- Exhaustive 59,049 pairs over alphabet `{A,B,C}` and length five, with complete positional equality across WL-F1, WL-F2 and an independent oracle. This is correctness work, not a fabricated timing result.
- At least 30 measured batches from fixed full-alphabet random pairs and real five-letter dictionary/answer pairs. Stratify no matches, all matches, exact/partial mixtures, repeated target letters, repeated guess letters and excess occurrences.
- Time both feedback functions with the same validated inputs and output consumption. Report allocations for the marker version only if measured or established directly by inspected implementation; distinguish allocation count reasoning from process-memory measurement.
- Keep five-letter results primary. Any variable-length internal-kernel experiment must be labelled a generalized stress test, not the shipped Wordle game or evidence of a large five-letter win.
- Optional auxiliary comparison: existing-trie membership versus binary search of the owned five-letter snapshot, including extraction/setup cost and realistic repeated-query traces. Do not change the established gameplay ownership just for a throughput claim.

## 8. Deferred trie storage track

The original comparison ladder remains documented but is not activated by the game prompts:

| Version family | Implementation | Shared workloads |
|---|---|---|
| T0 | Existing fixed-index arena with pruning/reuse/compaction | Corpus loading, successful/failed lookup, insert/delete churn, compaction, game traversal |
| TA | Boxed 26-child ownership tree | Same words, operation streams and game outputs |
| TD | Adaptive sparse/`SmallVec` representation | Same correctness and traversal workloads; report capacity/allocation overhead |
| TM-H / TM-B | `HashMap` / `BTreeMap` children | Same cases, including hashing/tree bookkeeping and memory |

An eventual implementation must preserve a comparable query/cursor boundary, separately test deletion/reclamation semantics, and measure actual memory plus load/search performance. Do not assume one is smaller or faster from asymptotic notation alone. This track requires a separate later task/branch; no game checkpoint depends on implementing it.

## 9. Independent-review evidence

At review time, the subagent must inspect the harness and raw records, check that metadata names the reviewed code/data versions, verify equal workloads and correctness gates, rerun benchmark smoke and representative comparisons, and flag omitted failures or unsupported claims. Reports must distinguish what the reviewer executed from what it inspected in the author's saved full run. Missing or blocked mandatory comparative evidence prevents checkpoint completion.
