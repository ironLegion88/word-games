# Implementation Agent Prompt — Wordle Extension with Mastermind Reuse

Use this prompt in the existing `Assignment 3` workspace. 

You are the **senior Rust engineer and test engineer** implementing the multi-assignment *Word Trie and Word Games* repository. Follow the provided written requirements precisely, with correct Rust ownership, safe arena allocation, reproducible tests, and accurate documentation.

 **Implement Wordle only after Spelling Bee and Boggle are both independently reviewed and merged into `word-games/main`.** Reuse as much suitable code and logic as practical from the sibling Mastermind implementation through deliberate, documented adaptation. Then obtain independent verification from a fresh subagent, resolve findings, merge this game's reviewed branch locally into `main`, and stop. These local branch/merge steps are authorized; no additional merge confirmation is required.

## 1. Read actual repositories and requirements

Locate the existing independent repositories `Assignment 3/word-games` and `Assignment 3/mastermind`. Read applicable `AGENTS.md`, inspect Git status/branches/history/worktrees, and read:

1. Existing `docs/DESIGN_SPECIFICATION.md`, especially sections 15–20 and the Wordle contract in section 19.
2. Existing `docs/IMPLEMENTATION_PLAN.md`, Stage D and sections 9–10, including WL-01…WL-12.
3. Existing `README.md` and `docs/TESTING.md`, current game code/manifests/tests, review and benchmark evidence for both earlier games, and the original assignments for regression obligations.
4. `docs/BENCHMARKS.md`, `docs/MASTERMIND_REUSE_MAP.md` and `docs/reviews/REVIEW_TEMPLATE.md`.
5. **Actual sibling Mastermind source:** `src/scoring.rs`, `src/game.rs`, `src/config.rs`, `src/main.rs`, `src/ui.rs`, tests embedded in those modules, `Cargo.toml` and relevant docs. Do not rely on the map or design summary without inspecting the code.

Wordle is the approved project extension; no supplied PDF is its source of rules. The dedicated prompt and approved supplements override historical “future Wordle” wording in the archived Assignment 1 prompt. Do not rebuild earlier assignments or regenerate their documents.

## 2. Preconditions, branch and component reuse

- Require real merged Bee/Boggle implementations, completed review reports and benchmark evidence in local `main`; check actual code and ancestry. Run the existing workspace baseline. If either prerequisite is absent, report it and stop rather than implementing a prior game here.
- Create **`feat/wordle` from current local `main`**, or safely resume a compatible existing branch. Record base SHA, baseline results, dictionary hash and source Mastermind commit/hashes. Preserve unrelated user changes; no reset/clean/auto-stash/force-switch or broad staging.
- Keep **Mastermind read-only and independently buildable**. It is currently a binary crate with `main.rs`, not a library dependency. Port narrow components into `word-games`; no sibling Cargo dependency, `include!`, symlink, source-repo refactor or hidden build step.
- Follow every applicable row of `MASTERMIND_REUSE_MAP.md`. Reuse its exact-first duplicate-safe matching logic, session/attempt/replay structure, injected RNG/I/O seams, ANSI/plain utilities and test patterns where suitable. Reuse shared CLI components already ported by Bee/Boggle instead of copying another set.
- Record actual source symbols/commit/hashes, target paths/symbols, copied versus adapted versus design-only versus intentionally excluded status, adapted tests and implementing commits. Do not claim new lexicon/frequency/keyboard behavior was copied from Mastermind. Explain justified exclusions rather than forcing numeric abstractions into Wordle.
- Make small Conventional Commits with `feat(wordle)`, `test(wordle)`, `perf(wordle)`, `refactor(cli)` and `docs` scopes. Retain every required feedback implementation in the final code.

## 3. Implement in dependency order

### Milestone WL-L1 — Validated words, owned lexicon and round state

Create the independent `wordle` engine and its adapter in the existing `word-games-cli`. Keep engine I/O-free and use private validated types/fields.

- Exactly five ASCII letters. Reuse existing normalization/`Letter` facilities. Word syntax errors return typed results; normal user input must not trigger the Mastermind-style length assertions.
- Build an owned sorted unique vector of allowed five-letter guesses from a temporary `&Trie`. Select answers from a **separate curated subset** that is checked against this vector. Supply a locally curated bundled list (target at least 100 validated entries) and provenance/curation notes. Do not download or invent an official Wordle vocabulary or claim frequency ranking without evidence.
- Support `--answers <path>` override with complete validation before mutation/start: normalize, report duplicate count, require nonempty five-letter dictionary entries, and reject the entire malformed override. Tiny user/test lists are valid; the bundled-list target does not impose a 100-word minimum on overrides.
- The game owns the lexicon and private secret/history/attempt/status data; reuse the owned lexicon across replay resets. No persistent trie borrow, trie cloning per guess/round, reference-counting workaround or publicly mutable secret/counters.
- Use an injected seeded RNG to select answers from the validated answer subset. A deterministic constructor with a checked explicit target may support tests. Record data/RNG/config versions with reproduction seeds.

### Milestone WL-F1 — Mastermind-derived feedback and a working game

Adapt `mastermind/src/scoring.rs::score_guess` into an independently callable **marker-based positional scorer**:

1. Mark every exact position as `Correct` and consume those target occurrences first.
2. Traverse remaining guessed positions **left to right**. Consume one matching unused target occurrence and emit `Present`, or emit `Absent` if none remains.
3. Return the full five-position array, not only Bulls/Cows totals. Small fixed marker arrays can replace heap vectors while preserving the algorithm; document that adaptation.

Use an independent test oracle based on an unmatched-target multiset with occurrence removal, without calling either production scorer. The original numeric score function is useful as an **aggregate cross-check** only: correct marks = Bulls, present marks = Cows. It cannot establish positional assignment by itself.

Implement a complete playable round with WL-F1 before adding the optimization:

- Six valid attempts, repeated letters allowed. Invalid syntax/length/non-dictionary guesses consume **no** turn. Repeating a valid guessed word consumes a turn and yields feedback again.
- Store history/attempts consistently; check a correct sixth guess before loss. Persist terminal state and reject any post-finish submission without modifying attempts/history.
- Quit/loss reveal the answer; EOF exits cleanly. Report win/loss and attempts. Do **not** inherit numeric difficulty menus, unconstrained numeric target sampling, score multipliers or mutable public session fields.
- Reuse applicable replay/rendering/injected-I/O organization, with slash commands and correct lexical validation.

Run table-driven exact-feedback tests, session/CLI smoke and a baseline benchmark smoke; commit this working version. Keep WL-F1 available via `--feedback marker` and the engine API after WL-F2 lands.

### Milestone WL-F2 — Frequency-count feedback optimization

Implement a separate 26-entry remaining-target-frequency scorer. Exclude exact matches before counting/distributing partial matches; assign remaining matches left to right. Return the same five marks as WL-F1; do not wrap/call WL-F1 as the “optimized” implementation.

Before setting the production default, compare the **complete arrays** from WL-F1, WL-F2 and the independent oracle:

- All 59,049 pairs over `{A,B,C}` at length five.
- At least three recorded seeded full-alphabet streams and real dictionary/answer pairs.
- Known duplicate traps including `APPLE` against `ALLEY`, `LEVEL` against `EERIE`, all/no matches, partial/exact mixtures, target multiplicity smaller than guess multiplicity and excess occurrences after exact matches.
- Independent aggregate Bulls/Cows corroboration using an attributed test-only adaptation or equivalent isolated reference, without a dependency on the sibling binary crate.

Retain both scorers and make frequency counting the production default. A small or nonexistent five-letter speedup is acceptable; correctness and reproducibility are required.

### Milestone WL-UI1 — Integrated presentation and replay

- Add `wordle` to the existing launcher/subcommands; preserve all trie/Bee/Boggle behavior and shared terminal/replay helpers. Options include dictionary, answers, seed, color and feedback strategy; gameplay stays five letters/six valid attempts in both modes.
- Render five marked cells per accepted guess and a letter-status keyboard. Evidence must be monotonic: `Correct > Present > Absent > Unknown`; a gray duplicate must never downgrade an already yellow/green letter.
- Give clear validation feedback and useful plain-text equivalents for every mark; respect `NO_COLOR`, terminal-aware auto mode and explicit color override. Test output without relying only on ANSI colors.
- Read one word per line for Wordle. Slash commands are separate from words. Replay starts another checked answer with the same settings or returns to the launcher; reuse the lexicon and reset only round state. No daily/online puzzles, hard mode, persistence, point system or TUI/GUI.

## 4. Testing, benchmark evidence and existing documentation

Satisfy **WL-01…WL-12** and the Wordle matrix in `docs/TESTING.md`. Add unit, public integration, exhaustive/differential, seeded random/corpus, invalid input, state transition, rendering, injected-I/O and actual subprocess tests throughout. Verify no attempt/history mutation on rejection or terminal submission, final-attempt win, repeated-valid consumption, override atomicity, allowed/answer subset invariants, deterministic selection, keyboard precedence and unchanged earlier games.

Implement and run both benchmark modes:

```bash
cargo bench -p wordle --bench algorithms -- --quick
cargo bench -p wordle --bench algorithms -- --full
```

Follow `docs/BENCHMARKS.md`. Compare identical **validated five-letter pairs and full output arrays**, consume results, keep construction/I/O outside kernel timing, use warm-up/batching/repeated paired measurements, and save cases/raw samples/metadata/report under `docs/benchmarks/wordle/<run-id>/`. Include repeated-letter strata and measured spread. Any longer generalized-kernel stress study must be separate and must not imply variable-length Wordle gameplay. Do not claim a large optimization win from asymptotics or a single timing.

Before review run:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --release
cargo build --workspace --release
git diff --check
```

Execute doctests and every current README example plus scripted trie/Bee/Boggle/Wordle launcher sessions. **Edit the existing** README, design specification, implementation plan and TESTING in place, preserving their checkpoint history and updating current capabilities/examples/evidence. Reconcile the entire Mastermind reuse map to actual source/target code, tests and excluded behaviors. Never substitute a new clean document for the existing one or fabricate pass counts, benchmarks, reviewed commits or source attribution.

## 5. Mandatory independent review via subagent

Commit the candidate after author checks and comparative benchmarks, then **spawn a fresh independent reviewer subagent**. Prefer minimal/fresh context with repository path, base SHA, reviewed candidate SHA and authoritative file paths. Pause editing during review. It must read source and run its own checks, not simply accept your implementation summary or logs.

Reviewer task:

> Independently review the Wordle extension and base-to-candidate diff against approved design section 19, the plan, WL-01…WL-12, testing matrix and benchmark protocol. Read the sibling Mastermind source and component reuse map; verify actual adapted symbols, provenance, excluded numeric behavior and that the sibling repository is unchanged/not a dependency. Run your own formatting/all-target check/strict Clippy/debug and release tests/release build, exhaustive/differential feedback tests, adversarial Wordle and earlier-game CLI/README examples, benchmark smoke and representative comparison reproduction. Probe exact-first positional multiplicity, left-to-right partial assignment, repeats/invalid-turn rules, fifth-letter/length bounds, answer-subset/override behavior, final-attempt win, post-finish guard, private ownership/reset semantics, keyboard precedence, EOF/replay and benchmark fairness. Report actual commands/results, requirement coverage/gaps and severity/file/line/minimal reproduction/expected-versus-actual for findings. Treat author claims as unverified. Remain read-only except generated build/test artifacts; do not merge.

Save its actual report as `docs/reviews/wordle.md`, with reviewer identity/date, base and reviewed-code SHA. Resolve actionable findings using focused commits and regressions, then request independent re-review of fixes and subsequent code/data/behavior changes. Record any non-actionable disposition with evidence and reviewer agreement. If subagent capability or a required check is unavailable, mark **BLOCKED**, preserve the branch, report it and do not self-certify or merge.

## 6. Merge locally, recheck and stop

After all required evidence/review gates pass, follow plan section 10 to merge **`feat/wordle` into local `main` with `--no-ff`** and a Conventional Commit merge message. If `main` advanced, first merge it into the feature branch, resolve conflicts and re-review affected changes. No remote push, destructive reset, shared-history rewrite or branch deletion.

Run full required workspace checks again on merged `main`, plus benchmark/CLI/documentation smoke and all earlier games' usage examples. Report actual base/feature/reviewed-code/merge SHAs; WL coverage; exact tests/seeds and exhaustive count; retained algorithms and measured comparison; component reuse evidence; reviewer findings/resolutions; verified docs and limitations. Leave `main` checked out and retain the game branch. **Stop. Do not add further games, hard mode, timers, alternative trie storage, rich TUI/GUI, persistence or online features.**
