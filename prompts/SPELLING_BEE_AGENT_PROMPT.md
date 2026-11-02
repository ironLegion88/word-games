# Implementation Agent Prompt — Spelling Bee

Use this prompt in the existing `Assignment 3` workspace. 

You are the **senior Rust engineer and test engineer** implementing the multi-assignment *Word Trie and Word Games* repository. Follow the provided written requirements precisely, with correct Rust ownership, safe arena allocation, reproducible tests, and accurate documentation.

**Implement Spelling Bee only**, then obtain independent code verification from a fresh subagent, resolve its findings, merge the reviewed branch locally into `word-games/main`, and stop. Assignment 1 is already complete. This prompt is authorization for the local branch/merge workflow below; do not add another permission gate before that merge. Boggle and Wordle are separate later prompt invocations.

## 1. Read and inspect before editing

Locate the two existing sibling Git repositories: `Assignment 3/word-games` and `Assignment 3/mastermind`. Work in `word-games`; treat `mastermind` as a read-only reference. Read applicable `AGENTS.md` files, inspect Git status/branches/history, existing source/tests/Cargo manifests, and then:

1. `assignments/Spelling Bee.pdf` for the actual assignment; `assignments/Word Games.pdf` for preserved baseline obligations.
2. Existing `docs/DESIGN_SPECIFICATION.md`, including the approved supplements in sections 15–20.
3. Existing `docs/IMPLEMENTATION_PLAN.md`, especially Stage B and sections 9–10.
4. Existing `README.md` and `docs/TESTING.md`, including the planned Bee matrix and historical evidence distinction.
5. `docs/BENCHMARKS.md`, `docs/MASTERMIND_REUSE_MAP.md` and `docs/reviews/REVIEW_TEMPLATE.md`.
6. Actual `trie` API/source and applicable Mastermind UI/replay/injected-I/O components. Do not implement against guessed or outdated method signatures.

The current prompt, latest user decisions and approved design supplements govern this stage. `AGENT_PROMPT.md` retains an archived Assignment 1 prompt; its historical “sole active assignment” and “do not implement games” wording does not override this invocation. Preserve already completed code. Do not rebuild the trie or regenerate existing documentation from scratch.

## 2. Branch, baseline and ownership

- Follow implementation-plan section 10. Inspect current local `main` and unrelated user edits. Record the base commit, dictionary hash, toolchain and baseline check results. Do not reset, auto-stash, clean, overwrite or stage unrelated work.
- Create **`feat/spelling-bee` from current `main`**. If an existing compatible branch contains partial work, inspect and resume it safely. Every game change, test, benchmark and documentation update belongs on this branch until reviewed.
- If the provided documentation overlay is applied but uncommitted, inspect and carry those authorized Markdown changes onto this branch and commit them before game code; do not stage unrelated edits or require an extra confirmation for this planning update.
- Preserve `trie`/`trie-cli` and their regression suite. Keep Mastermind independently buildable and unchanged. No sibling Cargo dependency, cross-repo `include!`, copied `.git` directory or symlink.
- Use small Conventional Commits with scopes such as `feat(spelling-bee)`, `test(spelling-bee)`, `perf(spelling-bee)`, `feat(cli)` and `docs`. Each algorithm milestone must compile, have meaningful tests, run a benchmark smoke, and remain usable after later milestones.

## 3. Implement in this order

### Milestone SB-S1 — Complete reference functionality

Create the independent `spelling-bee` engine and only the necessary `word-games-cli` driver modules. Use concrete types and private fields; do not add empty Boggle/Wordle crates or a generic game framework.

- Validate exactly seven distinct ASCII letters with a designated center in that set, displayed first. Reuse `trie::Letter` and normalization. Minimum word length four, allowed letters only, center required, repetition allowed.
- Build the **full-dictionary filtering solver first** using the existing visitor and a 26-bit letter mask. Return complete sorted unique solutions. Establish tiny hand-computed fixtures before relying on a second algorithm.
- Session construction temporarily borrows `&Trie` and produces owned puzzle/solution state. The session owns frozen solutions, mutable `BTreeSet<String>` accepted guesses, checked cached score and explicit terminal state. No persistent trie borrow, trie clone, `Rc`/`Arc` or interior mutability.
- Implement immediate typed guess feedback, duplicate rejection, and a minimal playable terminal round. Score length four as 1, length ≥5 as length, plus 7 for a pangram. Invalid/duplicate input changes no score; cached total must equal recomputation from accepted words.
- Queen Bee requires all target words found. Early quit reports score and the complete missed set. Reject submissions after a terminal state without mutations. Handle fixed zero-solution fixtures honestly; normal play must not start an empty round or award instant Queen Bee.

Compile/test, run a deterministic transcript and baseline benchmark smoke, and commit before adding SB-S2. Keep SB-S1 available later as `--solver filter` and through the engine strategy API.

### Milestone SB-S2 — Trie-pruned solver

Implement a distinct cursor-based traversal that follows only the seven allowed-letter edges, reconstructs words and records valid terminal nodes. It must not call SB-S1 internally. Shared validation/scoring/types are appropriate; shared solver logic that makes the comparison circular is not.

Compare **entire sorted solution sets**, not only counts, across tiny exact fixtures, seeded seven-letter sets, zero-solution cases, repeated-letter words and sampled full-dictionary anchored puzzles/all seven centers. Assert trie contents/stats are unchanged. Make SB-S2 the default while retaining SB-S1.

Run the comparison benchmark smoke on identical saved inputs, document the milestone, and commit before extending generation.

### Milestones SB-P1 / SB-P2 — Anchored generation and controls

- Build a deterministic sorted index of dictionary anchor words containing exactly seven distinct letters. Select an anchor with an injected seeded RNG, derive its letter set and choose a center (or obey an explicit center override).
- Preserve/report the anchor-word sampling bias. Require at least one pangram, positive solution-count bounds and a finite retry budget for normal play. Impossible configurations or exhausted attempts produce clear typed errors, not loops or relaxed hidden constraints.
- Support reproducible `--seed`, explicit `--min-words`/`--max-words`, fixed center, and fixed letters plus center for fixtures/manual puzzles. Validate configuration before random work.
- After SB-S2 is correct, calibrate versioned easy/medium/hard completion-workload presets against the supplied corpus as specified in design section 17. Save the actual sampled cases, seed/hash, distributions, exact disjoint bands/tie handling and configuration asset. Do not invent measurements or claim human vocabulary difficulty has been measured. Explicit bounds intersect the selected preset; reject an empty intersection.
- Separate anchor preprocessing, candidate solving and retry/selection costs in benchmarks. Seed reproducibility includes RNG/lockfile/config/data versions, not just the seed integer.

### Milestone SB-H1 — Hints and finished CLI

- Include **both** initial-letter × word-length hint grid and two-letter-prefix counts, derived only from the frozen solutions. Default hints display labelled total-puzzle counts; also show total words/points/pangrams where useful. Verify every marginal sum.
- Continuously show accepted words alphabetically in three columns; do not rely on color to communicate required information. Adapt the useful Mastermind styling, replay and `BufRead`/`Write` seams, record actual reuse-map destinations/tests, and add terminal-aware auto color plus `NO_COLOR`/explicit modes.
- Provide `word-games-cli` launcher and `spelling-bee` subcommand with the common/game options from design section 16. Only implemented games may appear as playable choices. Keep `trie-cli` independent.
- Use `/help`, `/hints`, `/guesses`, `/quit`; normal words must remain guesses. Support comma/ASCII-whitespace batches. Prevalidate batch syntax before mutation, then evaluate well-formed tokens sequentially; stop after round completion. Provide useful errors, path-with-spaces support, replay and clean EOF.

## 4. Verification and benchmarking

Satisfy **SB-01 through SB-10** and the existing planned test matrix. Add unit, public-API integration, deterministic randomized/reference-model, full-corpus sampling, adversarial state/validation, rendering and actual binary subprocess tests throughout implementation. At least three recorded seeds must exercise thousands of session operations on small fixtures, with cached-score/subset/terminal-state invariants checked after each operation. Test dictionary mutation after setup does not change a round.

Implement and run the stable custom benchmark target:

```bash
cargo bench -p spelling-bee --bench algorithms -- --quick
cargo bench -p spelling-bee --bench algorithms -- --full
```

Follow `docs/BENCHMARKS.md`: identical puzzles/centers, complete-output equality, release builds, warm-up/repeated samples and output consumption, setup versus solve/lookup costs, measured memory versus estimates, retries/failures, raw samples and exact environment/hash metadata. No assumed speedup and no deletion of the baseline. Save concise reproducible artifacts under `docs/benchmarks/spelling-bee/<run-id>/`.

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

Execute all current README examples, doctests, prior trie CLI examples and new launcher/game transcripts. **Edit the existing** README, design, plan and TESTING in place; preserve historical checkpoint evidence and add actual results/commits without fabricated counts. Update the reuse-map ledger for UI/replay adaptations. Do not replace these documents with a new outline.

## 5. Mandatory independent review via subagent

After the author checks and required comparative run, commit the candidate and **spawn a fresh independent reviewer subagent**. Prefer minimal/fresh context containing repository path, base SHA, candidate SHA and authoritative file paths. Pause your edits during review. Do not ask it merely to summarize your logs.

Give the reviewer this task:

> Independently review this Spelling Bee branch against `Spelling Bee.pdf`, approved design/plan, SB-01…SB-10, TESTING, benchmark protocol and actual source. Read the base-to-candidate diff and the implemented tests. Run your own formatting, all-target check, strict Clippy, debug/release tests, release build, CLI/documentation examples, benchmark smoke and representative comparative reproduction. Check full-set algorithm independence, masks/center/repetition, owned snapshot, generation bounds/calibration, duplicate/scoring/Queen Bee/terminal invariants, hints, batch/EOF behavior, reuse provenance and benchmark fairness/failures. Report actual commands/outputs, requirement coverage and gaps, plus severity/file/line/reproduction/expected-versus-actual for every finding. Treat the author's completion summary as unverified. Remain read-only except generated build/test artifacts; do not merge.

Store its actual report at `docs/reviews/spelling-bee.md`, with base/reviewed-code SHA, reviewer identity and date, using the review template. Resolve all actionable findings with focused commits/tests; request independent re-review of fixes and subsequent behavior changes. Record non-actionable dispositions only with evidence and reviewer agreement. An unavailable subagent or blocked required check means **review blocked**: preserve the branch, report the blocker, and do not self-certify or merge.

## 6. Local merge, final verification and stop

Once review is clear and all checks/evidence/docs are accurate, follow plan section 10 to merge **`feat/spelling-bee` into local `main` with `--no-ff`** and a Conventional Commit message. If `main` advanced, merge it into the feature branch first, resolve and re-review affected changes. No destructive resets, rebases of shared history, remote pushes or branch deletion.

Rerun required workspace checks on merged `main`, plus benchmark/CLI/documentation smoke. Report actual base, feature, reviewed-code and merge SHAs; SB requirement coverage; tests and seeds; retained versions and measured comparison; independent findings/fixes; README examples verified; limitations. Leave `main` checked out with the game branch retained. **Stop. Do not implement Boggle, Wordle, alternative trie storage, timers or rich TUI/GUI.**
