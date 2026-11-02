# Independent Review — Spelling Bee

## Identity and scope

| Field | Observed value |
|---|---|
| Review date/time and timezone | 2026-11-02 20:00 IST (UTC+05:30) |
| Reviewer subagent identifier | Independent Senior Rust Code & Test Reviewer Subagent (`aa148674-3f5b-4636-8276-b4157e1a2424`) |
| Repository and feature branch | `word-games` at `d:\Ashoka\Monsoon 2026\Programming Lab\Assignment 3\word-games`, branch `feat/spelling-bee` |
| Base/main commit | `39bec8d3c19645591f5ebc5486a5a84dba82c31d` |
| Reviewed code commit | `73770f6881ebbee08b9894f285c5cb5ce6b845da` |
| Remediation commits reviewed | None (initial review pass) |
| Specs/assignments and requirement IDs | `assignments/Spelling Bee.pdf`, `docs/DESIGN_SPECIFICATION.md` (sections 15–17), `docs/IMPLEMENTATION_PLAN.md` (Stage B, sections 9–10), `docs/TESTING.md`; Requirement IDs SB-01 through SB-10 |
| Dictionary, answer/dice and benchmark data versions | `assets/dictionary.txt` SHA-256: `eda7bc8b86a534de8065a7d0cc091dc918d400475061749d05a5711afda12a3b` (178,691 words); Calibration asset: `docs/benchmarks/spelling-bee/calibration.json` (`SB-P2-CALIBRATION-V1`, seed `0xCA11_B2A7_2026`, 256 sampled sets); Benchmark report: `docs/benchmarks/spelling-bee/run-20261102/report.md` |
| Working-tree state during review | Clean on `feat/spelling-bee`; author edits paused; sibling `mastermind` clean on `main` (`32e3020`) and completely untouched |

---

## Independent execution

| Exact command / probe | Environment and input | Exit/result | What it verifies |
|---|---|---|---|
| `cargo fmt --all --manifest-path word-games/Cargo.toml -- --check` | Windows x86_64, `rustc 1.98.1` | **FAIL** (exit code 1) | Code formatting compliance; 6 files have trailing newline/whitespace diffs (Finding SB-01) |
| `git diff --check 39bec8d3c19645591f5ebc5486a5a84dba82c31d..73770f6881ebbee08b9894f285c5cb5ce6b845da` | Git 2.x | **FAIL** (exit code 1) | Whitespace integrity; blank line at EOF in `Cargo.toml:9` (Finding SB-02) |
| `cargo check --workspace --all-targets --manifest-path word-games/Cargo.toml` | Dev profile | **PASS** (exit code 0) | Type-check and borrow-check all workspace targets (trie, trie-cli, spelling-bee, word-games-cli, benches, tests) |
| `cargo clippy --workspace --all-targets --manifest-path word-games/Cargo.toml -- -D warnings` | Clippy strict | **PASS** (exit code 0; 0 warnings) | Linter checks with zero warnings across all targets |
| `cargo test --workspace --manifest-path word-games/Cargo.toml` | Debug test harness | **PASS** (exit code 0; **95 passed**, 0 failed) | Full workspace test suite (29 spelling-bee, 5 word-games-cli, 52 trie, 8 trie-cli, 1 doctest) |
| `cargo test --workspace --release --manifest-path word-games/Cargo.toml` | Release test harness | **PASS** (exit code 0; **95 passed**, 0 failed) | Full workspace test suite under release profile optimizations |
| `cargo build --workspace --release --manifest-path word-games/Cargo.toml` | Cargo release | **PASS** (exit code 0) | Release binary generation for `word-games-cli` and `trie-cli` |
| `cargo bench -p spelling-bee --bench algorithms --manifest-path word-games/Cargo.toml -- --quick` | Quick smoke mode | **PASS** (exit code 0) | 5 sampled puzzles: SB-S1 = 13.51 ms/solve, SB-S2 = 93.12 µs/solve (145.10x speedup) |
| `cargo bench -p spelling-bee --bench algorithms --manifest-path word-games/Cargo.toml -- --full` | Full verification mode | **PASS** (exit code 0) | 30 puzzles × 10 iterations: SB-S1 = 11.67 ms/solve, SB-S2 = 30.81 µs/solve (378.66x speedup); 100% solution set equality |
| Subprocess: `word-games-cli --help` | Direct CLI execution | **PASS** (exit code 0) | CLI launcher usage and option documentation |
| Subprocess: fixed puzzle play with batch guesses, `/hints`, `/guesses`, `/quit` | Piped commands: `spelling-bee --letters CAIMORT --center C --color never` | **PASS** (exit code 0) | Evaluated guesses, pangram bonus, hint grid formatting, 3-column output, quit score reveal |
| Subprocess: batch syntax error atomicity | Input: `ACAI, MIC!RO` followed by `/quit` | **PASS** (exit code 0) | Pre-validation rejects whole line on syntax error; `ACAI` not accepted; zero state mutation |
| Subprocess: Queen Bee completion | Input: 5 words of small dictionary | **PASS** (exit code 0) | Terminal accolade triggered; subsequent token discarded with notice; replay prompt cleanly handled |
| Subprocess: invalid flags & constraints | `--letters CAIMORT` without `--center`, `--center Z` not in letters, invalid `--difficulty extreme`, `--color neon`, `--solver quantum` | **PASS** (exit code 1) | Rejection with typed diagnostics without panics |
| Subprocess: path with spaces | `--dictionary "target/test space/my dict.txt"` | **PASS** (exit code 0) | File paths with embedded spaces handled seamlessly |
| Subprocess: clean EOF handling | Immediate EOF at guess and replay prompts | **PASS** (exit code 0) | Clean exit without loop or panic |

---

## Requirement and algorithm audit

| Requirement ID / version | Source evidence (path:line) | Independently inspected or executed evidence | Status / gap |
|---|---|---|---|
| **SB-01**: 7 distinct letters, valid center, center-first display | `spelling-bee/src/puzzle.rs:28-175` | Tested in `spelling-bee/tests/puzzle.rs:4-74`; verified duplicate letter rejection, case normalization, non-ASCII rejection, and `display_letters` output matching PDF format (`[ C ] A I M O R T`) | **VERIFIED** |
| **SB-02**: Minimum 4 letters, repetition allowed, allowed mask, center compulsory | `spelling-bee/src/puzzle.rs:177-230` | Tested in `puzzle.rs:93-125`; words <4 letters rejected, missing center rejected, out-of-set letters rejected, repeated letters accepted | **VERIFIED** |
| **SB-03**: Complete owned solutions from both retained solvers | `spelling-bee/src/solver.rs:49-141` | Tested in `filter_solver.rs:5-57` and `differential.rs:18-151`. Verified complete equality between SB-S1 (`solve_filter`) and SB-S2 (`solve_trie_pruned`) across tiny fixtures, empty trie, zero solutions, seeded random letter sets, and full-corpus sampled sets with all 7 centers. Full set equality verified with zero mutation of the underlying trie. | **VERIFIED** |
| **SB-S1 vs SB-S2 Independence** | `spelling-bee/src/solver.rs:49` and `86` | Source inspection: `solve_filter` iterates `trie.visit_words` using 32-bit bitmasks; `solve_trie_pruned` performs DFS exploring only allowed letter edges via `trie.cursor()`. Neither function calls the other. Both are independently implemented and exposed publicly. | **VERIFIED** |
| **Owned Snapshot Semantics** | `spelling-bee/src/session.rs:68-93`, `tests/session.rs:165-209` | `SpellingBeeSession` stores owned `Vec<String>` and `BTreeSet<String>` with no lifetime or borrow of `&Trie`. Test `test_dictionary_mutation_after_session_setup_does_not_affect_round` demonstrates that mutating the trie (insert/delete) post-session setup does not alter active round solutions or acceptance. | **VERIFIED** |
| **SB-04**: Pangram-anchored generation, bounded retries, explicit exhaustion error | `spelling-bee/src/generation.rs:28-91, 178-258` | Tested in `generation.rs:20-63, 142-160`. Anchors guaranteed to have 7 distinct letters; generation guarantees $\ge 1$ pangram; impossible bounds exhaust retries and return `GenerationError::NoSuitablePuzzle { retries }`. Sampling bias towards letter sets with more dictionary anagrams is explicitly documented in module docstring. | **VERIFIED** |
| **SB-05**: Calibrated difficulty presets & bounds intersection | `spelling-bee/src/generation.rs:93-176`, `docs/benchmarks/spelling-bee/calibration.json` | Calibrated against dictionary tertiles (Easy: 15..=125, Medium: 126..=237, Hard: 238..=600); tested in `spelling-bee/tests/calibration.rs:9-65` and `generation.rs:90-122`. Verified that preset bounds intersect with explicit min/max overrides and empty intersection returns `GenerationError::InvalidBounds`. CLI runs confirmed 75 words for Easy and 260 words for Hard. | **VERIFIED** |
| **SB-06**: Checked cached scoring and duplicate rejection | `spelling-bee/src/session.rs:18-30, 148-202, 225-256` | Tested in `scoring.rs:4-29`, `session.rs:16-84`, and `test_randomized_session_invariants` (3 seeds × 1,000 operations fuzzing). 4-letter = 1 pt; $\ge 5$ letters = length; pangram bonus = +7 pts; duplicate submissions score 0; `cached_score == recalculate_score()` asserted after every submission. | **VERIFIED** |
| **SB-07**: Queen Bee accolade and early-quit summary | `spelling-bee/src/session.rs:125-129, 185-189, 205-222` | Tested in `session.rs:87-141` and interactive CLI probes. Queen Bee requires `accepted_guesses.len() == solutions.len()`; post-terminal submissions return `GameFinished` with zero score or state mutation; `/quit` returns `QuitSummary` containing score, found count, total count, and complete sorted list of missed words. | **VERIFIED** |
| **SB-08**: Three-column alphabetical word display and hint views | `spelling-bee/src/hints.rs:12-188`, `session.rs:269-300` | Tested in `session.rs:144-162`. Hint grid marginal row totals and column totals match `grand_total_words`; two-letter prefix counts match `grand_total_words`; hints display total puzzle counts (not remaining counts) matching PDF layout; three-column layout correctly formats words alphabetically down columns. | **VERIFIED** |
| **SB-09**: Interactive CLI, batch syntax atomicity, replay, EOF | `word-games-cli/src/spelling_bee.rs`, `word-games-cli/src/replay.rs` | Tested in `word-games-cli/tests/spelling_bee_cli.rs:9-138` and independent subprocess invocations. Syntax pre-validation rejects entire line if any token contains invalid characters; valid batch evaluates tokens sequentially; post-terminal tokens ignored with notice; replay menu handles choices and EOF cleanly. | **VERIFIED** |
| **SB-10**: Comparative benchmarks and documentation | `spelling-bee/benches/algorithms.rs`, `docs/benchmarks/spelling-bee/` | Benchmark harness runs with `--quick` and `--full`; raw report recorded in `docs/benchmarks/spelling-bee/run-20261102/report.md`. Calibration recorded in `calibration.json`. Testing and README updated, but small documentation discrepancies noted in Findings. | **FINDING** |
| **Mastermind Reuse Map** | `docs/MASTERMIND_REUSE_MAP.md:92-98` | Reconciled against `mastermind/src/game.rs` and `src/ui.rs`: MM-11 (`ReplayChoice`), MM-22 (`ColorMode`), MM-28 (friendly errors), MM-29 (`prompt_replay`), MM-T5 (subprocess CLI tests). No cross-repo Cargo dependencies or file modifications exist. | **VERIFIED** |
| **Untouched Reference Repo** | `Assignment 3/mastermind` | Verified via `git status` and `git diff`: 0 modified files, 0 untracked files, working tree clean at commit `32e3020`. Sibling repository is completely untouched. | **VERIFIED** |

---

## Findings

### FINDING-SB-01: Formatting Diff in Multiple Files (Severity: P2 — Actionable Defect)
- **Files & Lines:**
  - `spelling-bee/src/hints.rs:192`
  - `spelling-bee/src/lib.rs:20`
  - `spelling-bee/tests/filter_solver.rs:55`
  - `spelling-bee/tests/scoring.rs:27`
  - `word-games-cli/src/replay.rs:61`
  - `word-games-cli/src/terminal.rs:93`
- **Violated Requirement:** `docs/IMPLEMENTATION_PLAN.md` Section 10.2 preflight gate: `cargo fmt --all -- --check` must exit `0`.
- **Minimal Reproduction:**
  ```powershell
  cargo fmt --all --manifest-path word-games/Cargo.toml -- --check
  ```
- **Expected Behavior:** `cargo fmt` check exits with code `0` with zero formatting diffs.
- **Observed Behavior:** Command exited with code `1` reporting formatting diffs (extra trailing empty lines) in the 6 files listed above.
- **Proposed Remediation:** Run `cargo fmt --all --manifest-path word-games/Cargo.toml` or remove the offending trailing blank lines and commit the clean formatting.
- **Disposition:** RESOLVED (author formatted all workspace code; `cargo fmt --all -- --check` verified PASS).

---

### FINDING-SB-02: Trailing Blank Line at EOF in Root Manifest (Severity: P2 — Actionable Defect)
- **File & Line:** `word-games/Cargo.toml:9`
- **Violated Requirement:** `docs/IMPLEMENTATION_PLAN.md` Section 10.2: `git diff --check` must exit `0`.
- **Minimal Reproduction:**
  ```powershell
  git -C word-games diff --check 39bec8d3c19645591f5ebc5486a5a84dba82c31d..73770f6881ebbee08b9894f285c5cb5ce6b845da
  ```
- **Expected Behavior:** Clean git diff check without whitespace warnings.
- **Observed Behavior:** Exits with code `1` and message: `Cargo.toml:9: new blank line at EOF.`
- **Proposed Remediation:** Remove the empty trailing line 9 in `word-games/Cargo.toml`.
- **Disposition:** RESOLVED (author removed trailing blank line; `git diff --check` verified PASS).

---

### FINDING-SB-03: Test Count Inconsistency in Documentation (Severity: P3 — Documentation Polish)
- **File & Line:** `word-games/docs/TESTING.md:177`
- **Violated Requirement:** Documentation accuracy (IMPLEMENTATION_PLAN.md Section 10.2, 10.4).
- **Minimal Reproduction:** Inspect `word-games/docs/TESTING.md` line 177:
  ```markdown
  | Spelling Bee | feat/spelling-bee | SB-S1, SB-S2, SB-P1/P2, SB-H1 | IMPLEMENTED / 28 new tests PASS (89 total) / 449.87x speedup (26.94 µs vs 12.12 ms) / Pending subagent review / NOT MERGED |
  ```
- **Expected Behavior:** Summary test count reflects the actual executed test count.
- **Observed Behavior:** Line 177 reports "28 new tests PASS (89 total)". However, running `cargo test --workspace` reveals 34 new tests (29 tests in `spelling-bee` crate across `puzzle.rs`, `filter_solver.rs`, `differential.rs`, `scoring.rs`, `generation.rs`, `calibration.rs`, `session.rs` + 5 integration tests in `word-games-cli/tests/spelling_bee_cli.rs`). The total workspace test count is 95 passed tests (61 Assignment 1 tests + 34 new tests). The table in Section 8 of `TESTING.md` also correctly enumerates 29 + 5 = 34 tests.
- **Proposed Remediation:** Update line 177 in `word-games/docs/TESTING.md` to state: `34 new tests PASS (95 total)`.
- **Disposition:** RESOLVED (author updated line 177 to state 34 new tests PASS, 95 total).

---

### FINDING-SB-04: Obsolete Planning Disclaimer in README (Severity: P3 — Documentation Polish)
- **File & Line:** `word-games/README.md:265`
- **Violated Requirement:** Documentation accuracy (IMPLEMENTATION_PLAN.md Section 10.2, 10.4).
- **Minimal Reproduction:** Inspect `word-games/README.md` line 265:
  ```markdown
  The planned terminal entry point is `word-games-cli`, with an interactive launcher and game subcommands. These are design targets, not commands available in this snapshot.
  ```
- **Expected Behavior:** README text should consistently describe the working state of the current branch.
- **Observed Behavior:** Line 265 states `word-games-cli` is not available in this snapshot, directly contradicting Section 6 (lines 212–240) which documents and demonstrates running `cargo run -p word-games-cli -- spelling-bee`.
- **Proposed Remediation:** Remove or update line 265 in `word-games/README.md` to indicate that `word-games-cli` is implemented and available for Spelling Bee.
- **Disposition:** RESOLVED (author updated README description of `word-games-cli`).

---

## Final disposition

- **VERDICT: PASS (ALL FINDINGS RESOLVED)**
- **Summary:**
  All findings from the initial review pass (FINDING-SB-01 through FINDING-SB-04) have been fully resolved by the author:
  - Automated formatting checks (`cargo fmt --all -- --check`) exit 0 with zero diffs.
  - Whitespace integrity checks (`git diff --check`) exit 0 with zero warnings.
  - Workspace tests (`cargo test --workspace`) pass 95/95 tests cleanly.
  - Documentation counts and CLI launcher state are accurate and synchronized.
  - Branch `feat/spelling-bee` is cleared for local merge into `main`.

*Per section 10 protocol, the reviewer does not merge into `main`.*
