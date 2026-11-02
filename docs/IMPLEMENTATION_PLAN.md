# Word Trie / Word Games — Agent-Executable Implementation Plan

**Purpose:** A dependency-ordered, test-first implementation runbook for a Rust agent.  
**Current baseline:** Assignment 1 (`trie` and `trie-cli`) complete. **Approved implementation sequence:** Spelling Bee → Boggle → Wordle, one dedicated prompt and feature branch at a time.<br>
**Hard stop:** After the invoked game passes author checks, independent subagent review, documentation verification, and its local merge to `main`, report and stop. Do not invoke the next game prompt automatically. Section 10 defines the branch/merge protocol; section 8 adds Wordle; section 9 fixes the retained algorithm versions. This is a documentation-only revision dated 2026-11-02.<br>
**Companion authority:** [`DESIGN_SPECIFICATION.md`](DESIGN_SPECIFICATION.md). This plan gives execution order and objective proof obligations; the design spec gives representation and semantic decisions.

## 0. Agent operational contract

1. Read the source PDFs, `dictionary.txt`, `DESIGN_SPECIFICATION.md`, this plan, and the existing repository before editing anything. Do not overwrite unrelated work. Respect already present source, local modifications, and existing Git history.
2. Confirm checkout, branch, Rust toolchain, workspace members, and available test commands; inspect `.gitignore` and `git status`. If no repo exists and user has authorized new repository creation, initialize one. Do not force-push, rewrite shared history, or delete unrelated files.
3. Work in **small, compiling, independently testable increments**. A function is not done until corresponding positive, negative, edge-case, and invariant tests exist and pass.
4. **Conventional Commits** (examples: `feat(trie): add checked insertion`, `fix(trie): prevent double-free on pruning`, `test(trie): cover free-list churn`, `feat(trie-cli): add present and absent filters`, `docs: verify assignment-1 commands`). Use imperative subjects, coherent scopes, and explanatory bodies for consequential design decisions. Do not record secrets or generated build outputs in Git.
5. Run formatting, compiler/lint checks, unit/integration tests, and release checks at appropriate intervals. Fix warnings introduced by your changes, do not mask test failures.
6. Update `README.md` and `docs/TESTING.md` **as features land**, reflecting only behavior that can be verified in the checkout. No unchecked claims, invented measurements, or future feature labels implying implementation.
7. At **each assignment checkpoint**, independently re-read source code, workspace manifests, CLI help, tests, and relevant PDF; rerun documentation examples and tests; correct documentation before final checkpoint commit.
8. Report completed requirements, exact commands and outcomes, tests with seeds, benchmarking results (if run), any known limitations, final Git commits, and a mapping from each PDF requirement to verification evidence. **STOP**; no autonomous continuation into next assignment.
9. Do not add real game engines or game-specific crates during Assignment 1. It is permitted to keep future designs in Markdown.
10. If instruction conflicts occur, prioritize the user's latest explicit decisions and attached assignment requirements, then the design spec, then this plan. Flag material gaps; do not silently substitute a different representation or ownership model.

## 1. Milestones and dependency graph

```mermaid
flowchart TD
  R[Read PDFs, inspect repo and toolchain] --> P1[1. Scaffold trie and trie-cli]
  P1 --> P2[2. Letter validation and errors]
  P2 --> P3[3. Indexed arena and invariants]
  P3 --> P4[4. Insert / lookup / prefix / cursors]
  P4 --> P5[5. Delete + free list + auto-compaction]
  P5 --> P6[6. Loading / batches / filtering]
  P6 --> P7[7. Interactive trie-cli]
  P7 --> P8[8. Property, integration and performance tests]
  P8 --> DOC[9. Reverify docs and PDF checklist]
  DOC --> G1{Assignment 1 fully verified?}
  G1 -- No --> FIX[Fix and rerun] --> DOC
  G1 -- Yes --> STOP1[[COMMIT, REPORT, STOP]]
  STOP1 -. only after explicit user permission .-> SB[10. Spelling Bee]
  SB --> G2[[DOCS + TESTS + COMMIT + STOP]]
  G2 -. only after explicit user permission .-> BG[11. Boggle]
  BG --> G3[[DOCS + TESTS + COMMIT + STOP]]
```

## 2. Stage A — Assignment 1 (`trie` and `trie-cli`) [COMPLETED BASELINE — retain for regression]

### A0. Preflight, repository and acceptance setup

**Tasks**
- Locate/read exact attachments or their copies in the user repo: `assignments/Word Games.pdf` is first-stage authority, with future PDFs for architectural awareness. Check for existing code and docs rather than blindly generating.
- Create/select `word-trie/` workspace root with `trie` (library) and `trie-cli` (binary). Use stable Rust edition supported by installed toolchain; declare edition/MSRV assumptions in README. Avoid dependency proliferation; standard library is sufficient for core trie.
- Add `.gitignore` including `/target/`, temporary files, local benchmarks/coverage outputs as appropriate; include `Cargo.lock` for the workspace application.
- Preserve supplied `assets/dictionary.txt` byte-for-byte in a clearly documented asset location (or reference an external location if its license/repo inclusion is uncertain). Do not silently reformat or drop its unterminated last line.
- Establish requirement checklist IDs `WG-01` ... `WG-11` below and an evidence log in `docs/TESTING.md`.
- Create minimal accurate README stating that implementation is underway, with working commands only when they exist.

**Done when:** `cargo metadata --no-deps` shows only `trie` and `trie-cli` as workspace members; `cargo check --workspace` succeeds; repository status is understood; docs do not falsely claim game implementation.

**Suggested commit:** `chore: initialize trie workspace and stage-one documentation`

### A1. ASCII alphabet abstraction, normalization and typed errors

**Files:** `trie/src/letter.rs`, `trie/src/error.rs`, public exports from `lib.rs`.

**Tasks**
1. Define `Letter(u8)` with private field, checked constructor from ASCII `A-Z`/`a-z`, `index() -> usize`, `to_char() -> char`, and `Copy` semantics.
2. Introduce a single canonical word validation/normalization path. Inputs to public APIs are `&str`; allow ASCII alphabetic only; convert lowercase to uppercase; reject empty, non-ASCII, numbers, apostrophes, hyphens, embedded spaces, etc. Ensure lookups can normalize without allocating on common uppercase path; avoid Unicode case folding.
3. Public conversion helper for professor's explicit uppercase requirement; return owned/borrowed string through `Cow<str>` or equally clear API.
4. Define informative `WordError` and `LoadError` types; include offending character/index or source line/column when possible. Do not call `unwrap` on user input.
5. Decide and document `contains(invalid) == false`, `try_contains(invalid) == Err`, `has_prefix("") == true`, and empty-word insertion/removal == `Err`.

**Tests, minimum:** `letter_A-Z`, `letter_a-z`, `letter_invalid_ascii`, `letter_unicode`, `case_upper_identity`, `case_lower`, `case_mixed`, `empty_rejected`, `space_rejected`, `hyphen_rejected`, `apostrophe_rejected`, `digit_rejected`, `unicode_expansion_rejected`, `invalid_lookup_safe`, `normalization_idempotent`. Include a loop over all 26 letters and a table-driven list of malformed tokens.

**Done when:** all validation tests pass; rustdoc explains normalization/error rules.

**Suggested commit:** `feat(trie): add ASCII letter validation and normalization`

### A2. Node arena, allocation and debug invariants

**Files:** `trie/src/node.rs`, private core methods in `lib.rs`.

**Tasks**
1. Implement private `NodeId(u32)`, `Node { children: [u32; 26], is_word: bool, subtree_words: u32 }`, `NO_CHILD=u32::MAX`; check overflow before conversion/index allocation.
2. Initialize exactly one root at index zero; preserve it even when dictionary becomes empty.
3. Implement `allocate_node` using free list before `Vec::push`, and `free_node` that resets slot and appends ID only after the parent's edge is detached. Prevent freeing root, duplicate free, freeing reachable/live nodes.
4. Track `word_count` and `live_node_count`; expose read-only `TrieStats` (words, live nodes, slots allocated, free slots, maybe spare capacity). Stats must distinguish live nodes from vector length.
5. Add test-only invariant walker: reachable graph, no free index reachable, no duplicate free IDs, count recurrence, reachable node counts, terminal words count, no nonroot zero-subtree reachable nodes, child index bounds, root preserved.
6. Avoid pointers/references into `Vec` over operations that may reallocate. Ensure all indexing is safe/checked at the relevant boundary.

**Tests:** fresh root, no free nodes, child map sentinel, ID capacity boundary (with test seam/mocked allocator if needed), allocate/reuse free slot, cannot free root, stats consistency, internal invariant violation detection where practical.

**Done when:** safe allocation primitives compile; property/invariant helper can validate empty and synthetic trees.

**Suggested commit:** `feat(trie): implement indexed arena and allocation invariants`

### A3. Trie operations: creation, insertion, lookup, prefixes, traversal

**Files:** `trie/src/lib.rs`, `trie/src/traversal.rs`, `trie/tests/operations.rs`.

**Tasks**
1. `Trie::new()`, `Default`, `len()`, `is_empty()`, `stats()`.
2. `insert(&mut self, &str) -> Result<bool,WordError>`: validate all input first; detect duplicates; follow/create edges; increment subtree counters only once for successful new terminal; cap-check before mutation; preserve invariants on failure.
3. `contains(&self, &str) -> bool` and `try_contains` fallible variant; distinguish complete words from proper prefixes.
4. `has_prefix(&self, &str) -> bool`: normalize ASCII; permit empty prefix, reject invalid prefix via false.
5. Read-only `TrieCursor` with `advance(Letter) -> Option<Self>`, `is_word`, `has_children`; no public mutable node IDs. Cursor must follow the current immutable trie without allocations.
6. `visit_words` enumerates every active dictionary word exactly once; lexicographic order strongly recommended, with no exposure of dead nodes. Prefer iterative traversal for unbounded word lengths; if recursive, justify stack limitations and test long words.

**Tests:** empty, single word, prefix not full word, prefix itself full word, overlapping branches (`CAR`, `CART`, `CAT`), root branches, duplicate/uppercase/lowercase/mixed input, invalid does not mutate, lexicographic iteration, complete enumeration vs `HashSet`, cursor path validity, absent child, allocation reuse after later deletion. Verify `contains` borrows `&Trie` and does not cause mutations via stats checks.

**Done when:** `cargo test -p trie` passes and every successful insert preserves debug invariants; all APIs documented with examples.

**Suggested commit:** `feat(trie): implement insertion, membership and read-only cursors`

### A4. Deletion, pruning, free-list reuse and automatic compaction

**Files:** trie internals, `trie/tests/deletion.rs`, randomized fixtures.

**Tasks**
1. Implement single-word `remove` with full validation before mutation. Absent word/absent terminal returns `Ok(false)`, no structural changes.
2. Record all `NodeId`s/edge indices along path; clear terminal and decrement root-to-terminal subtree counts exactly once.
3. Walk upward, unlink each zero-subtree child, clear slot, add to free list; stop when surviving subtree found. Never reclaim reachable nodes or root.
4. Repeated insertion after deletion should reuse free slots before growing the arena. Assert stability of lookup and counts through churn.
5. Add threshold-based `maybe_compact`: when at least 4,096 slots free and at least one-third of slots are free (threshold configurable/private), traverse live reachable nodes, build dense arena, rewrite child IDs, clear free list, recompute or verify counters, shrink capacity. Make it automatic in single deletions and once at end of batch deletions. Prefer temporarily disabling the trigger within batch so it doesn't run repeatedly.
6. Test explicit forced-compaction hook in unit tests even on tiny fixtures so compaction is exercised without needing 4,096 frees; keep the hook private/test-only.
7. Do not make public cursors persistable across mutation; borrow checker should prevent a simultaneous long-lived cursor and `&mut Trie` in safe Rust.

**Tests, minimum:** delete absent, delete leaf, delete word that is prefix, delete child while keeping siblings, delete deepest while keeping shorter prefix, delete every word and retain root, reinsert deleted, case-insensitive delete, duplicate delete, invalid delete leaves state unchanged, counts all ancestors, free-list slot reuse, repeated churn 1000+ operations, delete entire branch, force compact, automatic threshold path, compaction reindexes edges safely, root stable, no double-free, large delete-many/insertion alternation. Test documented memory behavior: slot reuse immediate; RSS reduction not guaranteed by allocator.

**Done when:** every case matches reference `HashSet`, invariant walker passes after each operation and each compaction, no stale/free ID is reachable.

**Suggested commit:** `feat(trie): prune deleted paths and reclaim arena storage`

### A5. Text parsing, file loading, batch operations, filtering

**Files:** `trie/src/loading.rs`, `trie/tests/loading.rs`.

**Tasks**
1. Implement parser over `BufRead` with comma and ASCII whitespace delimiters, valid mixed inputs, CRLF, blank lines, final line lacking newline, duplicate words, and reported invalid token locations.
2. Use one shared parser for insertion files, deletion files, presence/absence filter files, and CLI operations. Do not reimplement tokenization in CLI.
3. Implement `Trie::from_reader` (new trie), path convenience wrapper as appropriate, and `extend_from_reader` (existing trie), with errors that prevent partial update due to *validation/I/O* errors.
4. Implement batch deletion of an arbitrary sequence/list and a file wrapper; all input tokens validated before changing the trie. Avoid compaction after every item; compact once at end if needed.
5. Provide membership classification `present` and `absent`: preserve input order and duplicates, return normalized uppercase tokens, share parser and central `contains` logic.
6. Reports: total tokens, newly inserted, already present; number requested/actually removed/not found. Be consistent about duplicates.
7. Empty file -> valid zero-token batch; repeated commas/whitespace -> no phantom empty words; reject semicolons/punctuation and non-ASCII rather than quietly skipping them.

**Tests:** one-per-line, comma-only, mixed separators, tabs, CRLF, blank lines, leading/trailing separators, no final newline, duplicate batch, empty file, invalid token at beginning/middle/end, Unicode, unexpected semicolon, I/O failure mid-read using custom `Read`, atomic append/delete on invalid/IO, filtering all present/all missing/mixed/prefix-only, filtering preserves order/duplicates, uppercase normalization, equivalence of loading a file and manually inserting its tokens.

**Done when:** 100% assignment data-operation coverage, unit/integration tests pass for an in-memory reader and actual temporary files.

**Suggested commit:** `feat(trie): support streaming parsing and atomic batch operations`

### A6. `trie-cli` interactive terminal

**Files:** `trie-cli/src/{main.rs,commands.rs,repl.rs}`, `trie-cli/tests/cli.rs`.

**Tasks**
1. Support optional startup `--dictionary <path>`, retaining an empty trie if no path provided. Startup load failures should report an error and exit nonzero without an unusable partially loaded trie.
2. Provide REPL commands: `help`, `new`, `load`, `add-file`, `insert`, `contains`, `remove`, `remove-file`, `present`, `absent`, `normalize`, `stats`, `quit`/`exit`. Preserve existing PDF's `trie-cli` package name.
3. Clarify that `load` **replaces** dictionary and `add-file` **extends** it; print state-change reports; handle quoted or raw remainder-of-line file paths with spaces where practical.
4. User-friendly errors for invalid words, missing paths, malformed files, unknown commands, missing arguments, extra arguments, EOF, and I/O errors; never dump a Rust panic on normal input.
5. CLI contains reports a boolean; provide helpful explanation for malformed words. `present`/`absent` print normalized entries in original order, including duplicates.
6. REPL works with redirected stdin, making end-to-end integration tests deterministic and allowing scripted demonstrations.
7. Avoid implementing game UI or shared TUI framework now.

**Tests:** spawn binary using Cargo executable path, pipe scripted commands, assert stdout/stderr/exit status, check init/load/add/remove/contains/bulk lists/present/absent/normalization/stats/help/exit, malformed command, missing file, malformed file, path with spaces, empty input, EOF, no silent loss after failed batch, case normalization. Test representative complete session from README verbatim.

**Done when:** manual walkthrough and scripted subprocess suite pass without requiring interaction with real stdin during tests.

**Suggested commit:** `feat(trie-cli): add interactive dictionary management`

### A7. Rigorous verification and regression suite

**Required layers:**

1. **Unit tests:** isolated alphabet, normalization, node allocation, trie operations, batches, parser, cursors, pruning, compaction.
2. **Integration tests:** public library API from a separate crate/test module; the real supplied dictionary; temp files; binary REPL with redirected input.
3. **Reference-model randomized tests:** fixed-seed operation streams against `HashSet<String>`; compare insert boolean/remove boolean/contains/len/complete iteration after every operation or at checkpoints; deterministic reproducibility via seed logged in output.
4. **Property-style tests:** no invalid input mutates trie; normalization idempotent; an inserted word is present; deleting a word removes exactly that membership; operations on one word do not alter other words; for any word `w` all nonempty valid prefixes exist structurally until all descendant words removed; `root.subtree_words == len`.
5. **Adversarial churn:** deep prefixes, branching and shared prefixes, repeatedly empty/refill, high alphabet fanout, long words and many duplicates, randomized deletion-order permutations, intermittent forced compactions.
6. **File-format fuzz table:** generated comma/space/newline mixes; malformed token injection; chunk boundaries; missing final newline; simulated I/O failure; invalid UTF-8 handling as correct reported error, not silent corruption or crash.
7. **Real corpus regression:** expect 178,691 unique words, word length min 2/max 15, exact normal trie nodes 395,185, every dictionary word present, no stray words in separately generated controls, iterations sorted/unique. Guard against missing last line when using `lines()` or `wc -l`.
8. **Release checks:** `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo build --workspace --release`; where possible also `cargo test --workspace --release`.
9. **Benchmark smoke:** load time, memory via platform facilities if available, valid/invalid lookup throughput, insert/remove throughput, free-list reuse, compaction time. Label hardware, Rust version, optimizer/build settings, repeats, and confidence/distribution. No invented results or unsupported performance claims.
10. **Documentation tests:** run rustdoc doctests; execute exact commands and REPL examples from README; if a command differs from actual output, correct it.

**Suggested deterministic seeds:** at least three distinct documented seeds (e.g., `0xC0FFEE`, `0x5EED2026`, `0xBADC0DE`) and streams of 1k, 10k, and, if CI/resources allow, 100k mixed operations. Use seeded PRNG; random fuzz cases must be reproducible from recorded seed. These counts are goals, not excuses to sacrifice invariant verification.

**Acceptance:** no failing tests, no unaddressed project warnings, no unexplained discrepancy with baseline dictionary facts, no unsupported CLI usage examples.

**Suggested commit:** `test(trie): add model-based and full-dictionary regression coverage`

### A8. Documentation verification and submission gate

**Inspect and reconcile:** actual workspace manifests, public Rust API, executed CLI `help`, README installation/commands, TESTING command records/seeds/results, git diff/status, `Word Games.pdf` line-by-line requirements, and compiled release binary. Do not simply append a “verified” stamp without checking.

**Checkpoint 1 acceptance table (all MUST be checked and evidenced):**

| ID | Source requirement | Minimum evidence |
|---|---|---|
| WG-01 | Create empty trie | `new`, `Default`, CLI `new`, fresh-root test |
| WG-02 | Boolean lookup | `contains`, direct and CLI test |
| WG-03 | Insert single word | `insert`, duplicate and mixed-case tests |
| WG-04 | Build from one-word/line file | real dictionary and fixture load tests |
| WG-05 | Build from comma/whitespace file | mixed-delimiter parsing tests |
| WG-06 | Lowercase → uppercase | public helper and CLI normalization test |
| WG-07 | Delete one word | prefix/sibling/pruning tests |
| WG-08 | Delete list of words | batch and file deletion tests |
| WG-09 | Return words in trie from list file | present filtering test, order/duplicates preserved |
| WG-10 | Return words absent from trie | absent filtering test, order/duplicates preserved |
| WG-11 | Thorough manual/generated testing; modular Rust, docs, Cargo | randomized suite, corpus test, README, TESTING, Cargo release checks |

**One essential adversarial command sequence:** load full dictionary, verify `AA` and `ZZZ`, remove `AA`, verify `AAH` remains, reinsert `AA`, exercise `remove-file` with mixed separators, check `present`/`absent` with duplicates, and exit cleanly. Do not ship if any of these fail.

**Checkpoint outputs:**
- README leads with the current **working trie/CLI**, precise build/run commands, verified examples, and later games under a clearly labeled roadmap (not installation guidance for nonexistent crates).
- TESTING contains executable commands, actual run outcomes (including skipped tests), regression numbers, reproducible seeds, target environment, benchmark results only if measured, and limitations.
- Design and plan remain checked in so an agent/user understands the architecture and stopped stage.
- `git status` clean after meaningful Conventional Commits; if dirty due user work, report rather than overriding.
- Final status report cites requirement IDs, test evidence, exact commits, and states **“Assignment 1 complete; awaiting explicit approval for Spelling Bee.”**

**HARD STOP.** Do not scaffold or implement `spelling-bee`, `boggle`, or shared game UI crates before receiving approval.

---

## 3. Stage B — Spelling Bee [IMPLEMENTED & VERIFIED on feat/spelling-bee]

### B0. Stage transition and workspace

Before starting, re-open `Spelling Bee.pdf`, confirm approval, read current docs/working code, ensure Assignment 1 remains passing, and add `spelling-bee` engine crate plus `word-games-cli` only as necessary. Avoid `games-core` until shared code truly exists. Keep `trie-cli` unchanged except bug fixes.

### B1. Immutable puzzle state and solution-generation oracle

- `PuzzleConfig` contains seed, center-choice policy, required pangram count, solution count range, and difficulty category thresholds.
- `SpellingBeeGame` owns seven letters/allowed mask, frozen sorted solution vector, accepted `BTreeSet`, cached score, terminal state. No persistent dictionary reference/lifetime.
- Implement exhaustive dictionary visitor + `u32` masks **first**; manually enumerate tiny fixtures; assert all solutions length≥4, use only seven letters and contain center; detect pangrams by mask equality.
- Model tests: repeated letters allowed; invalid center; fewer/more than seven distinct letters rejected; no solutions; pangram and nonpangram; obscure dictionary words.

### B2. Trie-pruned enumerator

- Add DFS from immutable trie cursor, only exploring allowed-letter edges, recording terminal words of length≥4 including central letter.
- Differential-test complete **sets** against exhaustive filtering on fixed puzzles, random seeds and all seven-letter anchors selected in a manageable sample. Avoid comparing only solution counts.
- Benchmark filtering and pruned traversal with identical puzzles; store method, compiler mode, sample count, min/median/max times, and solution count.

### B3. Pangram-anchored puzzle generator, difficulty, hints

- Pre-index eligible dictionary words having exactly seven distinct letters; choose seed anchor via injected deterministic RNG. Set seven letters from its distinct-letter set, select central letter, compute solutions, test difficulty constraints; bounded retry, explicit “no eligible puzzle” error. Document sampling bias.
- Define user-adjustable targets and reproducible seed override; reject incoherent configurations.
- Hints: starting-letter×length grid and two-letter-prefix counts derived from frozen solution set, never leaked guesses.
- Tests: same seed/config => same puzzle, anchor pangram exists, retry exhaustion, difficulty bounds, hint sums match solution count.

### B4. Interactive play, scoring and CLI

- Validate each guess immediately with typed outcomes; score 4-letter=1, ≥5 letters=length, pangram bonus +7 default, **only once** per new valid guess.
- Maintain `cached_score == sum(score(guesses))`; randomized assertion after every submission. Show guessed words sorted in three columns; Queen Bee and `/quit` early reveal behavior.
- CLI tests: invalid/outside letters, missing center, duplicates, score increments, pangram, no premature Queen Bee, early missed-word report, clean EOF.

### B5. Assignment 2 hard checkpoint

- Re-run all Assignment 1 tests and new engine/CLI tests; verify from `Spelling Bee.pdf` each mandatory requirement; inspect README and TESTING **against repository** and regenerate examples/results; commit.
- Update README primary emphasis to *currently working* trie CLI and Spelling Bee; Boggle/Wordle explicitly future.
- Record performance comparison evidence and known limitations.
- Complete independent subagent review and remediation under section 10; merge `feat/spelling-bee` locally into `main`, verify the merge, report, and **STOP**. Boggle begins through its own prompt.

---

## 4. Stage C — Boggle [IMPLEMENTED & VERIFIED on feat/boggle]

### C0. Read source and transition

Re-read `Boggle.pdf`, inspect current code and updated docs, verify permission, add `boggle` engine crate. Keep all previously passing functionality intact.

### C1. Validated board, cells, neighbors and tiles

- Grid size `{4,5,6}`, flattened tiles `Vec<Tile>` with `Letter` or `Qu` enum; immutable during round. Reject invalid size/count.
- Precompute 8-neighbor adjacency for every cell, with no self-edge and no duplicate neighbor; verify symmetry, corners/edges/center counts (3/5/8 neighbors).
- Test `Qu` expands to letters `QU` while consuming one cell; string length=2 for scoring and word-minimum purposes.

### C2. Independent on-demand validation oracle

- Given a word already checked against dictionary, backtrack over board paths using a copied `u64` visited mask; never reuse a single cell. Do not incorrectly forbid using two **different** cells bearing same letter.
- Tests: diagonal, reverse path, repeated letters, alternate paths, impossible word that is in dictionary, Q-without-U limitation, `Qu` path, fixed-size boundary and 36th-bit correctness.

### C3. Primary trie-guided DFS enumeration

- Starting at each tile, advance trie cursor by letter(s), stop at invalid prefixes, visit neighbors not in mask; record unique terminal words of length≥3; maintain no game-session borrow after setup.
- Test complete solution **set equality** against independent oracle over fixed/test-generated boards and sampled dictionary subsets. Stress highly repeated/vowel-poor boards, 4/5/6 sizes; bound resource budgets and report search time.
- Benchmark precompute-plus-lookup vs per-guess DFS for varying number of submitted guesses, not just isolated function calls; compare throughput and memory.

### C4. Generation, score, session, interaction

- Uniform A–Z generation first; deterministic seed injection; log board seed/config for reproduction.
- Scoring: 3–4→1, 5→2, 6→3, 7→5, 8→11, 9–12→2L, 13–14→3L, lengths ≥15→4L; verify each boundary and a custom-dictionary word longer than 15. Qu contributes two spelled letters.
- Immediate feedback; accept comma/space/newline-separated guesses; reject malformed; record accepted guesses irreversibly, no duplicate score, clean `/quit`, track cached score with invariant.
- Keep gameplay untimed. Implement frequency-weighted and dice-like board generation as the approved C5 increments after the uniform/core checkpoint; retain all three generators. Timers and difficulty-based timed settings remain later extensions.

### C5. Approved generation increments and comparative benchmarks

- **BG-G1:** retain uniform sampling as the baseline; map sampled Q to one `Qu` tile.
- **BG-G2:** add selectable frequency-weighted sampling using versioned counts derived from the supplied dictionary, with source hash and derivation recorded.
- **BG-G3:** add selectable six-face dice-like generation for 4×4, 5×5 and 6×6 boards, one draw per die followed by a shuffle. Commit reproducible synthetic dice data and its provenance; do not claim official Boggle dice.
- At each increment, run tests, deterministic reproduction checks, and a benchmark smoke run before the next increment. Compare all generators on raw draws and separately on playable-board selection; report word counts, total possible score, word-length distribution, vowel balance, zero-word rates, retries, solver cost and failures. See `BENCHMARKS.md`.
- Bounded regeneration of zero-solution random boards is required for normal play; explicit test boards may contain zero words. No silent partial solution sets after resource exhaustion.
- Timed mode and difficulty-based timers remain deferred; they must not delay this untimed branch.

### C6. Assignment 3 hard checkpoint

- Run entire workspace tests, randomized/differential cases, original assignment CLI/manual checks, release checks and benchmarking where applicable.
- Re-check README/TESTING **with executed outputs and actual files**, update first-page feature list to reflect three implemented assignments and clearly separate optional future work.
- Complete the independent review/remediation and local merge of `feat/boggle` under section 10, verify merged `main`, report, and **STOP**. Wordle starts only when its dedicated prompt is invoked and both prior games are merged.

---

## 5. Living documentation protocol (applies at every stage and every significant change)

```mermaid
flowchart LR
  CODE[Source / CLI / Cargo changes] --> TEST[Run tests & inspect behavior]
  TEST --> R[Update README: what works now]
  TEST --> T[Update TESTING: actual evidence]
  R --> VERIFY[Independent document audit against repository]
  T --> VERIFY
  VERIFY -->|Mismatch| CODE
  VERIFY -->|Confirmed| COMMIT[Commit with Conventional Commit message]
```

**README must always:** explain project purpose/current stage; precisely list current working crates and features; installation prerequisites, build and run commands; genuinely runnable examples; current architecture summary; testing quickstart; status of optional games/UI; known limitations; links to design/plan/testing; licensing/asset provenance. Prefer a newcomer-focused README over an exhaustive design dump.

**TESTING must always:** maintain test layers, exact commands actually run, scope/seed/fixture counts, last verified date/commit when known, the actual pass/fail/skip status, benchmark methodology/results with provenance, and unresolved issues. Mark planned cases as `NOT YET IMPLEMENTED` rather than claiming they passed. Explain how someone else can reproduce tests. Do not fabricate dates, results, runtime, memory, or Git commit IDs.

**Checkpoint audit:** view actual files; run `cargo metadata`, `cargo test`, `cargo clippy`, release build and `--help`/REPL; run README command samples; compare PDFs and feature checklist; write evidence; review documentation diff; commit. If actual execution is unavailable, accurately report `NOT RUN` and do not declare checkpoint passed.

## 6. Git working conventions

- Default small commits at module or cohesive behavior boundary; test changes bundled with related feature or separate `test(scope)` commits; docs updated with each relevant feature.
- Prefixes: `feat`, `fix`, `test`, `docs`, `refactor`, `perf`, `chore`, `ci`; use conventional `type(optional-scope): short imperative description`, ideally ≤72-character subject.
- No secrets, binaries, `target/`, machine-specific paths, or synthetic benchmark results committed.
- Before each commit: `git diff --check`, inspect diff, run relevant tests, `git status`; commit only owned changes. Never stage unrelated user edits wholesale.
- Do not make dummy commits, alter/remap prior commits, force-push, or promise CI passes without running it.
- At checkpoint, report real commit SHAs from Git and branch, exactly as returned by the repository.

## 7. Recorded Assignment 1 deliverable checklist

The checked entries below belong to the supplied checkpoint-one record. They are not evidence that any later game has been implemented or reviewed. Use the fresh gates in section 10 for every game.

- [x] Every PDF-mandated function/behavior for **the authorized assignment** exists.
- [x] Public API has examples, documented errors and ownership semantics.
- [x] Core invariants checked and regression-tested.
- [x] Unit, integration, seeded randomized, adversarial, CLI and corpus tests appropriate to current stage pass.
- [x] `cargo fmt --check`, `cargo check`, `cargo clippy -D warnings`, `cargo test`, and release build completed.
- [x] Benchmark methodology/results recorded if measurements were run; no unsupported claims.
- [x] README describes CURRENT capabilities only; examples actually run.
- [x] TESTING is accurate, reproducible, and records executed verification.
- [x] Design spec and plan reflect any user-approved deviations.
- [x] Conventional commits exist for completed work, with clean owned diff.
- [x] Checkpoint report includes requirement→test mapping, limitations, Git commits.
- [x] **STOP**; await explicit authorization for the next assignment.

## 8. Stage D — Wordle extension [after both earlier games are merged]

**Prompt:** [`../prompts/WORDLE_AGENT_PROMPT.md`](../prompts/WORDLE_AGENT_PROMPT.md). **Branch:** `feat/wordle`, created from current local `main`. This stage is an extension authorized by the user's decisions; do not attribute its rules to a supplied assignment PDF.

### D0. Prerequisites and reuse inventory

1. Verify the Spelling Bee and Boggle code, review reports, benchmark evidence, and completed commits are present in `main`; run the complete existing workspace baseline. A directory or branch name alone does not prove that a game was merged or passed review.
2. Read the sibling Mastermind source, not only its design notes. Check its Git state without modifying it. Record the source commit when available and hashes of reused files in `MASTERMIND_REUSE_MAP.md`. The uploaded ZIP has no Git history, so the planning map deliberately contains no invented source SHA.
3. Confirm that the target repository has no dependency on the sibling repository. Create the `wordle` engine and its CLI adapter only now. Reuse the terminal/replay components already added by earlier game branches.
4. Update each planned reuse-map row to its actual destination and disposition as code is ported; copied, adapted, design-only, and deliberately unused must remain distinct.

### D1. Validated words, lexicon and a minimal playable round

- Add a private validated five-letter representation using existing `Letter`/normalization facilities. Public string inputs return typed validation errors; expected bad input must not panic.
- Extract all five-letter allowed guesses from `&Trie` into a sorted unique vector. Build the separately curated answer subset and checked `--answers` override loader, including duplicate accounting. Document answer-data provenance.
- Own this lexicon in the engine; keep secret, history, attempts and terminal state private. Permit deterministic construction for tests and injected seeded random answer selection for play.
- Implement a minimal complete round with WL-F1 next, then retain it while adding WL-F2. No numeric-difficulty menu or score multipliers are required.

### D2. WL-F1 — Mastermind-derived positional feedback baseline

- Adapt the exact-matches-first marker algorithm from `mastermind/src/scoring.rs::score_guess`. Preserve one-to-one target consumption. Return one mark for every guessed position, assigning partial matches left to right.
- Add table-driven known outcomes and the independent multiset-removal oracle before optimizing. Test duplicates in either/both words, all matches, no matches, all partial matches and excess occurrences.
- Reuse numeric Mastermind scoring only as an aggregate cross-check; its output alone cannot prove positional correctness. Record the corresponding source-to-target test mapping.
- Complete playable CLI feedback, normal guess validation, attempts, quit and replay using WL-F1. Compile, test, run a benchmark smoke, and commit this working baseline before D3.

### D3. WL-F2 — Frequency-count optimization and retained comparison

- Add a fixed 26-entry remaining-target frequency implementation. Exclude exact matches before allocating partial matches left to right. Keep WL-F1 runnable via the strategy option.
- Exhaustively compare WL-F1, WL-F2 and the independent oracle for all `3^5 × 3^5 = 59,049` pairs; run seeded full-alphabet and real-word randomized cases. Verify aggregate Bulls/Cows equivalence separately.
- Benchmark equivalent output arrays at length five with fixture creation and I/O outside timing. Record raw samples, batch sizes, environment, setup and the measured uncertainty; a small or absent speedup is a valid result.

### D4. Session/UI integration and comprehensive verification

- Six valid attempts; invalid input leaves attempts/history unchanged; repeated valid guesses consume turns. Preserve the final-attempt win rule and reject submissions after terminal state.
- Show positional feedback and a monotonic letter-status keyboard in ANSI and plain text. Report attempts and win/loss; reveal the answer on loss/quit. EOF terminates normally.
- Add Wordle to the existing launcher/subcommands without replacing earlier games or breaking `trie-cli`. Avoid arbitrary commands being confused with playable words; use slash commands.
- Verify tiny answer-file fixtures, all-rejected overrides, seeded reproduction, replay without lexicon cloning per guess, malformed input, mixed case, non-ASCII, score-free win/loss summaries, and unchanged prior-game behavior.

### D5. Extension checkpoint

- Run all gates in section 10. Verify WL requirements below, update the existing README, TESTING and reuse map, execute both feedback benchmark modes, and obtain independent subagent review.
- Resolve findings and re-review changed code, merge `feat/wordle` locally into `main`, verify the merge, report real SHAs/results, and stop. Do not add hard mode, daily puzzles, TUI/GUI, persistence, or additional games.

## 9. Retained versions and requirements traceability

### 9.1 Version milestones

Use coherent commits on the same per-game branch. Each milestone below must compile, have its own meaningful correctness tests, run the relevant benchmark smoke, and leave the previous versions available at the final branch head. This is not a requirement to copy the engine into several directories or publish several crate major versions.

| Game | Milestone order | Required pre-merge comparison |
|---|---|---|
| Bee | SB-S1 filtering → SB-S2 trie pruning → SB-P1/P2 generator/difficulty → SB-H1 hints/UI | Complete set equality, solve/setup cost, anchor preprocessing and acceptance/retries |
| Boggle solving | BG-S1 per-word DFS → BG-S1E scan enumeration → BG-S2 trie enumeration | Complete sets on identical boards; setup plus varying query-trace lengths |
| Boggle generation | BG-G1 uniform → BG-G2 weighted → BG-G3 dice-like | Raw and playable-selection quality across all three board sizes; same solver/seed manifests |
| Wordle | WL-F1 marker baseline/playable round → WL-F2 frequencies → final integration | Complete feedback-array equality and real five-letter cost, not just aggregate counts |

Follow [BENCHMARKS.md](BENCHMARKS.md) for fixtures, metadata, sample policies, failures and output files. Initial smoke is not a substitute for the final comparative run. No required version may be “implemented” as a wrapper around the competing algorithm. Keep common validation/scoring factored, and retain genuinely independent search/feedback paths.

The original alternative-trie ladder remains a **separate deferred track**: current indexed arena baseline → boxed arrays (A) → adaptive `SmallVec` (D) → `HashMap`/`BTreeMap` variants, with identical load/lookup/churn/game workloads and memory measurements. Record it in the benchmark roadmap, but do not expand the current game's branch into that track.

### 9.2 Spelling Bee acceptance IDs

| ID | Required behavior | Minimum independent verification |
|---|---|---|
| SB-01 | Seven distinct letters, valid center, center-first display | Invalid sizes/duplicates/center; fixed and seeded cases |
| SB-02 | Minimum four letters, repetition, allowed letters and compulsory center | Positive/negative boundary and repeated-letter fixtures |
| SB-03 | Complete owned solutions from both retained solvers | Exact tiny expected sets and full-set differential tests |
| SB-04 | Pangram-anchored generation, fixed-center/count controls, bounded errors | Seed/config/hash reproduction, impossible constraints, retry exhaustion |
| SB-05 | Versioned calibrated difficulty presets | Calibration dataset/method/results and explicit override tests |
| SB-06 | Checked cached scoring and duplicate rejection | Pure score table; randomized invariant after every guess |
| SB-07 | Queen Bee and early-quit missed-word report | No premature finish; terminal-state immutability; complete missing set |
| SB-08 | Three-column alphabetical words and both hint views | Rendering/order tests; every hint marginal matches total solutions |
| SB-09 | Helpful interactive input, batch syntax, replay and EOF | Scripted subprocess sessions; plain/ANSI output; paths with spaces |
| SB-10 | Benchmarks, accurate docs, independent review and merge | Evidence files, resolved findings, full workspace and merged-main checks |

### 9.3 Boggle acceptance IDs

| ID | Required behavior | Minimum independent verification |
|---|---|---|
| BG-01 | Valid 4/5/6 boards and immutable eight-neighbor adjacency | Shape validation, corners/edges/center, symmetry, no duplicate/self edges |
| BG-02 | Dictionary words of ≥3 spelled letters; no cell reuse | Diagonal/reverse/multiple paths, illegal jumps, distinct equal-letter tiles |
| BG-03 | `Qu` = two letters/one tile and documented bare-Q limit | Path, minimum-length and score fixtures; 36th-bit coverage |
| BG-04 | BG-S1, BG-S1E and BG-S2 retained with complete results | Independent exact fixtures and full-set comparisons, no truncation on limits |
| BG-05 | Exact extended score table and immutable evaluated guesses | Every boundary including >15; duplicates; checked cached total |
| BG-06 | Uniform/weighted/dice generators for all board sizes | Determinism, distribution/data shape, one face per die, shuffled assignment |
| BG-07 | Bounded zero-word regeneration and honest failures | Raw empty board versus search failure; exhaustion; fixed empty boards |
| BG-08 | Immediate robust input, score summaries, replay/quit/EOF | Mixed delimiters, malformed-batch atomicity, no retraction, text fallback |
| BG-09 | Solver and board-quality benchmarks | Equal fixtures, amortized query traces, raw versus accepted draws |
| BG-10 | Docs, prior-game regressions, independent review and merge | Requirement evidence, resolved findings, full workspace and merged-main checks |

### 9.4 Wordle acceptance IDs

| ID | Required behavior | Minimum independent verification |
|---|---|---|
| WL-01 | Validated five-letter ASCII input and dictionary validation | Mixed case, punctuation/Unicode/length and absent word tests |
| WL-02 | Allowed guesses separate from curated answers | Subset invariant, nonempty answers, provenance, fail-safe overrides |
| WL-03 | Six valid attempts, repeats consume turns, invalid inputs do not | Attempt/history mutation checks for each rejection and repetition |
| WL-04 | Two retained exact-first duplicate-safe positional scorers | Known arrays, 59,049 exhaustive pairs, independent oracle and random tests |
| WL-05 | Correct final-attempt win and terminal-state guard | Win on sixth, loss after sixth, no seventh evaluation |
| WL-06 | Seeded answer selection and replay | Same seed/config/data reproduction; reusable owned lexicon |
| WL-07 | Letter rows and monotonic keyboard with plain fallback | No yellow/gray downgrade; excess duplicates; no ANSI in plain mode |
| WL-08 | Quit/loss reveals, EOF, no automatic next game | Scripted round and launcher/replay sessions |
| WL-09 | Component-by-component Mastermind reuse evidence | Actual symbols/paths, source hashes/commit, adapted tests, exceptions explained |
| WL-10 | Both feedback algorithms benchmarked fairly | Actual five-letter complete outputs, raw samples, setup excluded/labeled |
| WL-11 | Previous assignments remain working | Full workspace tests and examples for trie, Bee and Boggle |
| WL-12 | Accurate docs, independent review, local merge and report | Resolved findings, recorded review scope/commit and post-merge checks |

## 10. Per-game branch, independent review, and local merge protocol

```mermaid
flowchart TD
  M["Current main and required earlier checkpoints"] --> F["Own game branch: baseline and alternative versions"]
  F --> A["Author checks, benchmarks and documentation"]
  A --> R{"Fresh subagent review clear?"}
  R -->|Findings| X["Focused fixes and regression checks"]
  X --> R
  R -->|Yes| G["Merge reviewed game branch into main"]
  G --> P{"Post-merge checks pass?"}
  P -->|No| X
  P -->|Yes| S["Report evidence and stop"]
```

### 10.1 Branch preflight

1. Locate `Assignment 3/word-games` as an existing Git repository. Read applicable `AGENTS.md` files and the invoked game prompt. Locate the sibling Mastermind repository read-only. Do not initialize a replacement repository merely because the prompt was launched from the parent folder.
2. Inspect `git status --short`, current branch, `main`, log, worktrees, and existing feature branches. Preserve unrelated changes. Do not reset, auto-stash, clean, force-switch, or stage all user changes. If the named feature branch is an existing compatible partial implementation, inspect and resume it; do not duplicate or overwrite it.
   If this documentation overlay is newly applied but uncommitted, inspect its exact Markdown-only changes. They are authorized task files: carry them onto `feat/spelling-bee` and commit them there as a coherent documentation update before game code, while preserving unrelated user edits. Alternatively, use an already committed overlay on `main`. Do not require a separate approval just to commit the provided planning update.
3. Start from current **local `main`**, after required earlier games are already reviewed and merged. Record the base commit, the new branch name, dictionary hash and Mastermind source state. No network fetch/pull/push is required by this workflow.
4. Run the pre-existing baseline checks and record failures as pre-existing or introduced with evidence. Do not declare a new checkpoint complete with unresolved required failures. Fix relevant pre-existing defects on this branch only when needed, with focused tests and a documented reason.
5. Create `feat/spelling-bee`, `feat/boggle` or `feat/wordle` from `main`. Keep all work for that game, its tests, benchmark data and docs on that branch. Use small Conventional Commits; do not work directly on `main` or edit Mastermind.

### 10.2 Author completion and fresh subagent review

After all required versions and game behavior are implemented, run:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --release
cargo build --workspace --release
git diff --check
```

Also run the game's full comparative benchmark suite, deterministic CLI transcripts, rustdoc examples, and every current README usage example. Review docs against actual manifests, source and output; preserve old checkpoint evidence with its date, and add a new dated/commit-scoped section. Do not reuse the inherited 61-test count as a new result.

Spawn a **fresh independent reviewer subagent** after this author pass. Give it the absolute repository path, base commit, reviewed code commit, the relevant requirement IDs, exact authoritative files, and instructions below. Prefer a clean/minimal context rather than the author's whole reasoning history. Do not change the model solely for review. The author must not edit files while the reviewer reads or runs commands.

The reviewer must:

- Read the original assignment for Bee/Boggle, approved design additions, implementation plan, benchmark protocol, testing record, current source, and `git diff <base>...<reviewed-code-commit>`. For Wordle, read the Mastermind source and reuse map as well. Treat source and requirements as evidence, not the author's completion claim.
- Independently run formatting, compilation, strict Clippy, debug/release tests, release build, representative CLI transcripts, documentation examples, and benchmark smoke checks. Inspect the saved full benchmark inputs/results and reproduce at least representative comparisons. Record exact commands and exit results.
- Check algorithm independence, complete set/feedback equality, duplicate rules, ownership/snapshot semantics, terminal-state guards, corpus/answer/dice provenance, benchmark fairness and failures, scope, branch ancestry, and accurate documentation. Add adversarial reasoning and targeted manual probes beyond the existing happy-path tests.
- Return a structured report with severity, file/line, violated requirement, reproduction, expected versus actual behavior, and remediation. Explicitly enumerate all covered requirement IDs, untested gaps and any environment blockers. “No findings” alone is insufficient evidence.
- Work read-only except build/test outputs. Suggest focused tests or patches; the author owns implementation fixes. It must not merge, rewrite history, or certify the author's statements without its own inspection.

Save the report at `docs/reviews/spelling-bee.md`, `docs/reviews/boggle.md` or `docs/reviews/wordle.md`, with the actual reviewed code SHA, base SHA, reviewer identifier, review date, commands, results, coverage and findings. Use [`reviews/REVIEW_TEMPLATE.md`](reviews/REVIEW_TEMPLATE.md); it is a template, not a completed review. If subagents are unavailable, mark `BLOCKED: independent review unavailable`, preserve the branch and report the blocker; do not relabel an author self-review as independent or merge without the required review.

### 10.3 Remediation, re-review and merge

1. Resolve every actionable finding. Fix correctness, assignment, data, benchmark or documentation gaps with tests and focused commits. Mark non-actionable observations with evidence and have the reviewer confirm that disposition; do not quietly waive requirements or known defects.
2. Re-run relevant and full required checks after code changes. Ask the reviewer to re-inspect the fix commits and rerun affected checks. Any source, test, asset, dependency, executable benchmark or behavior change after review requires review of that delta. A later report-only commit may record the reviewed code SHA; do not fabricate a self-referential report commit SHA.
3. Ensure the final README and TESTING describe the branch as it exists, the reuse map uses actual destinations, and required benchmark data/reports are present. A required skipped/blocked check is not a pass. Include the review's remediation status and remaining limitations honestly.
4. Inspect current `main`. If it advanced during work, merge it into the feature branch without rewriting history, resolve conflicts, repeat affected checks and obtain review of resulting changes. Do not merge against an unreviewed conflict resolution.
5. With owned changes committed and no unrelated dirty files at risk, switch to `main` and run `git merge --no-ff <game-branch>` with a Conventional Commit merge message such as `feat(spelling-bee): merge reviewed game implementation`. The user has requested these local per-game merges; do not add another confirmation gate. Do not push, delete branches, rewrite history or publish remotely as part of these prompts.
6. On merged `main`, rerun the full required checks above, the game's benchmark smoke and CLI/documentation smoke. If a merge-specific failure appears, stop progression, repair it on the retained game branch through a reviewed corrective commit and merge again; do not reset or rewrite the merge.
7. Report branch/base/reviewed-code/merge SHAs, requirement coverage, actual tests and benchmark comparisons, review findings and resolutions, verified README examples, and limitations. Leave `main` checked out and the game branch retained for traceability. **STOP**; the next game is a separate prompt invocation.

### 10.4 Fresh completion checklist — one per game

- [ ] Baseline and every required alternative algorithm remain runnable and tested.
- [ ] Every active requirement ID maps to actual code and passing verification.
- [ ] Fair benchmarks ran with saved metadata/raw data; failures and uncertainty are included.
- [ ] Existing README, design, plan and TESTING were updated rather than replaced; examples were executed.
- [ ] Mastermind reuse map is reconciled to source symbols and actual target code.
- [ ] A fresh subagent inspected code/diff and ran independent checks.
- [ ] Review findings were resolved and fixes re-reviewed; no required check is blocked/skipped.
- [ ] Conventional Commits and clean owned diff; Mastermind and unrelated user files unchanged.
- [ ] Own game branch merged locally to `main`; post-merge checks passed.
- [ ] Final report uses real Git evidence and stops before the next game or extension.
