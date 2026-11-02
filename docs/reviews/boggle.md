# Independent Review — Boggle

## Identity and scope

| Field | Observed value |
|---|---|
| Review date/time and timezone | 2026-11-02 21:15 IST (UTC+05:30) |
| Reviewer subagent identifier | Independent Senior Rust Code & Test Reviewer Subagent (`ac2c3bb0-288f-49f8-9a6f-37035e77c5bf`) |
| Repository and feature branch | `word-games` at `D:\Ashoka\Monsoon 2026\Programming Lab\Assignment 3\word-games`, branch `feat/boggle` |
| Base/main commit | `4bcf4e7fa150bf445d8387976dfc8ea979a57965` (merged Spelling Bee) |
| Reviewed code commit | `94ff06e0e6928a9396280fca71d10c827de165bd` |
| Remediation commits reviewed | Pending commit (all 3 findings verified resolved in working tree) |
| Specs/assignments and requirement IDs | `assignments/Boggle.pdf`, `docs/DESIGN_SPECIFICATION.md` (sections 15–18), `docs/IMPLEMENTATION_PLAN.md` (Stage C, sections 9–10), `docs/BENCHMARKS.md`, `docs/TESTING.md`, `docs/MASTERMIND_REUSE_MAP.md`; Requirement IDs BG-01 through BG-10 |
| Dictionary, answer/dice and benchmark data versions | `assets/dictionary.txt` SHA-256: `eda7bc8b86a534de8065a7d0cc091dc918d400475061749d05a5711afda12a3b` (178,691 words); Versioned generators: `BG-G1-UNIFORM-V1`, `BG-G2-WEIGHTED-V1`, `BG-G3-DICE-V1`; Benchmark seed: `0x5EED2026`; Benchmark report: `docs/benchmarks/boggle/run-20261102/REPORT.md` |
| Working-tree state during review | Clean on `feat/boggle` at commit `94ff06e0e6928a9396280fca71d10c827de165bd`; author edits paused; sibling repository `mastermind` clean on `main` (`32e3020`) and completely untouched |

---

## Independent execution

| Exact command / probe | Environment and input | Exit/result | What it verifies |
|---|---|---|---|
| `cargo fmt --all -- --check` | Windows x86_64, `rustc 1.98.1` | **PASS** (exit code 0; 0 diffs) | Workspace code formatting compliance at reviewed commit |
| `git diff --check 4bcf4e7fa150bf445d8387976dfc8ea979a57965..94ff06e0e6928a9396280fca71d10c827de165bd` | Git 2.x | **FAIL** (exit code 1) | Whitespace integrity; blank line at EOF in `docs/benchmarks/boggle/run-20261102/REPORT.md:59` (**Finding BG-01**) |
| `cargo check --workspace --all-targets` | Dev profile | **PASS** (exit code 0) | Type-check and borrow-check all workspace targets (`trie`, `trie-cli`, `spelling-bee`, `boggle`, `word-games-cli`, benches, tests) |
| `cargo clippy --workspace --all-targets -- -D warnings` | Clippy strict | **PASS** (exit code 0; 0 warnings) | Linter checks with zero warnings across all targets |
| `cargo test --workspace` | Debug test harness | **PASS** (exit code 0; **149 passed**, 0 failed) | Full workspace test suite (48 boggle, 6 boggle_cli, 29 spelling-bee, 5 spelling_bee_cli, 52 trie, 8 trie-cli, 1 trie doctest) |
| `cargo test --workspace --release` | Release test harness | **PASS** (exit code 0; **149 passed**, 0 failed) | Full workspace test suite under release profile optimizations |
| `cargo build --workspace --release` | Cargo release | **PASS** (exit code 0) | Release binary generation for `word-games-cli` and `trie-cli` |
| `cargo bench -p boggle --bench algorithms -- --quick --suite all` | Quick smoke mode | **PASS** (exit code 0) | Study 1: 4x4 (835.6x speedup), 5x5 (102.7x), 6x6 (301.5x); Study 2: setup amortization across trace lengths 1..1000; Study 3: uniform/weighted/dice generation quality |
| Subprocess: `word-games-cli --help` & `word-games-cli boggle --help` | Direct CLI execution | **PASS** (exit code 0) | CLI launcher usage and option documentation for Boggle |
| Subprocess: fixed board play with batch guesses, `/guesses`, `/quit` | Piped commands: `boggle --board "CATSDOGSQREWBIRD" --color never` with input `CATS, DOGS BIRD` | **PASS** (exit code 0) | Evaluated guesses (`+1 pts`), running score, Qu display, 4-column missed word formatting, quit summary (3 / 389 pts, 3 / 217 words) |
| Subprocess: batch syntax error atomicity | Input: `DOG!S` | **PASS** (exit code 0) | Pre-validation rejects whole line on invalid character `!`; zero state or score mutation |
| Subprocess: board sizes 4, 5, 6 and generators | Flags: `--size 5 --generator weighted`, `--size 6 --generator dice` | **PASS** (exit code 0) | Valid grid generation, formatting, solving, and clean forfeiture across 5x5 and 6x6 boards |
| Subprocess: invalid flags & constraints | `--size 3`, `--size 7`, `--generator magic`, `--solver quantum`, `--board CATS` | **PASS** (exit code 1) | Rejection with typed user-friendly error diagnostics without panics |
| Subprocess: clean EOF handling | Immediate EOF at guess prompt | **PASS** (exit code 0) | Clean exit with `EOF received. Exiting game.` without infinite loop or panic |
| Sibling repository audit | `Assignment 3/mastermind` | **PASS** (exit code 0) | Verified `git status` clean on `main` at `32e3020`; zero modified or untracked files |

---

## Requirement and algorithm audit

| Requirement ID / version | Source evidence (path:line) | Independently inspected or executed evidence | Status / gap |
|---|---|---|---|
| **BG-01**: Valid 4/5/6 boards and immutable 8-neighbor adjacency | `boggle/src/board.rs:114–179` | Tested in `boggle/tests/board.rs:1–178` (8 tests). Verified dimension validation (`4..=6`), exact cell count `size * size`, neighbor degree counts (corners = 3, edges = 5, interior = 8), symmetry (`A ∈ neighbors(B) ↔ B ∈ neighbors(A)`), no duplicate edges, and no self-edges. Bitmask `u64` verified across all cells up to bit 35. | **VERIFIED** |
| **BG-02**: Spelled length $\ge 3$, no cell reuse, multiple equal letters | `boggle/src/path.rs:14–113, 207–237` | Tested in `boggle/tests/path.rs:1–190` (11 tests). Verified minimum 3-letter word length, horizontal/vertical/diagonal/reverse paths, illegal jumps rejected, cell reuse strictly forbidden via copied `u64` bitmask, and multiple distinct cells with identical letters accepted. | **VERIFIED** |
| **BG-03**: `Qu` tile: 1 cell, 1 mask bit, spells 2 letters ("QU") | `boggle/src/board.rs:31–101`, `path.rs:58–65, 98–108` | Tested in `tests/board.rs:55–71` and `tests/path.rs:95–118`. Verified that 'Q' normalizes to `Tile::Qu`, consumes exactly one grid cell and one bit in `visited_mask`, spells 2 letters ("QU"), advances cursor 2 positions, and contributes 2 letters for minimum length and scoring. Coverage of the 36th cell (bit 35 on 6x6) independently verified. | **VERIFIED** |
| **BG-04**: Retention and independence of BG-S1, BG-S1E, BG-S2; set equality; search limits | `boggle/src/path.rs:40–113`, `solver.rs:49–140` | Tested in `tests/scan_solver.rs:1–80` and `tests/trie_solver.rs:1–152`. Verified BG-S1 (path search oracle), BG-S1E (`solve_scan`), and BG-S2 (`solve_trie`) are independently implemented without calling each other. 100% solution set equality verified across tiny fixtures, empty trie, zero-solution boards, pathological repeated-letter boards, and full-corpus seeded boards. Explicit `SolverError::SearchLimitExceeded` error verified when search budget is exhausted, publishing no partial session. | **VERIFIED** |
| **BG-05**: Extended score schedule, checked arithmetic, cached score invariant | `boggle/src/scoring.rs:22–59`, `session.rs:164–243` | Tested in `tests/scoring.rs:1–87` (3 tests) and `tests/session.rs:1–157` (5 tests). Verified scoring schedule: 3–4 $\to$ 1 pt, 5 $\to$ 2 pts, 6 $\to$ 3 pts, 7 $\to$ 5 pts, 8 $\to$ 11 pts, 9–12 $\to$ $2 \times L$, 13–14 $\to$ $3 \times L$, $\ge 15 \to 4 \times L$. Realizable 16-letter serpentine snake fixture verified on 4x4 board scoring 64 points. Arithmetic overflow handled with typed errors. `cached_score == recalculate_score()` asserted after every submission. | **VERIFIED** |
| **BG-06**: Three board generators (uniform, weighted, dice) across sizes 4, 5, 6 | `boggle/src/generation.rs:40–70, 147–308` | Tested in `tests/generation.rs:1–252` (12 tests). BG-G1: uniform A–Z with Q $\to$ `Qu`. BG-G2: frequency-weighted sampling from dictionary spellings with add-one Laplace smoothing; weights matched versioned constant. BG-G3: synthetic 6-face dice using largest-remainder quota allocation with alphabetical tie breaking, round-robin distribution to $N \times N$ dice, and Fisher–Yates cell shuffle. Determinism and reproducibility verified for all generators across sizes 4, 5, 6. Dictionary vocabulary bias explicitly documented in module docstring. | **VERIFIED** |
| **BG-07**: Playable board selection retry policy and honest failure | `boggle/src/generation.rs:310–336` | Tested in `tests/generation.rs:140–210`. Normal play retries zero-solution boards up to finite budget (`DEFAULT_RETRY_BUDGET = 100`), returning `GenerationError::NoPlayableBoard { retries }` upon exhaustion. Raw empty boards preserved for tests and benchmarks without synthetic padding. | **VERIFIED** |
| **BG-08**: Robust batch parsing, pre-validation atomicity, quit summary, clean EOF | `word-games-cli/src/boggle.rs:43–217` | Tested in `word-games-cli/tests/boggle_cli.rs:1–149` (6 tests). Comma/space batch tokenization; pre-validation rejects entire line if any token has invalid characters (zero state mutation); evaluated guesses are non-retractable; early quit displays achieved score, possible score, found count, total count, and 4-column missed words; immediate EOF terminates cleanly. | **VERIFIED** |
| **BG-09**: Benchmark harness and comparative studies | `boggle/benches/algorithms.rs`, `docs/benchmarks/boggle/run-20261102/` | Executed `cargo bench -p boggle --bench algorithms -- --quick --suite all`. Output reports verified in `docs/benchmarks/boggle/run-20261102/` (`REPORT.md`, `metadata.json`, `cases.json`, `samples.csv`). Covers Study 1 (complete solution enumeration speedups: 102x–835x), Study 2 (setup-amortized query trace cost with breakeven at $\sim 1000$ queries), and Study 3 (raw vs playable board quality across 900 boards for uniform/weighted/dice). | **VERIFIED** |
| **BG-10**: Documentation accuracy, prior regressions, and reuse ledger | `README.md`, `docs/TESTING.md`, `docs/MASTERMIND_REUSE_MAP.md` | Prior regressions pass (100% of Assignment 1 and Spelling Bee tests pass). Mastermind reuse map updated with rows MM-14, MM-22, MM-28, MM-29, MM-T5. Discrepancies noted in Findings BG-02 and BG-03. | **FINDING** |
| **Untouched Reference Repo** | `Assignment 3/mastermind` | Verified via `git status` and `git diff`: 0 modified files, 0 untracked files, working tree clean at commit `32e3020`. No cross-crate Cargo dependencies. | **VERIFIED** |

---

## Findings

### FINDING-BG-01: Git Diff Whitespace Check Failure in Benchmark Report (Severity: P3 — Polish)
- **File & Line:** `docs/benchmarks/boggle/run-20261102/REPORT.md:59`
- **Violated Requirement:** `docs/IMPLEMENTATION_PLAN.md` Section 10.2: `git diff --check` must exit `0`.
- **Minimal Reproduction:**
  ```powershell
  git -C word-games diff --check 4bcf4e7fa150bf445d8387976dfc8ea979a57965..94ff06e0e6928a9396280fca71d10c827de165bd
  ```
- **Expected Behavior:** `git diff --check` exits with code `0` with zero whitespace warnings.
- **Observed Behavior:** Command exited with code `1` reporting:
  ```text
  docs/benchmarks/boggle/run-20261102/REPORT.md:59: new blank line at EOF.
  ```
- **Proposed Remediation:** Remove the empty trailing line 59 in `docs/benchmarks/boggle/run-20261102/REPORT.md`.
- **Disposition:** RESOLVED (trailing blank line removed; `git diff --check` passes with exit code 0).

---

### FINDING-BG-02: Test Count Inconsistency in Documentation (Severity: P3 — Documentation Polish)
- **File & Line:** `docs/TESTING.md:178`
- **Violated Requirement:** Documentation accuracy (`docs/IMPLEMENTATION_PLAN.md` Section 10.2, 10.4).
- **Minimal Reproduction:** Inspect `docs/TESTING.md` line 178:
  ```markdown
  | Boggle | feat/boggle | BG-S1, BG-S1E, BG-S2, BG-G1/G2/G3 | IMPLEMENTED / 51 new tests PASS (146 total) / 355x–760x speedup (54 µs vs 41 ms) / Benchmarks complete / Pending independent subagent review / NOT MERGED |
  ```
- **Expected Behavior:** Summary test counts reflect actual executed tests.
- **Observed Behavior:** Line 178 reports "51 new tests PASS (146 total)". However, running `cargo test --workspace` executes 54 new tests (48 tests in `boggle` crate across `board.rs` (8), `generation.rs` (12), `path.rs` (11), `scan_solver.rs` (3), `scoring.rs` (3), `session.rs` (5), `trie_solver.rs` (6) + 6 integration tests in `word-games-cli/tests/boggle_cli.rs`). The total workspace test count is 149 passed tests (95 prior tests + 54 new tests).
- **Proposed Remediation:** Update line 178 in `docs/TESTING.md` to state: `54 new tests PASS (149 total)`.
- **Disposition:** RESOLVED (line 178 updated to `54 new tests PASS (149 total)`).

---

### FINDING-BG-03: Obsolete Planning Disclaimer Text in README (Severity: P3 — Documentation Polish)
- **File & Line:** `README.md:363` and `README.md:374`
- **Violated Requirement:** Documentation accuracy (`docs/IMPLEMENTATION_PLAN.md` Section 10.2, 10.4).
- **Minimal Reproduction:** Inspect `README.md` lines 363 and 374:
  - Line 363:
    ```markdown
    The terminal entry point is `word-games-cli`, providing an interactive launcher and the implemented `spelling-bee` game subcommand (with Boggle and Wordle subcommands planned for their respective milestones).
    ```
  - Line 374:
    ```markdown
    No game implementation, game benchmark execution, feature branch creation, or merge was performed by this documentation update.
    ```
- **Expected Behavior:** README consistently describes the current branch state where both Spelling Bee and Boggle are implemented.
- **Observed Behavior:** Line 363 claims the Boggle subcommand is only "planned", directly contradicting Section 7 which demonstrates running `cargo run -p word-games-cli -- boggle`. Line 374 contains leftover disclaimer text from the initial planning commit.
- **Proposed Remediation:** Update line 363 to indicate that both `spelling-bee` and `boggle` subcommands are implemented and available in `word-games-cli` (with Wordle planned), and update or remove line 374.
- **Disposition:** RESOLVED (README.md line 363 updated to state `spelling-bee` and `boggle` subcommands are implemented; line 374 updated to note Spelling Bee and Boggle implementations are complete).

---

## Final disposition

- **VERDICT: READY FOR LOCAL MERGE (All 3 P3 Findings Resolved)**
- **Summary:**
  The core Boggle game implementation, algorithmic solvers (BG-S1, BG-S1E, BG-S2), board generators (BG-G1, BG-G2, BG-G3), scoring schedules, session state management, CLI driver, and comparative benchmarks are exceptionally well-engineered, robust, and completely verified:
  - `cargo check --workspace --all-targets` PASS (dev profile)
  - `cargo clippy --workspace --all-targets -- -D warnings` PASS (0 warnings)
  - `cargo test --workspace` PASS (149/149 tests pass)
  - `cargo test --workspace --release` PASS (149/149 tests pass)
  - `cargo build --workspace --release` PASS
  - `cargo bench -p boggle --bench algorithms -- --quick --suite all` PASS
  - Sibling repository `mastermind` remains completely untouched on `main` (`32e3020`)
  
  All three minor P3 documentation/whitespace check findings (FINDING-BG-01, FINDING-BG-02, FINDING-BG-03) have been addressed and verified:
  1. Trailing empty line in `docs/benchmarks/boggle/run-20261102/REPORT.md` removed; `git diff --check` passes cleanly with code 0.
  2. Test count in `docs/TESTING.md:178` updated to `54 new tests PASS (149 total)`.
  3. `README.md` updated to reflect that `spelling-bee` and `boggle` are implemented in `word-games-cli`.

The branch `feat/boggle` is verified and ready for local non-fast-forward merge into `main`.
