# Implementation Agent Prompt — Boggle

Use this prompt in the existing `Assignment 3` workspace. 

You are the **senior Rust engineer and test engineer** implementing the multi-assignment *Word Trie and Word Games* repository. Follow the provided written requirements precisely, with correct Rust ownership, safe arena allocation, reproducible tests, and accurate documentation.

 **Implement Boggle only**, including its retained solver versions and uniform/weighted/dice-like generation increments. Then obtain independent verification from a fresh subagent, resolve findings, merge the reviewed branch locally into `word-games/main`, and stop. Wordle is a separate later invocation. These local branch/merge steps are already authorized; do not add another confirmation gate.

## 1. Prerequisites and authoritative context

Locate `Assignment 3/word-games` and its read-only sibling `Assignment 3/mastermind`; quote paths containing spaces. Inspect applicable `AGENTS.md`, Git status/history/branches, actual code/manifests/tests, and read:

1. `assignments/Boggle.pdf` and existing assignment-one/Spelling Bee obligations for regressions.
2. Existing `docs/DESIGN_SPECIFICATION.md`, especially sections 15–20.
3. Existing `docs/IMPLEMENTATION_PLAN.md`, Stage C and sections 9–10, including BG-01…BG-10.
4. Existing `README.md`, `docs/TESTING.md`, `docs/BENCHMARKS.md`, `docs/MASTERMIND_REUSE_MAP.md` and the review template.
5. The actual trie cursor/visitor, merged Spelling Bee code, shared CLI terminal/replay utilities, completed review and benchmark evidence.

**Require Spelling Bee already reviewed and merged into local `main`.** Verify actual code, review record and commit ancestry, not just a branch name or feature directory. Run the existing baseline. If the prerequisite is absent, report it and stop this prompt rather than implement Bee on the Boggle branch. Archived Assignment 1 prompt wording does not override this invocation.

## 2. Branch and implementation boundaries

- Create **`feat/boggle` from current local `main`**, or safely resume its compatible existing partial branch. Record the base SHA, toolchain, corpus hash and baseline checks. Preserve unrelated user work; no reset/clean/auto-stash/force-switch or indiscriminate staging.
- Keep existing trie, `trie-cli`, Bee and their examples working. Add an independent `boggle` engine and adapt the existing `word-games-cli`; do not replace the launcher or rebuild a second CLI framework.
- Reuse applicable terminal, replay, validation and RNG-injection components already adapted from Mastermind. Reconcile actual destinations/tests in the reuse-map ledger. Mastermind remains untouched and must not become a build dependency.
- Engines contain no terminal I/O or global RNG. Session setup temporarily borrows `&Trie`, then owns the full sorted unique solutions, board, guesses and checked cached score. No cloned trie, persistent trie borrow, unnecessary reference counting/interior mutability or speculative shared framework.
- Work in small tested Conventional Commits (`feat(boggle)`, `test(boggle)`, `perf(boggle)`, `feat(cli)`, `docs`). Each version is a working tested milestone; retain all required implementations at the final branch head.

## 3. Implement in dependency order

### Milestone BG-S1 — Board and on-demand baseline

- Validate runtime board sizes 4, 5 and 6 and exact tile count. Use immutable flattened `Vec<Tile>` and precompute symmetric eight-neighbor adjacency with no self/duplicate edges.
- Track visited cells with copied `u64` masks, including bit 35. Reusing one cell in a word is forbidden; using two different cells with the same letter is valid.
- Q input/generated faces normalize to `Qu`: advance two spelled letters while consuming one cell/bit. Count spelled letters for minimum length and scoring. Bare-Q gameplay and Q-without-U dictionary words remain a documented limitation.
- Implement **independent per-word board DFS first**, with dictionary lookup in the on-demand validator. Path search must not use trie-prefix pruning or call the later production solver. Test exact paths, diagonal/reverse movement, illegal jumps, repeated letters, multiple paths and `Qu` before adding enumeration.
- Add BG-S1E by scanning dictionary words and applying the independent path search to generate a complete sorted unique snapshot. This is the `scan` solution strategy and correctness/fallback baseline. It is not a persistent dictionary-borrowing game session.
- Implement minimal uniform boards, a playable untimed round, immediate feedback, unique accepted guesses, score and quit. Keep exact fixed boards for deterministic tests.

Compile/test, run a scripted session and baseline benchmark smoke, and commit the complete baseline before BG-S2.

### Milestone BG-S2 — Trie-guided enumeration

Start DFS at each tile, carry an immutable trie cursor and copied visited mask, prune nonexistent prefixes, advance `Q` then `U` for `Qu`, and record terminal words of at least three letters. Deduplicate by word, not path. Keep recursion/search work bounded by explicit validated input and any documented expansion guard.

Compare **complete sorted solution sets** against BG-S1E on hand-built exact boards, seeded small dictionaries/boards and representative full-corpus boards at every size. Include pathological repeated-letter boards and zero-word cases. A limit/timeout must return an explicit error with **no published partial session**; never return a truncated set or label a search failure as zero solutions.

Retain `scan` and `trie` strategy options; make trie the default. Keep BG-S1 callable independently for tests and query-trace benchmarks. Run equality and benchmark smoke before committing this milestone.

### Milestone BG-G1 — Uniform generation and normal-play policy

Retain independent uniform A–Z sampling, mapping Q to `Qu`. Accept an injected seeded RNG and record generator/config/data versions. Raw generators may yield zero-word boards. Normal random play uses a documented finite retry budget to choose a nonempty board and returns `NoPlayableBoard` on exhaustion. Fixed-board APIs retain empty boards for tests. Keep solver errors distinct from legitimate empty results.

### Milestone BG-G2 — Frequency-weighted generation

After uniform/core tests pass, add a selectable weighted generator. Derive per-letter counts from supplied dictionary spellings with add-one smoothing; commit versioned weights, derivation code/method, corpus hash and reproducibility tests. Explain the dictionary's sampling bias; do not call these natural-language usage frequencies. Do not replace the uniform implementation.

### Milestone BG-G3 — Dice-like generation

After BG-G2 passes, add selectable synthetic six-face dice for **all three board sizes**. Follow design section 18: allocate `6*N*N` faces using weighted largest remainders with alphabetical ties, deterministic round-robin assignment to `N*N` dice, Q→`Qu`; choose one face per die and seeded-shuffle assignments to cells. Commit generated tables, derivation/provenance and hashes. Do not invent official commercial dice data or claim these are physical Boggle dice.

Test six faces and valid symbols per die, exact die count, one draw per die, correct cell count, shuffle permutation and stable reconstruction. Keep uniform/weighted/dice selectable, with uniform still the default. Run tests and benchmark smoke at each generation milestone.

### Finished gameplay and UI

- Preserve the agreed score table: 3–4→1, 5→2, 6→3, 7→5, 8→11, 9–12→2×length, 13–14→3×length, **≥15→4×length**. Include a realizable >15-letter custom-dictionary fixture; do not cap generic trie words at 15. Use checked arithmetic and maintain cached-total equality.
- Give immediate per-word feedback. Invalid/duplicate guesses never earn points; an evaluated guess cannot be retracted. After quit/completion, further submissions cannot change session state. Use an honest `NotASolution` message when a frozen snapshot cannot distinguish dictionary absence from board absence.
- Add Boggle to the existing launcher/subcommands. Support size, generator, solver, dictionary, seed, color and fixed-board options as specified in the design. No Wordle placeholder crate or playable menu item.
- Accept comma/ASCII-whitespace batches. Prevalidate malformed syntax before mutation, evaluate valid tokens in order and stop on terminal completion. Use slash commands, useful errors, board/score/accepted-word display, missed-word summary, replay and clean EOF. Plain mode must remain useful; avoid ANSI on redirected output in auto mode.
- Keep gameplay untimed. Timers, human-calibrated Boggle difficulty and rich TUI/GUI are deferred, while the three generator versions above are **required in this branch**.

## 4. Tests, benchmarks and existing documentation

Cover **BG-01…BG-10**, `docs/TESTING.md`'s Boggle matrix and all earlier game regressions. Add meaningful unit, public integration, seeded randomized/differential, corpus, adversarial, CLI subprocess and rendering tests during each milestone. Use at least three recorded random seeds and deliberate restricted-alphabet fixtures to stress cell reuse/backtracking. Compare whole sets, cached scores and immutable state after rejected/terminal submissions.

Implement and run:

```bash
cargo bench -p boggle --bench algorithms -- --quick --suite all
cargo bench -p boggle --bench algorithms -- --full --suite all
```

Follow `BENCHMARKS.md` and save metadata/cases/raw samples/report under `docs/benchmarks/boggle/<run-id>/`. Required studies:

1. Complete BG-S1E versus BG-S2 enumeration on identical boards.
2. Setup-inclusive trie snapshots versus on-demand membership/path queries for shared traces of varying lengths; report whether/when setup is repaid.
3. Uniform versus weighted versus dice-like generation across sizes 4/5/6, using the same solver, recording raw word/score/length/vowel/zero-word/failure statistics and separately playable-selection retries/accepted-board statistics.

No omitted setup cost, guessed memory measurements, discarded failures, hidden conditioning on good boards or assumed “better” generator. Default full generator study is 100 raw boards per generator/size (900 boards); smoke may be smaller. Document budgets/common reduced expensive workloads and their limits honestly, with reviewer scrutiny.

Before review run formatting, all-target compile-check, strict all-target Clippy, debug tests, release tests, release build and `git diff --check` using plan section 10's exact commands. Run doctests, every current README usage example and scripted trie/Bee/Boggle launcher sessions. **Update the existing** README, design, plan and TESTING in place; preserve prior dated evidence and record actual new commands/results/commits, version milestones and limitations. Never replace the documents wholesale or claim planned cases passed.

## 5. Mandatory independent code review via subagent

Commit the candidate after the author checks and full comparative studies, then **spawn a fresh independent reviewer subagent** with the repository path, base SHA, candidate SHA and authoritative paths. Prefer a clean/minimal context; pause edits during its inspection. It must not certify only from your summary or logs.

Reviewer task:

> Independently review the Boggle source and base-to-candidate diff against `Boggle.pdf`, the approved design/plan, BG-01…BG-10, testing matrix, benchmark protocol and current CLI behavior. Run your own formatting/all-target check/strict Clippy/debug and release tests/release build, prior-game and Boggle CLI/README examples, benchmark smoke and representative comparative reproduction. Probe adjacency, illegal cell reuse, repeated tiles, `Qu`, bit 35, >15-letter scoring, duplicate/terminal/cached-score rules, independent solver completeness, search-limit handling, all three generator/data derivations, zero-board retry policy, snapshot ownership, input/EOF/replay, reuse provenance, raw-versus-accepted quality statistics and amortized-query fairness. Report exact independent commands/results, requirement coverage/gaps, and severity/file/line/minimal reproduction/expected-versus-actual for findings. Remain read-only except generated build/test outputs; do not merge.

Store the real report at `docs/reviews/boggle.md` with reviewer identity/date, base and reviewed-code SHA. Resolve actionable findings with focused commits/regressions and obtain re-review of fixes and later behavior changes. Record non-actionable dispositions only with evidence/reviewer agreement. Missing subagent capability or a blocked required check means **BLOCKED**; preserve the feature branch and report it, without claiming self-review is independent or merging anyway.

## 6. Merge locally, verify and stop

After all gates pass, follow plan section 10 to merge **`feat/boggle` into local `main` with `--no-ff`** and a Conventional Commit message. If `main` advanced, first merge it into the feature branch, resolve conflicts and re-review affected code. No remote push, destructive reset, history rewriting or branch deletion.

Run all required workspace checks again on merged `main`, plus benchmark, documentation and CLI smoke. Report real branch/base/reviewed-code/merge SHAs; BG requirement mapping; tests/seeds; retained solvers/generators and measured comparisons; reviewer findings/fixes; updated examples; limitations. Leave `main` checked out and retain the game branch. **Stop; do not implement Wordle, timers, alternative trie backends or rich TUI/GUI.**
