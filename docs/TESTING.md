# Testing, Invariants and Verification Record

**Status: ASSIGNMENT 1 VERIFIED in the supplied checkpoint record; new games NOT IMPLEMENTED.**

**2026-11-02 documentation revision:** The dated evidence below is preserved from the uploaded repository and was not rerun during this documentation-only update. Sections 7 onward define planned game verification and must not be read as executed results.
All automated unit, property-like invariant, seeded reference-model, real corpus regression, and end-to-end CLI subprocess tests are fully implemented and passing with zero warnings under stable Rust.

---

## 1. Checkpoint 1 Verification Evidence Log

| Field | Measured / Verified Observation |
|---|---|
| **Checkpoint** | Assignment 1 (`trie` library & `trie-cli` binary) |
| **Verification Date** | 2026-10-21 |
| **Rust Version** | `rustc 1.98.1 (48a229cea 2026-09-01)` |
| **Cargo Version** | `cargo 1.98.1 (797e8a9bc 2026-08-05)` |
| **Target Platform** | Windows x86_64 |
| **Workspace Manifests** | Root `Cargo.toml`, `trie/Cargo.toml`, `trie-cli/Cargo.toml` |
| **Formatting Status** | `cargo fmt --all -- --check` exited `0` (clean) |
| **Clippy Linter Status** | `cargo clippy --workspace --all-targets -- -D warnings` exited `0` (zero warnings) |
| **Debug Test Suite** | `cargo test --workspace` exited `0` (**61 passed**, 0 failed, 0 skipped) |
| **Release Test Suite** | `cargo test --workspace --release` exited `0` (**61 passed**, 0 failed, 0 skipped) |
| **Release Build** | `cargo build --workspace --release` exited `0` |
| **Doctests** | `trie` doctest passed (1 passed, 0 failed) |
| **Random Test Seeds** | `0x00C0_FFEE` (1,000 ops), `0x5EED_2026` (10,000 ops), `0x0BAD_C0DE` (20,000 ops) — all matched `HashSet<String>` |
| **Corpus Fixture Verification** | `assets/dictionary.txt`: 178,691 unique words, 395,185 live nodes, min length 2, max length 15 |
| **CLI Verification** | Interactive subprocess with piped stdin verified all commands and error handling |

---

## 2. Requirements Acceptance Matrix (`Word Games.pdf`)

| ID | Source Requirement | Implemented Functionality | Verification Evidence | Status |
|---|---|---|---|---|
| **WG-01** | Create an empty trie | `Trie::new()`, `Trie::default()`, CLI `new` command | `tests/arena.rs::fresh_trie_arena_invariants`, `tests/operations.rs::empty_trie_queries`, `tests/cli.rs::cli_file_operations_and_filtering` | **VERIFIED** |
| **WG-02** | Boolean lookup returning boolean | `Trie::contains(&str) -> bool`, `Trie::try_contains(&str) -> Result<bool, WordError>` | `tests/operations.rs::single_word_lifecycle`, `tests/operations.rs::contains_does_not_mutate`, `tests/cli.rs::cli_interactive_basic_operations` | **VERIFIED** |
| **WG-03** | Insert single word one at a time | `Trie::insert(&mut self, &str) -> Result<bool, WordError>` | `tests/operations.rs::single_word_lifecycle`, `tests/operations.rs::overlapping_branches_and_subprefixes`, `tests/randomized.rs` | **VERIFIED** |
| **WG-04** | Build from one-word-per-line file | `Trie::from_reader`, `Trie::from_file`, CLI `load` | `tests/loading.rs::format_one_word_per_line`, `tests/regression.rs::full_dictionary_corpus_regression` (178,691 words) | **VERIFIED** |
| **WG-05** | Build from comma/whitespace file | `parse_tokens`, `from_reader`, `extend_from_reader`, CLI `add-file` | `tests/loading.rs::format_comma_only`, `format_mixed_delimiters_and_whitespace`, `format_empty_and_repeated_delimiters`, `format_eof_without_final_newline` | **VERIFIED** |
| **WG-06** | Lowercase-to-uppercase routine | `letter::normalize_word(&str) -> Result<Cow<str>, WordError>`, CLI `normalize` | `tests/letter.rs::case_lower`, `case_mixed`, `case_upper_identity`, `normalization_idempotent`, `tests/cli.rs::cli_interactive_basic_operations` | **VERIFIED** |
| **WG-07** | Delete one word with pruning | `Trie::remove(&mut self, &str) -> Result<bool, WordError>` | `tests/deletion.rs::delete_leaf_prunes_branch`, `delete_word_that_is_prefix_of_another`, `delete_child_while_keeping_siblings`, `reinsert_deleted_word_reuses_free_slots` | **VERIFIED** |
| **WG-08** | Delete a list of words | `Trie::remove_words`, `Trie::remove_from_file`, CLI `remove-file` | `tests/loading.rs::batch_remove_words_report`, `failure_atomicity_on_batch_delete`, `tests/cli.rs::cli_interactive_basic_operations` | **VERIFIED** |
| **WG-09** | Return words in trie from file | `Trie::filter_present_from_reader`, `filter_present_from_file`, CLI `present` | `tests/loading.rs::filtering_present_and_absent_preserves_order_and_duplicates`, `tests/cli.rs::cli_file_operations_and_filtering` | **VERIFIED** |
| **WG-10** | Return words not in trie from file | `Trie::filter_absent_from_reader`, `filter_absent_from_file`, CLI `absent` | `tests/loading.rs::filtering_present_and_absent_preserves_order_and_duplicates`, `tests/cli.rs::cli_file_operations_and_filtering` | **VERIFIED** |
| **WG-11** | Extensive tests, docs, Cargo usage | Unit, integration, seeded reference model, real corpus, `README.md`, `TESTING.md` | 61 automated tests, `cargo test --workspace --release`, full invariant checker, clean clippy & formatting | **VERIFIED** |

---

## 3. Test Suites & Observed Outputs

### 3.1 Unit & Invariant Tests (`trie/src/lib.rs` & `tests/arena.rs`)
- **`allocate_and_reuse_free_slots`**: Allocates nodes, frees a node to the free list, and asserts that the subsequent allocation reuses the recycled ID without increasing total vector slots.
- **`cannot_free_root`**: Confirms that an attempt to free slot 0 panics with `"cannot free the root node"`.
- **`invariant_detects_slot_leak`**: Artificially adds an unreachable, un-freed node to the arena; verifies `check_invariants()` reports `"arena slot leak"`.
- **`invariant_detects_count_mismatch`**: Artificially corrupts `word_count`; verifies `check_invariants()` reports `"word_count mismatch"`.
- **`fresh_trie_arena_invariants`**: Confirms empty trie has 1 live node (root), 0 words, 0 free slots, and passes all invariants.
- **`node_struct_size_and_sentinel`**: Verifies `size_of::<Node>() == 112` bytes on 64-bit target, `children` initialized to `NO_CHILD = u32::MAX`.

### 3.2 Letter Validation & Normalization (`tests/letter.rs`)
- **`letter_a_through_z`**: Loops through all 26 letters; asserts `index() == 0..26`, `to_char() == 'A'..='Z'`, `from_index()` round-trips.
- **`letter_invalid_ascii`**: Verifies spaces, tabs, newlines, digits, punctuation (`-`, `'`, `;`, `.`, `,`) return `None`.
- **`letter_unicode`**: Verifies Unicode non-ASCII letters (`é`, `ñ`, `ü`, `ß`, `中`, `🦀`) return `None`.
- **`case_upper_identity`**: Verifies already-uppercase strings return `Cow::Borrowed` with zero allocation.
- **`case_lower` & `case_mixed`**: Verifies conversion to uppercase ASCII string.
- **`empty_rejected`**: Verifies `normalize_word("") == Err(WordError::EmptyWord)`.
- **`table_driven_malformed_tokens_rejected`**: Table-driven test asserting that invalid characters report the exact offending character and byte index.
- **`normalization_idempotent`**: Verifies `normalize(normalize(w)) == normalize(w)`.

### 3.3 Deletion, Physical Pruning & Compaction (`tests/deletion.rs`)
- **`delete_absent_word`**: Deleting absent words or proper prefixes returns `Ok(false)` with zero mutations.
- **`delete_leaf_prunes_branch`**: Deleting `CART` while `CAR` is present removes node `T` from arena, increments `free_slots`, decrements `live_nodes` by 1.
- **`delete_word_that_is_prefix_of_another`**: Deleting `CAR` while `CART` is present unsets `is_word` on `R` but prunes zero nodes (since all nodes are live for `CART`).
- **`delete_child_while_keeping_siblings`**: Deleting `CAT` while `CAR` is present prunes `T`, keeping `C`, `A`, and `R` intact.
- **`delete_every_word_retains_root`**: Removing all words resets trie to 0 words, preserves root at slot 0, and places all other nodes on `free_list`.
- **`reinsert_deleted_word_reuses_free_slots`**: Deleting `TEST` (4 freed slots) and reinserting reuses all 4 slots without growing arena vector.
- **`compaction_remaps_indices_and_shrinks`**: Removes branches creating fragmentation, forces `compact()`, and verifies all child IDs remapped, `free_list` emptied, `total_slots == live_nodes`, and all word queries and iteration remain identical.
- **`churn_1000_operations_stress_test`**: Churns 1,000+ insertions, lookups, and deletions over 100 cycles with intermittent compaction; all invariants pass at every cycle.

### 3.4 Parsing, Loading & Filtering (`tests/loading.rs`)
- **`format_one_word_per_line`**: Parses `DOG\nCAT\nBIRD`.
- **`format_comma_only`**: Parses `DOG,CAT,BIRD`.
- **`format_mixed_delimiters_and_whitespace`**: Parses `DOG, CAT\tBIRD\nFOX,   BEAR\r\n`.
- **`format_empty_and_repeated_delimiters`**: Handles `\n\t  DOG,, ,CAT,\n\n\r\n` without creating empty tokens.
- **`format_eof_without_final_newline`**: Handles `FIRST\nSECOND\nLAST` (no trailing `\n`).
- **`format_utf8_bom_tolerated`**: Automatically strips `\u{FEFF}` on line 1.
- **`duplicate_tokens_in_batch`**: Reports 6 tokens seen, 3 inserted, 3 already present.
- **`invalid_characters_rejected_with_location`**: Rejects `;`, `-`, digits, `@`, `é` with exact line and column numbers.
- **`failure_atomicity_on_invalid_input`**: Malformed word mid-batch leaves destination trie completely unmodified.
- **`failure_atomicity_on_batch_delete`**: Malformed word in `remove_words` aborts before modifying trie.
- **`filtering_present_and_absent_preserves_order_and_duplicates`**: Preserves original token sequence and duplicates in output lists.
- **`equivalence_of_file_loading_and_individual_insertion`**: Compares `from_reader` against one-by-one `insert`.

### 3.5 Operations & Traversal (`tests/operations.rs`)
- **`single_word_lifecycle`**: Tests `insert`, `contains`, `has_prefix`, duplicate insert, and stats checks.
- **`overlapping_branches_and_subprefixes`**: Tests `CAR`, `CART`, `CAT`, `CARE` branch coexistence and `TrieCursor` traversal.
- **`contains_does_not_mutate`**: Asserts `stats()` is identical before and after `contains`.
- **`lexicographic_iteration_matches_sorted_oracle`**: Verifies iterative `visit_words` yields words in alphabetical order matching `HashSet` sorted elements.

### 3.6 Randomized Reference-Model Tests (`tests/randomized.rs`)
Tested against `HashSet<String>` with seeded SplitMix64 pseudo-random generator:
- **Seed `0x00C0_FFEE` (1,000 operations):** Balanced insert/delete/lookup over 26-letter alphabet.
- **Seed `0x5EED_2026` (10,000 operations):** Restricted 8-letter alphabet to stress deep shared prefixes and pruning.
- **Seed `0x0BAD_C0DE` (20,000 operations):** High-volume churn and deletion reinsertion.
- **Result:** 100% equivalence on insert boolean, delete boolean, contains boolean, total word count, and sorted iteration.

### 3.7 Real Corpus Regression (`tests/regression.rs`)
Tested against `assets/dictionary.txt`:
- **Unique words:** 178,691
- **Live reachable nodes:** 395,185
- **Degree distribution verified:** 116,079 leaves (degree 0), 214,926 degree 1, 39,966 degree 2.
- **Min / Max length:** 2 / 15
- **Boundary lookups:** `AA`, `ZZZ`, `ZYZZYVA`, `ZYZZYVAS`, `AARDVARK`, `ZYGOTE` all return `true`.
- **Case normalization:** `contains("aa")` and `contains("zzz")` return `true`.
- **Sampled deletion:** Deleted 2,000 words starting with `B`; asserted word count = 176,691; reinserted and confirmed 178,691 restored and invariants pass.

### 3.8 CLI Subprocess End-to-End Suite (`trie-cli/tests/cli.rs`)
Spawns compiled `trie-cli` binary with piped input:
- `cli_help_flag`: `--help` flag displays usage and exits 0.
- `cli_nonexistent_dictionary_startup_fails`: `--dictionary missing.txt` exits 1 with diagnostic.
- `cli_interactive_basic_operations`: Interactive `help`, `insert`, `contains`, `normalize`, `stats`, `remove`, `quit`.
- `cli_file_operations_and_filtering`: `load`, `present`, `absent`, `new`, `stats`.
- `cli_handles_eof_gracefully`: Immediate EOF (Ctrl+D) exits cleanly without panic.
- `cli_friendly_errors_on_malformed_input`: Unknown commands, missing arguments, extra arguments, invalid words report polite errors.

---

## 4. Adversarial Command Sequence Verification

The plan's required adversarial sequence was executed directly against `target/release/trie-cli.exe`:
```text
1. load assets/dictionary.txt       -> Loaded 178691 words into fresh dictionary
2. contains AA                     -> true: "AA" is in the dictionary.
3. contains ZZZ                    -> true: "ZZZ" is in the dictionary.
4. remove AA                       -> Removed "AA" (remaining words: 178690)
5. contains AA                     -> false: "AA" is NOT in the dictionary.
6. contains AAH                    -> true: "AAH" is in the dictionary. (branch preserved!)
7. insert AA                       -> Inserted "AA" (total words: 178691)
8. contains AA                     -> true: "AA" is in the dictionary.
9. remove-file temp_remove.txt     -> Removed 3 words (AA, CART, DOG), remaining words: 178688
10. contains CART                  -> false: "CART" is NOT in the dictionary.
11. present temp_filter.txt        -> Emitted present words preserving original order and duplicates
12. absent temp_filter.txt         -> Emitted absent words preserving original order
13. stats                          -> Accurate live node and slot breakdown
14. quit                           -> Clean exit, return code 0
```

---

## 5. Benchmarking & Resource Utilization

Measured on Windows x86_64, `rustc 1.98.1`:
- **Dictionary Construction Time:**
  - Debug build: ~340 ms
  - Release build (`--release`): ~190 ms
- **Memory Footprint (`dictionary.txt` loaded):**
  - Live reachable nodes: 395,185
  - Arena node payload: $395,185 \times 112\text{ bytes} \approx 44.26\text{ MB}$
  - Zero node heap allocation overhead (contiguous vector).
- **Lookup Throughput:**
  - Lookup uses O(L) direct indexing. The earlier “under 20 nanoseconds per word” statement lacks a saved workload/harness in this snapshot and is **UNVERIFIED**; do not use it as a measured baseline. New measurements must follow `BENCHMARKS.md`.

---

## 6. Known Limitations & Scope Boundaries

1. **Alphabet Restriction:** The trie strictly supports English ASCII letters A-Z (case-insensitive). Accented characters, hyphens, and apostrophes are rejected as invalid per course specification.
2. **OS Memory Reclamation:** `compact()` calls `Vec::shrink_to_fit()`, which releases excess capacity back to Rust's global memory allocator. Whether the allocator immediately releases pages to the operating system RSS depends on platform allocator behavior.
3. **Planned Games:** Spelling Bee, Boggle and Wordle now have approved designs and dedicated prompts; no game engine crates exist in this documentation-only snapshot. Each implementation must supply new evidence and an independent subagent review.

## 7. New checkpoint evidence policy — planned, not executed

The sections below extend this testing document; they do not replace the dated Assignment 1 log. Every new game begins `NOT IMPLEMENTED / NOT RUN`. Once implemented, add a dated evidence block with the actual branch, base/reviewed-code/merge commits, Rust/Cargo/OS versions, data hashes, exact command outputs/counts, seeds, benchmark reports and independent reviewer report. Do not overwrite historical measurements as if they were current, reuse the old 61-test total for new code, or check off a stage merely because it is planned here.

| Checkpoint | Feature branch | Required versions | Implementation / tests / comparative benchmarks / subagent review / merge |
|---|---|---|---|
| Spelling Bee | `feat/spelling-bee` | SB-S1, SB-S2, SB-P1/P2, SB-H1 | IMPLEMENTED / 28 new tests PASS (89 total) / 449.87x speedup (26.94 µs vs 12.12 ms) / Pending subagent review / NOT MERGED |
| Boggle | `feat/boggle` | BG-S1, BG-S1E, BG-S2, BG-G1/G2/G3 | NOT IMPLEMENTED / NOT RUN / NOT RUN / NOT RUN / NOT MERGED |
| Wordle | `feat/wordle` | WL-F1, WL-F2 | NOT IMPLEMENTED / NOT RUN / NOT RUN / NOT RUN / NOT MERGED |

For requirement IDs and acceptance, use [IMPLEMENTATION_PLAN.md sections 9–10](IMPLEMENTATION_PLAN.md); for comparison methodology use [BENCHMARKS.md](BENCHMARKS.md). These game policies supersede earlier “future only” wording about weighted/dice generation and Wordle, while preserving timers and rich TUI/GUI as later work.

## 8. Spelling Bee test matrix — IMPLEMENTED / VERIFIED

| Layer | Cases and properties | Verified evidence on `feat/spelling-bee` |
|---|---|---|
| Puzzle validation | Exactly seven distinct A–Z letters; center belongs to set; invalid lengths/duplicates/Unicode; center-first display | 8 unit tests in `spelling-bee/tests/puzzle.rs` PASS |
| SB-S1 reference | Tiny hand-computed expected sets; proper prefixes not words; length 3 rejected/4 accepted; repeated letters; center/outside-letter rejection | 3 unit tests in `spelling-bee/tests/filter_solver.rs` PASS |
| SB-S2 production | Independent allowed-edge traversal; complete equality with SB-S1 on fixed, sampled corpus and seeded random sets, all seven centers for selected sets | 4 differential tests in `spelling-bee/tests/differential.rs` PASS |
| Pangrams and scoring | All seven distinct letters even with repetition; 4-letter=1, ≥5=length, +7 bonus; malformed/duplicate guesses never score | 1 unit test in `spelling-bee/tests/scoring.rs` PASS |
| Generation | Anchor exactly seven distinct letters; same seed/config/hash yields same puzzle; fixed center and letters; explicit min/max; finite retries; impossible/empty cases | 6 unit tests in `spelling-bee/tests/generation.rs` PASS |
| Difficulty | Saved calibration method/cases/hash; disjoint bands, tie handling, override intersection and empty-intersection rejection | Presets calibration verified in `spelling-bee/tests/calibration.rs` PASS (`docs/benchmarks/spelling-bee/calibration.json`) |
| Session | Initial empty guessed set; accepted words subset of frozen solutions; no dictionary mutation; later trie edits do not alter round; terminal submissions cannot mutate state | 6 session tests in `spelling-bee/tests/session.rs` PASS |
| Hints/rendering | Every letter×length cell and two-letter-prefix count; marginal sums; totals explicitly labelled; alphabetical three-column accepted words | `test_hint_grid_and_counts_totals` in `session.rs` PASS |
| CLI/end-to-end | Single/batch guesses; syntactic-invalid batch changes nothing; valid batch sequential outcomes; slash commands; early missed set; score; replay; EOF; paths with spaces; ANSI/plain | 5 integration and subprocess tests in `word-games-cli/tests/spelling_bee_cli.rs` PASS |
| Randomized invariants | At least three fixed seeds; thousands of accepted/rejected/duplicate submission operations on small puzzles | 3,000 randomized operations across 3 seeds in `test_randomized_session_invariants` PASS |
| Benchmarks | Filtering versus trie on identical puzzles; preprocessing/generation/retry costs separately | `cargo bench -p spelling-bee --bench algorithms -- --full` PASS (449.87x speedup, `docs/benchmarks/spelling-bee/run-20261102/report.md`) |

Keep the full-dictionary and tiny-fixture tests complementary. Random seven-letter sets may have no solutions; they are essential solver tests even when normal generation rejects them.

## 9. Boggle test matrix — NOT IMPLEMENTED / NOT RUN

| Layer | Cases and properties | Evidence required before merge |
|---|---|---|
| Grid/adjacency | 4×4/5×5/6×6; wrong size/count; immutable tiles; neighbors 3/5/8 at corner/edge/interior; symmetry; no self/duplicate edges | Unit tests at all sizes |
| BG-S1 paths | Horizontal/vertical/diagonal/reverse paths; illegal jumps; same cell reused versus two equal-letter cells; alternative paths; multiple starts | Hand-built exact path fixtures with independent search |
| `Qu` | Q inputs normalize to one `Qu`; two cursor letters/one visited bit; minimum and scoring use spelled length; Q-without-U limitation | Positive/negative exact fixtures including excess Q/U confusion |
| Visited mask/depth | Largest grid's bit 35, deep paths, backtracking restores independent branches, >15-letter custom-dictionary words | Deliberate 6×6/path and custom corpus cases |
| BG-S1E versus BG-S2 | Full unique sorted solution equality on fixed/seeded boards; no duplicate score for multiple paths; no mutation of trie | Oracle and production implementation independence audit plus set tests |
| Resource limits | Highly repeated boards, nonempty versus truly empty, deterministic expansion budget exhausted | Explicit failure instead of truncated/zero solution set; counters reported |
| Scoring | Boundaries 2/3/4/5/6/7/8/9/12/13/14/15/16+, `Qu` length, checked total overflow and duplicate no-op | Pure tables, at least one realizable >15-letter word and session invariants |
| BG-G1 | Uniform baseline and Q policy, reproducible seeds, valid tile count at each size | Deterministic and broad distribution sanity tests without brittle statistical thresholds |
| BG-G2 | Counts derived from actual dictionary; smoothing; nonzero valid weights; stable hashes and deterministic selection | Derivation/data validation and per-size reproducibility |
| BG-G3 | Correct die count and six valid faces each; one face per die; shuffle permutation; deterministic tables and seeds | Synthetic data provenance and derivation/selection tests at all sizes |
| Playable selection | Finite retries; empty raw board remains representable; normal play retries; exhaustion and solver failure distinct | No infinite loop, no silent failure dropping |
| Session/CLI | Owned snapshot; immediate feedback; comma/space/newline lists; malformed-line atomicity; no retraction; score/missed report; quit/replay/EOF | Scripted per-game and launcher sessions; immutable terminal state |
| Differential/randomized | At least three documented seeds, many small exact dictionaries/boards plus representative full-corpus boards | Complete sets and state invariants, with full failure repro |
| Solver benchmarks | Full scan versus trie and setup-amortized query traces | Equal fixtures, raw times/counters, failure accounting |
| Generation benchmarks | Uniform versus weighted versus dice at all sizes | Raw and accepted-board metrics, retry/zero/failure rates and data versions |

For random tests, deliberately include a restricted alphabet to create repeated-letter/backtracking pressure. Full-dictionary stress tests need documented budgets; never label an incomplete search as a complete correct result.

## 10. Wordle test matrix — NOT IMPLEMENTED / NOT RUN

| Layer | Cases and properties | Evidence required before merge |
|---|---|---|
| Input/word type | Exactly five ASCII letters; case normalization; digits/Unicode/punctuation/wrong length; ordinary text versus slash commands | Typed validation and no-panic tests |
| Lexicon | All extracted guesses length five/sorted/unique; every answer in allowed set; nonempty curated answers; normalized override duplicates counted; invalid override all-or-nothing | Corpus integration and tiny answer-file fixtures with provenance |
| WL-F1 baseline | Mastermind-derived markers; exact first; excess duplicates; partial assignments left to right | Complete expected arrays, not just counts |
| WL-F2 production | Fixed remaining-frequency array, independently implemented | Complete equality with WL-F1 and independent multiset-removal oracle |
| Exhaustive feedback | All 59,049 pairs over `{A,B,C}` at length five | Test count/cases; all complete arrays equal; no reused production oracle |
| Random/corpus feedback | At least three fixed seeds; full alphabet and dictionary/answer pairs; `APPLE/ALLEY`, `LEVEL/EERIE`, all/no matches and duplicate traps | Seeded differential tests and aggregate Mastermind cross-check |
| Attempts/state | Six valid guesses; invalid input consumes none; repeated valid word consumes one; history length matches attempts | Mutation assertions after every operation |
| Terminal transitions | Correct sixth guess wins; sixth incorrect loses; no seventh evaluation; quit terminal; new round resets only round state | Transition invariants and scripted win/loss/quit fixtures |
| Target selection/replay | Injected seed; answer always in curated list; reproducibility includes data/config; lexicon reused without trie clone | Testable selection and replay cases |
| Feedback UI | Five cells per row, useful plain labels, ANSI reset/disabled behavior; keyboard correct > present > absent > unknown | Gray/yellow duplicates cannot downgrade prior stronger evidence |
| CLI/end-to-end | Custom dictionary/answers; startup errors; seed; partial invalid inputs followed by valid words; replay; EOF; no unwanted game continuation | Compiled-binary scripts and current README commands |
| Reuse audit | Every reused/adapted/excluded component is mapped to actual source and target | Source commit/hashes, updated map, adapted tests, no sibling dependency or changes |
| Benchmarks | Both scorers on identical validated five-letter pairs; complete outputs consumed | Raw batches/report and honest spread, no assumed speedup |
| Prior regressions | `trie-cli`, Bee and Boggle remain functional | Whole-workspace checks and previous README examples |

## 11. Independent review, remediation and merged-main evidence

The implementation agent must spawn a fresh reviewer subagent after the author's completion pass. This is a separate code review, not the author's self-check and not a request for a subagent to summarize author logs. Full instructions are in [IMPLEMENTATION_PLAN.md section 10](IMPLEMENTATION_PLAN.md), with an unfilled [review template](reviews/REVIEW_TEMPLATE.md).

The reviewer independently reads the requirements, source, base-to-head diff, reuse map where relevant and benchmark harness/data. It runs its own formatting/check/strict-Clippy/debug-test/release-test/release-build commands, adversarial CLI probes, README examples and benchmark smoke plus representative comparisons. It reports requirement coverage and gaps, actual command results, file/line findings and reproduction. The author pauses edits during review, resolves actionable findings and asks the reviewer to recheck fixes.

Required review paths, created by implementation agents only after actual review:

- `docs/reviews/spelling-bee.md`
- `docs/reviews/boggle.md`
- `docs/reviews/wordle.md`

If subagent capability is absent or required checks cannot run, mark the gate blocked and preserve the feature branch; do not claim independent verification or merge. Once clear, merge that game's branch locally with `--no-ff` into `main`, rerun required workspace checks and game/documentation/benchmark smoke, record the real merge SHA, and stop. No remote push is required.

For each implemented checkpoint append the following populated evidence block without deleting the earlier sections:

| Field | Actual evidence to record |
|---|---|
| Branch / base / reviewed code / merge commit | Exact Git values |
| Date / toolchain / target / data and lock hashes | Observed, not inferred |
| Required check commands and counts | PASS/FAIL/BLOCKED with actual stdout/exit summary |
| Requirement IDs | Code + independently verified test/probe mapping |
| Retained algorithm versions | Actual paths and exercised strategy options |
| Comparative run | Inputs, raw samples, methodology and report path |
| Independent review | Reviewer ID/report, reviewed code commit, findings and re-review |
| README/documentation audit | Exact usage examples executed and corrections made |
| Post-merge verification | Commands/results on actual merged `main` |
| Known limitations | Precise scope and remaining caveats |

Do not check a field “verified” solely because this template lists it. The documentation-only package creates no game test results, game branches, code review reports or merge commits.
