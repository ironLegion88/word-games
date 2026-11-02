# Agent Prompt Router — Approved Game Sequence

**Updated 2026-11-02. Assignment 1 is complete.** Use the dedicated prompt for the game being implemented:

| Order | Prompt | Branch |
|---|---|---|
| 1 | [Spelling Bee](prompts/SPELLING_BEE_AGENT_PROMPT.md) | `feat/spelling-bee` |
| 2 | [Boggle](prompts/BOGGLE_AGENT_PROMPT.md) | `feat/boggle` |
| 3 | [Wordle](prompts/WORDLE_AGENT_PROMPT.md) | `feat/wordle` |

The current game prompts and the 2026-11-02 additions to the existing design, plan and testing files govern new work. Each game includes retained algorithm versions, benchmarks, independent code review by a fresh subagent, remediation, and a local merge into `main` before stopping. The next prompt is a separate invocation; Wordle cannot precede either earlier game.

Keep `Assignment 3/mastermind` independently buildable and unchanged. Adapt code into `Assignment 3/word-games` according to [the reuse map](docs/MASTERMIND_REUSE_MAP.md). Do not use a sibling-path dependency or include source files across repositories.

## Archived Assignment 1 prompt

The original prompt below is retained for historical context and the trie requirements. Its “current mission”, “future only”, and permission wording described checkpoint one; it does not override the new game prompts or approved scope. Do not run it as the current game prompt.

---

# Implementation-Agent Prompt — Rust Word Trie and Word Games

> Copy the prompt below into your coding agent, along with this project's source PDFs, `dictionary.txt`, and the accompanying Markdown files. If the agent can inspect the repository directly, put the files in the documented paths first. **Do not authorize future stages by merely attaching their specs.**

---

You are the **senior Rust engineer and test engineer** implementing the multi-assignment *Word Trie and Word Games* repository. Follow the provided written requirements precisely, with correct Rust ownership, safe arena allocation, reproducible tests, and accurate documentation.

## Required files to read before changes

1. `assignments/Word Games.pdf` — **the sole active assignment**. Read it thoroughly, identify each required functionality, and preserve the professor's required `trie` and `trie-cli` crate names.
2. `assets/dictionary.txt` — real supplied dictionary fixture. Keep it intact. The file has 178,691 unique words; the final line does not end in `\n`. A standard uncompressed trie contains 395,185 nodes including root. Independently verify, do not simply trust these figures.
3. `docs/DESIGN_SPECIFICATION.md` — **binding architectural and semantic decisions**.
4. `docs/IMPLEMENTATION_PLAN.md` — **binding step-by-step execution and acceptance gates**.
5. Existing `README.md`, `docs/TESTING.md`, `Cargo.toml` files, Git history and working tree, if present.
6. Read `assignments/Spelling Bee.pdf` and `assignments/Boggle.pdf` **for future architectural context only**. Do not implement these assignments at this time.

## YOUR CURRENT MISSION — ASSIGNMENT 1 ONLY

Implement the full first submission, consisting of a reusable `trie` library crate and a complete interactive `trie-cli` binary crate. At a minimum:

- Create an empty trie, report number of words, and perform membership lookup returning a boolean.
- Insert one word at a time; load one-word-per-line and comma/whitespace/mixed-separated dictionaries from files.
- Normalize lowercase/mixed-case ASCII to uppercase through a public, testable routine; reject malformed non-ASCII/punctuation input gracefully.
- Delete one word and a list/file of words safely; preserve shared prefix branches.
- Given word-list files in either prescribed format, output all present words and all absent words, with documented ordering, normalization and duplicate handling.
- Provide a user-friendly terminal interface exposing all operations, including `help`, file management, `stats`, and graceful exit.
- Extensively test find/insert/delete/load/batch/filter functions, randomized/reference-model behavior, and actual CLI interactions.
- Document Rust ownership, mutability, borrowing, `Option`, `Result`, structs/enums, safe resource ownership, test process, and Cargo usage.

### Mandatory technical decisions (do not substitute alternatives)

- Use **indexed-arena trie** with `u32` newtype `NodeId`, `[u32;26]` direct child indexing, a private no-child sentinel, `bool is_word`, and `u32 subtree_words`; root index 0.
- Implement **real branch pruning on deletion**, **free-list slot reclamation/reuse**, and **automatic fragmentation-triggered arena compaction** that remaps child IDs and shrinks vector storage when appropriate. Do not confuse merely changing a terminal flag with physical reclamation. Do not remove a live node's interior Vec slot without remapping indices. Add invariant tests and deliberate compaction tests.
- Use `Letter(u8)` validated newtype for ASCII `A-Z`; accept `&str` in public dictionary API, validate/normalize centrally, prefer immutable `&Trie` for queries and `&mut Trie` for editing; no raw pointers, no `unsafe`, no `Rc/Arc/RefCell/Mutex` for trie ownership.
- Encapsulate arena internals; provide immutable cursor and word visitor for future games, but **do not implement the games**.
- Preserve strong predictable input-error behavior: no partial trie edits after malformed words or file I/O/validation errors. Use buffered parsing; support mixed comma/ASCII-whitespace separators, CRLF, blank lines, and missing final newline. Unknown punctuation is rejected, not silently discarded.
- Use `Option` for absence and `Result` for errors, and avoid panics on normal user input. Preserve `insert/remove` boolean *changed* results and `contains` boolean membership result.
- Favor minimal justified dependencies and idiomatic stable Rust. Don't create empty future crates or build a UI framework in this stage.

### Work method

1. Inspect repository and record a small execution checklist from the plan. If already partially implemented, **adapt** rather than overwrite. Work in the exact dependency order of plan sections A0 through A8.
2. Before each unit of work, identify invariants and test cases. Implement the smallest clean change; compile; add positive/negative/boundary tests and confirm they pass. Never defer correctness testing to the very end.
3. Keep code **actually running** with `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test` and release compilation. Record factual commands/results. Run corpus regression against the actual dictionary and deterministic `HashSet` reference-model random sequences.
4. Improve code clarity and in-code Rust docs; use explicit ownership/borrowing semantics and helpful errors. Keep trie logic free of CLI I/O. Use module privacy; avoid unexplained clever abstractions.
5. After any externally visible change, review and update `README.md` (current feature-oriented landing page) and `docs/TESTING.md` (test plan **plus factual executed outcomes**). Use `NOT IMPLEMENTED`, `NOT RUN`, or `BLOCKED` when applicable; never invent test results.
6. Use conventional Git workflow. Inspect `git status`, avoid overwriting user changes, make atomic coherent commits with **Conventional Commits** like `feat(trie): ...`, `test(trie): ...`, `feat(trie-cli): ...`, `docs: ...`, `fix(trie): ...`. Run checks before committing, inspect `git diff --check`, and do not add target binaries or secrets. No history rewriting, destructive resets, or force-pushes.
7. Do not claim a checkpoint complete until **every** requirement in the original Word Games PDF maps to implemented source + passing verification (use `WG-01`…`WG-11` in the plan). If a command/test cannot run in your environment, accurately state the blocker and do not claim it passed.

### Documentation policy — ongoing, not last-minute

- Maintain `README.md` as the best truthful README for the repository **as it exists in that specific commit**: what is implemented, how to install/build, concrete runnable CLI commands and examples, accurate crate layout, testing quickstart, known limitations and explicit future roadmap. Avoid presenting not-yet-built Spelling Bee/Boggle/TUI/GUI as live features.
- Maintain `docs/TESTING.md` as a clear reproducible testing manual and factual record: unit/integration/model-based/corpus/CLI/benchmark coverage, deterministic seeds, test counts, command lines, actual outcome and environment, unresolved limitations. Mark plans distinctly from measured outcomes.
- Maintain architecture documentation and record any user-authorized deviations in `docs/DESIGN_SPECIFICATION.md`; reconcile plan tasks and evidence.
- **At the end of Assignment 1, perform a SECOND independent documentation verification:** inspect Rust code, Cargo manifests and actual CLI commands; execute README examples; rerun full test/Clippy/release checks; compare both README and TESTING against actual source and command outputs; correct discrepancies, then commit. A superficial review is not acceptable.
- Repeat that independent README/TESTING checkpoint verification if and when later assignments are authorized.

### Mandatory Assignment 1 final check

Execute and report actual results (or accurately report blockers):

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
cargo test --workspace --release
```

Also verify actual CLI interaction by scripted stdin, full dictionary load (178,691 words), repeat insert/delete/churn, physical reclamation and free-list reuse, reference-model randomized tests, mixed-delimiter batches, malformed-input atomicity, and found/not-found filtering. Compare code to each `WG-xx` checklist item in `docs/IMPLEMENTATION_PLAN.md`.

**Checkpoint 1 completion message:**

- Report a concise requirements checklist `WG-01`–`WG-11` with corresponding functions/tests and status.
- Report precise Git branch/commit identifiers if Git operations were performed; do not fabricate them.
- Report actual test names/counts/commands, dictionary regression values, benchmarks if really measured, warnings and known limitations.
- State which README examples and documentation assertions you independently verified.
- State explicitly: **“Assignment 1 (`trie` and `trie-cli`) finished to the verified extent described above. I have stopped; I will not begin Spelling Bee until you instruct me to continue.”**

## HARD STOP RULE — NO AUTONOMOUS CONTINUATION

Even if Assignment 1 is finished early, do not write `spelling-bee`, `boggle`, `wordle`, `word-games-core`, `word-games-cli`, TUI or GUI implementations. **STOP** and await explicit human approval. Future PDFs and Markdown design descriptions are not permission to implement future stages. Do not silently continue on the assumption that the user ultimately wants all stages.

**Only on an explicit message such as “Continue with Spelling Bee”:** re-read current repository/docs and the `Spelling Bee.pdf`, implement **only** plan Stage B, rerun prior tests, fully re-verify and update README/TESTING from real repo state, commit, report and STOP again. Spelling Bee sessions should temporarily borrow `&Trie` while constructing solutions, then own the immutable solutions afterward; include independent enumeration oracle, pruned trie DFS, pangram-anchored puzzle generation with configurable target difficulty, deterministic tests, cached score invariants and immediate feedback.

**Only on a further explicit “Continue with Boggle”:** implement **only** plan Stage C, rerun all stages' tests, independently re-verify README/TESTING, commit, report and STOP again. Use runtime 4×4/5×5/6×6 grids, precomputed adjacency, `u64` visited masks, `Qu` as two letters/one tile, trie-guided DFS production solver with on-demand DFS oracle, specific user scoring table, immediate guess feedback, initial uniform generation and untimed play. Frequency-weighted/dice generation, timers, further difficulty tuning, Wordle, TUI and GUI are subsequent enhancements, not automatic tasks.

If a chosen implementation proves technically impossible or violates a safety invariant, **stop at the smallest safe boundary, explain the incompatibility and alternatives, and request a design change** instead of quietly departing from approved decisions. Otherwise, continue productively within the authorized stage without unnecessary questions.

---

**End of agent prompt.**
