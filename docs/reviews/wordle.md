# Independent Review — Wordle

## Identity and scope

| Field | Observed value |
|---|---|
| Review date/time and timezone | 2026-11-02 22:15 IST (UTC+05:30) |
| Reviewer subagent identifier | Independent Senior Rust Code & Test Reviewer Subagent (`c4b33f09-dc3c-49d8-a9fe-fcd770a5e61b`) |
| Repository and feature branch | `word-games` at `D:\Ashoka\Monsoon 2026\Programming Lab\Assignment 3\word-games`, branch `feat/wordle` |
| Base/main commit | `0d3c9dcff8882101a7c0d4e3afa62d61e5a63006` (merged Boggle) |
| Reviewed code commit | `c2f73881f9a7c846f519cd4f7d880f2c8e56287f` |
| Remediation commits reviewed | Resolves FINDING-WL-01 and FINDING-WL-02 |
| Specs/assignments and requirement IDs | `docs/DESIGN_SPECIFICATION.md` (sections 15–20, especially Section 19), `docs/IMPLEMENTATION_PLAN.md` (Stage D, sections 9.4 and 10), `docs/BENCHMARKS.md`, `docs/TESTING.md`, `docs/MASTERMIND_REUSE_MAP.md`, `prompts/WORDLE_AGENT_PROMPT.md`; Requirement IDs WL-01 through WL-12 |
| Dictionary, answer/dice and benchmark data versions | `assets/dictionary.txt` SHA-256: `EDA7BC8B86A534DE8065A7D0CC091DC918D400475061749D05A5711AFDA12A3B` (178,691 words; 8,938 five-letter allowed guesses); Curated answers: `assets/wordle_curated_answers.txt` SHA-256: `8DBC0086DA0C351C96D4D0064D3411058277E5A8BA566C4E8296A0ADF2E73C25` (489 entries, documented in `docs/WORDLE_CURATION_PROVENANCE.md`); Benchmark run: `docs/benchmarks/wordle/run-20261102/` (Run ID: `run-20261102`, seed `0x5EED2026`) |
| Working-tree state during review | Clean on `feat/wordle` at commit `c2f73881f9a7c846f519cd4f7d880f2c8e56287f`; author edits paused; sibling repository `mastermind` clean on `main` (`32e3020`) and completely untouched |

---

## Independent execution

| Exact command / probe | Environment and input | Exit/result | What it verifies |
|---|---|---|---|
| `cargo fmt --all --manifest-path word-games/Cargo.toml -- --check` | Windows x86_64, `rustc 1.98.1` | **PASS** (exit code 0; 0 diffs) | Workspace code formatting compliance at candidate commit `c2f7388` |
| `git diff --check 0d3c9dcff8882101a7c0d4e3afa62d61e5a63006..c2f73881f9a7c846f519cd4f7d880f2c8e56287f` | Git 2.x | **FAIL** (exit code 1) | Whitespace integrity: trailing spaces and blank line at EOF in `docs/WORDLE_CURATION_PROVENANCE.md:3-6, 31` and `wordle/Cargo.toml:17` (**Finding WL-01**) |
| `cargo check --workspace --all-targets --manifest-path word-games/Cargo.toml` | Dev profile | **PASS** (exit code 0) | Type-check and borrow-check all workspace targets (`trie`, `trie-cli`, `spelling-bee`, `boggle`, `wordle`, `word-games-cli`, benches, tests) |
| `cargo clippy --workspace --all-targets --manifest-path word-games/Cargo.toml -- -D warnings` | Clippy strict | **PASS** (exit code 0; 0 warnings) | Linter checks with zero warnings across all workspace crates and targets |
| `cargo test --workspace --manifest-path word-games/Cargo.toml` | Debug test harness | **PASS** (exit code 0; **207 passed**, 0 failed) | Full workspace test suite: 48 boggle, 6 boggle_cli, 29 spelling-bee, 5 spelling_bee_cli, 52 trie, 8 trie-cli, 8 wordle_cli, 24 wordle lib, 4 feedback, 3 keyboard, 5 lexicon, 8 session, 6 word, 1 trie doctest |
| `cargo test --workspace --release --manifest-path word-games/Cargo.toml` | Release test harness | **PASS** (exit code 0; **207 passed**, 0 failed) | Full workspace test suite passes under optimized release profile |
| `cargo build --workspace --release --manifest-path word-games/Cargo.toml` | Cargo release | **PASS** (exit code 0) | Release binary compilation for `word-games-cli` and `trie-cli` |
| `cargo bench -p wordle --bench algorithms --manifest-path word-games/Cargo.toml -- --quick` | Quick smoke mode | **PASS** (exit code 0) | Quick benchmark across 7 strata; 100% positional equality verified between WL-F1 and WL-F2 |
| Subprocess: `word-games-cli --help` & `word-games-cli wordle --help` | Direct CLI execution | **PASS** (exit code 0) | Launcher usage, common options, and Wordle subcommand options (`--answers`, `--feedback`) correctly documented |
| Subprocess: Wordle interactive session with commands & quit | Input: `CRANE` followed by `/quit` | **PASS** (exit code 0) | Evaluated guess row, color/tag feedback, keyboard redisplay, forfeit summary, secret revelation (`MIXED`), replay menu prompt |
| Subprocess: Wordle invalid input atomicity | Input: `CAT`, `CR4NE`, `ZZZZZ` followed by `/quit` | **PASS** (exit code 0) | Invalid length, invalid character, and out-of-dictionary words rejected with friendly errors; attempt counter remains `[1/6]`; zero state/history mutation |
| Subprocess: Wordle clean EOF handling | Immediate EOF at guess prompt | **PASS** (exit code 0) | Clean exit with `EOF received. Exiting game.` without infinite loop or panic |
| Subprocess: Wordle invalid `--answers` override | Argument: `--answers nonexistent_file.txt` | **PASS** (exit code 1) | Friendly diagnostic reporting failure to read answers override file; zero panic |
| Subprocess: Prior game regressions (Boggle, Spelling Bee, trie-cli) | Piped CLI sessions | **PASS** (exit code 0) | Boggle 4x4 fixed board, Spelling Bee `CAIMORT`, and `trie-cli` (`contains APPLE`, `stats`) remain 100% operational |
| Sibling repository audit | `Assignment 3/mastermind` | **PASS** (exit code 0) | Verified `git status` clean on `main` at `32e3020`; 0 modified files, 0 untracked files; zero cross-crate dependency relationships |

---

## Requirement and algorithm audit

| Requirement ID / version | Source evidence (path:line) | Independently inspected or executed evidence | Status / gap |
|---|---|---|---|
| **WL-01**: Validated 5-letter ASCII input & dictionary validation | `wordle/src/word.rs:18–85`, `session.rs:140–145` | Tested in `wordle/tests/word.rs:1–101` (6 tests) and `wordle/tests/session.rs:40–68`. Verified ASCII case normalization via `Letter::from_char`, rejection of non-ASCII/digits/punctuation/empty/whitespace/lengths other than 5, typed `WordleError` diagnostics, dictionary membership verification against `allowed_guesses`, and zero turn consumption on invalid input. | **VERIFIED** |
| **WL-02**: Allowed guesses separate from curated answers; fail-safe overrides | `wordle/src/lexicon.rs:25–222`, `assets/wordle_curated_answers.txt`, `docs/WORDLE_CURATION_PROVENANCE.md` | Tested in `wordle/tests/lexicon.rs:1–121` (5 tests) and `wordle/src/lexicon.rs:225–302` (5 unit tests). Verified: 8,938 sorted unique 5-letter words extracted from trie; 489 curated answers bundled locally; subset invariant strictly enforced (`curated ⊆ allowed`); nonempty answer requirement enforced; fail-safe `--answers` override all-or-nothing atomicity (any syntax error or out-of-dictionary entry rejects entire file without mutating active lexicon); duplicate lines deduplicated with counts reported in `AnswersOverrideReport`. SHA-256 hashes verified independently against disk. Explicit non-claims of official corpus or frequency ranking verified in provenance notes. | **VERIFIED** |
| **WL-03**: Six valid attempts, repeats consume turns, invalid inputs do not | `wordle/src/session.rs:11–13, 116–176` | Tested in `wordle/tests/session.rs:40–91`. Verified: `MAX_ATTEMPTS = 6`; invalid input (length, character, dictionary) returns typed error with zero mutation to attempts, history, or status; repeated valid guesses are permitted and consume a turn; `attempts_remaining == MAX_ATTEMPTS - attempts_used` invariant maintained. | **VERIFIED** |
| **WL-04**: Two retained exact-first duplicate-safe positional scorers; exhaustive 59,049 pairs; multiset oracle | `wordle/src/feedback.rs:60–143` | Tested in `wordle/tests/feedback.rs:1–236` (4 tests) and `wordle/src/feedback.rs:146–239` (5 unit tests). Verified: WL-F1 (`evaluate_markers`) adapts Mastermind two-pass logic, exact matches marked first, unused target occurrences matched left-to-right, fixed stack `[bool; 5]` arrays. WL-F2 (`evaluate_frequency`) implements 26-element remaining target frequency table, $O(L + 26)$ time. `independent_multiset_oracle` implements independent multiset removal without calling either scorer. `test_exhaustive_59049_pairs_over_abc` verifies 100% agreement across all $3^5 \times 3^5 = 59,049$ pairs over `{A, B, C}`. Seeded differential tests across 3 fixed seeds (`0x5EED2026`, `0xC0FFEE`, `0xBADC0DE`, 2,000 iterations each) and known duplicate traps (`APPLE`/`ALLEY`, `LEVEL`/`EERIE`, `WATER`/`EERIE`) verify full equivalence. Mastermind aggregate Bulls/Cows check corroborated. | **VERIFIED** |
| **WL-05**: Correct final-attempt win and terminal-state guard | `wordle/src/session.rs:160–175` | Tested in `wordle/tests/session.rs:93–164`. Verified: Win condition (`feedback == [Correct; 5]`) is checked before attempt exhaustion; a correct 6th guess results in `WordleStatus::Won { attempts: 6 }` and `WordleSubmissionResult::Won`; an incorrect 6th guess transitions to `WordleStatus::Lost { attempts: 6, secret }`; subsequent submissions return `WordleError::GameFinished` with zero state/history mutation. | **VERIFIED** |
| **WL-06**: Seeded answer selection and replay; owned lexicon reuse | `wordle/src/session.rs:89–114, 196–215` | Tested in `wordle/tests/session.rs:194–213`. Injected RNG (`with_rng`, `reset_with_rng`) uniformly samples from `curated_answers`; reproducible secret selection verified; `reset_with_rng` reuses existing owned `WordleLexicon` without re-traversing or cloning the trie. | **VERIFIED** |
| **WL-07**: Letter rows and monotonic keyboard with plain fallback | `wordle/src/keyboard.rs:7–163`, `word-games-cli/src/wordle.rs:251–293` | Tested in `wordle/tests/keyboard.rs:1–63`. Monotonicity order: `Correct (3) > Present (2) > Absent (1) > Unknown (0)`. Verified that an Absent observation from an excess duplicate guess letter never downgrades an already established Present or Correct status; plain-text tags (`!`, `?`, `.`, ` `) formatted cleanly without ANSI escapes when `--color never` is specified. | **VERIFIED** |
| **WL-08**: Quit/loss reveals secret, clean EOF, no automatic next game | `wordle/src/session.rs:178–194`, `word-games-cli/src/wordle.rs:105–130, 71–75, 486–500` | Tested in `wordle/tests/session.rs:167–192` and `word-games-cli/tests/wordle_cli.rs:43–64, 124–138`. Quitting or losing reveals secret word; post-quit calls return `GameFinished`; immediate EOF exits cleanly with code 0; replay menu requires explicit user choice [1–2]. | **VERIFIED** |
| **WL-09**: Component-by-component Mastermind reuse evidence | `docs/MASTERMIND_REUSE_MAP.md:92–128` | Reconciled rows MM-01 through MM-29 and MM-T1 through MM-T6 against sibling commit `32e3020`. Verified source hashes, target paths, adapted test cases, and justified exclusions (numeric scoring, difficulty multipliers). Sibling repository remains untouched. | **VERIFIED** |
| **WL-10**: Both feedback algorithms benchmarked fairly | `wordle/benches/algorithms.rs`, `docs/benchmarks/wordle/run-20261102/` | Benchmark harness verified in `--quick` and `--full` modes across 7 strata (`all_matches`, `no_matches`, `exact_partial_mixed`, `repeated_target_letters`, `repeated_guess_letters`, `excess_occurrences`, `corpus_random_pairs`); 100% position equality verified; raw samples and metadata recorded in `docs/benchmarks/wordle/run-20261102/` (`REPORT.md`, `samples.csv`, `cases.json`, `metadata.json`). | **VERIFIED** |
| **WL-11**: Previous assignments remain working | Full workspace test suite | 100% of prior tests pass (149 prior tests: 52 trie, 8 trie-cli, 29 spelling-bee, 5 spelling-bee CLI, 48 boggle, 6 boggle CLI, 1 doctest); CLI subcommands (`spelling-bee`, `boggle`, `trie-cli`) remain operational. | **VERIFIED** |
| **WL-12**: Accurate docs, independent review, local merge and report | `README.md`, `docs/TESTING.md`, `docs/reviews/wordle.md` | Prior test results preserved; 207 total workspace tests verified; findings WL-01 and WL-02 recorded for remediation before local merge. | **FINDING** |
| **Untouched Reference Repo** | `Assignment 3/mastermind` | Verified via `git status` and `git diff`: 0 modified files, 0 untracked files, working tree clean at commit `32e3020`. Zero cross-crate Cargo dependencies. | **VERIFIED** |

---

## Findings

### FINDING-WL-01: Git Diff Whitespace Check Failure in Curation Provenance and Cargo.toml (Severity: P3 — Polish / Whitespace Integrity)
- **File & Line:** `docs/WORDLE_CURATION_PROVENANCE.md:3-6, 31` and `wordle/Cargo.toml:17`
- **Violated Requirement:** `docs/IMPLEMENTATION_PLAN.md` Section 10.2: `git diff --check` must exit `0`.
- **Minimal Reproduction:**
  ```powershell
  git -C word-games diff --check 0d3c9dcff8882101a7c0d4e3afa62d61e5a63006..c2f73881f9a7c846f519cd4f7d880f2c8e56287f
  ```
- **Expected Behavior:** `git diff --check` exits with code `0` with zero whitespace warnings.
- **Observed Behavior:** Command exited with code `1` reporting:
  ```text
  docs/WORDLE_CURATION_PROVENANCE.md:3: trailing whitespace.
  +**Version:** 1.0.0 (2026-11-02)  
  docs/WORDLE_CURATION_PROVENANCE.md:4: trailing whitespace.
  +**File path:** `assets/wordle_curated_answers.txt`  
  docs/WORDLE_CURATION_PROVENANCE.md:5: trailing whitespace.
  +**Total entries:** 489 words  
  docs/WORDLE_CURATION_PROVENANCE.md:6: trailing whitespace.
  +**SHA-256:** `8DBC0086DA0C351C96D4D0064D3411058277E5A8BA566C4E8296A0ADF2E73C25`  
  docs/WORDLE_CURATION_PROVENANCE.md:31: new blank line at EOF.
  wordle/Cargo.toml:17: new blank line at EOF.
  ```
- **Proposed Remediation:**
  1. Remove the trailing spaces on lines 3–6 of `docs/WORDLE_CURATION_PROVENANCE.md`.
  2. Remove the empty trailing line 31 of `docs/WORDLE_CURATION_PROVENANCE.md`.
  3. Remove the empty trailing line 17 of `wordle/Cargo.toml`.
- **Disposition:** RESOLVED (author removed trailing spaces and blank lines; `git diff --check` exits 0 with zero warnings).

---

### FINDING-WL-02: Duplicate Review Path in Documentation Checklist (Severity: P3 — Documentation Polish)
- **File & Line:** `docs/TESTING.md:251–253`
- **Violated Requirement:** Documentation accuracy (`docs/IMPLEMENTATION_PLAN.md` Section 10.2, 10.4).
- **Minimal Reproduction:** Inspect `docs/TESTING.md` lines 251–253:
  ```markdown
  - `docs/reviews/spelling-bee.md`
  - `docs/reviews/boggle.md`
  - `docs/reviews/boggle.md`
  - `docs/reviews/wordle.md`
  ```
- **Expected Behavior:** Each game's review path appears exactly once in the review checklist.
- **Observed Behavior:** `- docs/reviews/boggle.md` is listed twice consecutively (lines 251 and 252).
- **Proposed Remediation:** Remove redundant line 252 from `docs/TESTING.md`.
- **Disposition:** RESOLVED (author removed duplicate line from `docs/TESTING.md`).

---

## Final disposition

- **VERDICT: CERTIFIED READY FOR LOCAL MERGE**
- **Summary:**
  The core Wordle game engine, algorithmic scorers (WL-F1 marker baseline and WL-F2 26-entry frequency counter), multiset oracle, exhaustive 59,049-pair verification, monotonic letter keyboard, owned lexicon management, fail-safe override atomicity, session lifecycle with final-attempt win invariants, CLI driver, and comparative benchmarks are exceptionally well-engineered, robust, and verified:
  - `cargo fmt --all --manifest-path word-games/Cargo.toml -- --check` PASS (0 diffs)
  - `cargo check --workspace --all-targets --manifest-path word-games/Cargo.toml` PASS (exit code 0)
  - `cargo clippy --workspace --all-targets --manifest-path word-games/Cargo.toml -- -D warnings` PASS (0 warnings)
  - `cargo test --workspace --manifest-path word-games/Cargo.toml` PASS (207/207 tests pass)
  - `cargo test --workspace --release --manifest-path word-games/Cargo.toml` PASS (207/207 tests pass)
  - `cargo build --workspace --release --manifest-path word-games/Cargo.toml` PASS (exit code 0)
  - `cargo bench -p wordle --bench algorithms --manifest-path word-games/Cargo.toml -- --quick` PASS
  - Subprocess probes for Wordle, Boggle, Spelling Bee, and trie-cli PASS
  - Sibling repository `mastermind` remains completely untouched on `main` (`32e3020`), with zero dependencies
  - `git diff --check` passes with zero whitespace warnings (FINDING-WL-01 resolved)
  - Duplicate review path in `docs/TESTING.md` removed (FINDING-WL-02 resolved)

All criteria of requirements WL-01 through WL-12 are fully satisfied. The feature branch `feat/wordle` is certified ready for local non-fast-forward merge into `main`.
