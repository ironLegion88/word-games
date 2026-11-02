# Word Trie and Word Games in Rust

> **Repository Status — Assignments 1, 2, 3, and Wordle Extension Complete & Verified.**
>
> The `trie` core library crate, `trie-cli` interactive command-line binary crate, `spelling-bee` engine, `boggle` engine, `wordle` engine, and `word-games-cli` interactive terminal launcher are fully implemented and verified. All pass 100% of unit, invariant, differential, randomized reference-model, real-dictionary regression, and end-to-end integration test suites (207 total tests passing).

---

## 1. Project Overview

This project implements a reusable, high-performance, memory-resident **Word Trie** in safe Rust. It serves as the dictionary engine for word processing applications and word games including Spelling Bee, Boggle, and Wordle.

### Milestones & Scope

| Milestone | Scope | Status |
|---|---|---|
| **Assignment 1** | `trie` library + `trie-cli` interactive binary: arena allocation, branch pruning, free-list reuse, compaction, file loading, batch deletion, present/absent filtering, CLI REPL | **Completed & Verified** |
| **Assignment 2** | `spelling-bee` engine + `word-games-cli` driver: filtering (SB-S1) and trie (SB-S2) solvers, anchored generator, hints, presets | **Completed & Verified** |
| **Assignment 3** | `boggle` engine + `word-games-cli` driver: on-demand path DFS (BG-S1), dictionary scan (BG-S1E) and trie-guided (BG-S2) solvers; uniform (BG-G1), weighted (BG-G2), and synthetic dice (BG-G3) generators; extended scoring table (3..≥15); untimed gameplay | **Completed & Verified** |
| **Extension 1** | `wordle` engine + `word-games-cli` driver: marker baseline (WL-F1) and 26-entry frequency table (WL-F2) positional feedback; owned lexicon; monotonic keyboard; fail-safe `--answers` override | **Completed & Verified** |
| **Future Extensions** | Rich TUI/GUI, alternative trie storage, timed Boggle | *Separate future scope* |

---

## 2. Architecture & Rust Design Principles

The implementation demonstrates idiomatic Rust ownership, data structures, and memory safety:

1. **Indexed Arena Trie (`trie::node::Node` & `trie::node::NodeId`):**
   - Nodes are owned by a single contiguous vector (`Vec<Node>`) inside `Trie`.
   - Node references are strongly-typed 32-bit indices (`NodeId(u32)`), avoiding raw pointers, reference counting (`Rc`/`Arc`), interior mutability (`RefCell`), and `unsafe` code.
   - Child branches use fixed 26-slot direct indexing (`[u32; 26]`), providing true $O(L)$ child lookup per letter where $L$ is word length. Absent edges use `NO_CHILD = u32::MAX`.
   - Each `Node` occupies 112 bytes: `[u32; 26]` (104 bytes) + `bool is_word` (1 byte + 3 padding) + `u32 subtree_words` (4 bytes).

2. **True Physical Pruning & Safe Slot Recycling:**
   - Single-word `remove` and batch `remove_words` perform physical branch pruning. When a word is removed, all nodes that no longer lead to another valid word are detached from their parents, cleared, and pushed onto a LIFO `free_list`.
   - Reinsertions reuse slots from `free_list` before allocating new capacity in the vector.
   - Slot 0 always stores the root node and is never freed.

3. **Fragmentation-Triggered Automatic Compaction:**
   - When fragmentation reaches configured thresholds (e.g. $\ge 4096$ free slots and $\ge 1/3$ of the arena is free), `maybe_compact()` automatically compacts live reachable nodes into a dense vector, remaps all child IDs, empties the free list, and calls `shrink_to_fit()`.

4. **Validated ASCII Letters & Normalization (`trie::letter::Letter`):**
   - The `Letter(u8)` newtype guarantees a valid ASCII index in `0..=25`.
   - The public routine `normalize_word(&str)` accepts lowercase, uppercase, and mixed-case ASCII, normalizing to uppercase without allocating heap memory on already-uppercase input via `Cow<'_, str>`.
   - Malformed inputs (digits, punctuation, spaces, non-ASCII) are rejected with typed `WordError` variants without panicking or modifying dictionary state.

5. **Borrowing & Lifetimes:**
   - Queries (`contains`, `has_prefix`, `cursor`, `visit_words`) borrow `&self` immutably.
   - Mutations (`insert`, `remove`, `extend_from_reader`) take `&mut self`.
   - The read-only `TrieCursor<'a>` borrows `&'a Trie` and navigates edges without heap allocation or lifetime leakage.

---

## 3. Workspace Layout

```text
word-games/
├── Cargo.toml                         # Workspace root manifest
├── Cargo.lock                         # Pinned dependency lockfile
├── README.md                          # Current project documentation
├── .gitattributes                     # Binary protection for dictionary fixture
├── .gitignore                         # Build and target artifacts filter
├── assets/
│   └── dictionary.txt                 # Official 178,691-word uppercase fixture
├── assignments/
│   ├── Word Games.pdf                 # Active Assignment 1 specification
│   ├── Spelling Bee.pdf               # Prospective Assignment 2 specification
│   └── Boggle.pdf                     # Prospective Assignment 3 specification
├── docs/
│   ├── DESIGN_SPECIFICATION.md        # Binding architectural specification
│   ├── IMPLEMENTATION_PLAN.md         # Step-by-step plan & acceptance gates
│   └── TESTING.md                     # Comprehensive testing & verification log
├── boggle/                            # Independent Boggle engine crate
│   ├── Cargo.toml
│   ├── benches/
│   │   └── algorithms.rs              # 3 benchmark studies (solvers, queries, generators)
│   ├── src/
│   │   ├── lib.rs                     # Public re-exports and API
│   │   ├── board.rs                   # 4/5/6 board, 8-neighbor adjacency, Qu tile
│   │   ├── error.rs                   # Typed error types
│   │   ├── generation.rs              # Uniform, weighted, synthetic dice generators
│   │   ├── path.rs                    # Independent path DFS & on-demand validator
│   │   ├── scoring.rs                 # Checked arithmetic scoring (3..≥15)
│   │   ├── session.rs                 # Owned BoggleSession, guesses, cached score
│   │   └── solver.rs                  # Scan (BG-S1E) and Trie-pruned (BG-S2) solvers
│   └── tests/
│       ├── board.rs                   # Board dimension and adjacency tests
│       ├── generation.rs              # Uniform, weighted, dice reproducibility tests
│       ├── path.rs                    # Path DFS, Qu, diagonal, bit 35 tests
│       ├── scan_solver.rs             # Scan solver baseline tests
│       ├── scoring.rs                 # Extended score schedule & overflow tests
│       ├── session.rs                 # Session lifecycle & invariant tests
│       └── trie_solver.rs             # Trie solver & differential tests
├── spelling-bee/                      # Independent Spelling Bee engine crate
│   ├── Cargo.toml
│   ├── benches/
│   │   └── algorithms.rs              # SB-S1 vs SB-S2 benchmark harness
│   ├── src/
│   │   ├── lib.rs                     # Public re-exports
│   │   ├── error.rs                   # Typed errors
│   │   ├── generation.rs              # Pangram-anchored generator & presets
│   │   ├── hints.rs                   # Hint grid & two-letter count matrix
│   │   ├── puzzle.rs                  # 7 distinct letters & center validation
│   │   ├── session.rs                 # Owned SpellingBeeSession & cached score
│   │   └── solver.rs                  # Filtering (SB-S1) and trie-pruned (SB-S2) solvers
│   └── tests/
│       ├── calibration.rs             # Calibrated difficulty preset tests
│       ├── differential.rs            # SB-S1 vs SB-S2 differential tests
│       ├── filter_solver.rs           # Filter solver baseline tests
│       ├── generation.rs              # Procedural anchored generator tests
│       ├── puzzle.rs                  # Puzzle validation tests
│       ├── scoring.rs                 # Scoring & pangram bonus tests
│       └── session.rs                 # Session state & Queen Bee tests
├── trie/                              # Reusable library crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                     # Main Trie struct, public API & invariants
│   │   ├── error.rs                   # Typed WordError and LoadError
│   │   ├── letter.rs                  # Letter(u8) newtype & normalization
│   │   ├── node.rs                    # Arena Node, NodeId, and sentinels
│   │   ├── traversal.rs               # Read-only TrieCursor
│   │   └── loading.rs                 # Buffered parsing, batch ops & filtering
│   └── tests/
│       ├── arena.rs                   # Arena layout & invariant tests
│       ├── deletion.rs                # Branch pruning, free-list reuse, compaction
│       ├── letter.rs                  # Letter validation & normalization tests
│       ├── loading.rs                 # Delimiter parsing, CRLF, BOM, atomicity
│       ├── operations.rs              # Insert, lookup, prefix, cursor tests
│       ├── randomized.rs              # Seeded SplitMix64 reference-model tests
│       └── regression.rs              # Real 178,691-word corpus regression
├── trie-cli/                          # Interactive command-line executable
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                    # CLI entry point & argument handling
│   │   ├── commands.rs                # Command enum & line parser
│   │   └── repl.rs                    # Interactive terminal loop
│   └── tests/
│       └── cli.rs                     # Subprocess integration tests with piped stdin
└── word-games-cli/                    # Shared interactive terminal launcher & driver
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs                     # Module exports
    │   ├── main.rs                    # Central binary launcher
    │   ├── boggle.rs                  # Boggle terminal driver
    │   ├── replay.rs                  # Replay menu prompt (adapted from Mastermind)
    │   ├── spelling_bee.rs            # Spelling Bee terminal driver
    │   └── terminal.rs                # ANSI & plain color formatting helpers
    └── tests/
        ├── boggle_cli.rs              # Subprocess & interactive Boggle CLI tests
        └── spelling_bee_cli.rs        # Subprocess & interactive Spelling Bee CLI tests
```

---

## 4. Building and Running

### Prerequisites

- Rust toolchain (edition 2021; verified on `rustc 1.98.1`).
- `cargo` package manager.

### Standard Build & Verification Commands

```bash
# Check formatting
cargo fmt --all -- --check

# Compile-check all workspace members
cargo check --workspace

# Lint with strict Clippy warnings enabled
cargo clippy --workspace --all-targets -- -D warnings

# Run all unit, invariant, randomized, and integration tests
cargo test --workspace

# Run all tests with release optimizations
cargo test --workspace --release

# Build optimized release binaries
cargo build --workspace --release
```

---

## 5. Using `trie-cli`

### Launching the CLI

You can start `trie-cli` with an empty dictionary:
```bash
cargo run -p trie-cli
```

Or preload a dictionary fixture on startup:
```bash
cargo run -p trie-cli -- --dictionary assets/dictionary.txt
```

### Supported Interactive Commands

| Command | Syntax | Description |
|---|---|---|
| `help` | `help` | Display available commands |
| `new` | `new` | Reset to a fresh empty dictionary |
| `load` | `load <path>` | Replace dictionary with words parsed from `<path>` |
| `add-file` | `add-file <path>` | Append and validate words from `<path>` into dictionary |
| `insert` | `insert <word>` | Insert a single word (case-insensitive) |
| `contains` | `contains <word>` | Check word membership; returns `true` or `false` |
| `remove` | `remove <word>` | Delete a single word with branch pruning |
| `remove-file`| `remove-file <path>`| Delete all words listed in file with pruning |
| `present` | `present <path>` | Output tokens from `<path>` present in dictionary |
| `absent` | `absent <path>` | Output tokens from `<path>` absent from dictionary |
| `normalize`| `normalize <word>` | Display uppercase conversion of `<word>` |
| `stats` | `stats` | Display word count, live nodes, arena & free slots |
| `quit` | `quit` or `exit` | Terminate session cleanly |

### Verified Session Example

```text
Word Trie Interactive CLI
Type 'help' for available commands, or 'quit' to exit.

trie> insert CAR
Inserted "CAR" (total words: 1).
trie> insert CART
Inserted "CART" (total words: 2).
trie> contains car
true: "CAR" is in the dictionary.
trie> contains dog
false: "DOG" is NOT in the dictionary.
trie> normalize bIrD
Normalized: BIRD
trie> stats
Dictionary Statistics:
  Words:            2
  Live nodes:       5
  Arena slots:      5
  Free slots:       0
  Allocated memory: 560 bytes (nominal arena nodes)
trie> remove CART
Removed "CART" (remaining words: 1).
trie> contains cart
false: "CART" is NOT in the dictionary.
trie> contains car
true: "CAR" is in the dictionary.
trie> stats
Dictionary Statistics:
  Words:            1
  Live nodes:       4
  Arena slots:      5
  Free slots:       1
  Allocated memory: 560 bytes (nominal arena nodes)
trie> quit
Goodbye!
```

---

## 6. Spelling Bee Interactive Game (`word-games-cli`)

The `word-games-cli` crate provides an interactive terminal driver for Spelling Bee, supporting seeded procedural puzzle generation, custom fixed letter sets, difficulty presets, hint grids, prefix lists, and 3-column alphabetical displays.

### CLI Usage

```bash
# Launch interactive game with procedurally generated anchored puzzle
cargo run -p word-games-cli -- spelling-bee

# Launch with calibrated difficulty preset and deterministic seed
cargo run -p word-games-cli -- spelling-bee --difficulty easy --seed 42

# Launch with fixed letters and designated central letter
cargo run -p word-games-cli -- spelling-bee --letters CAIMORT --center C

# Select solver strategy (trie-pruned default, or filtering baseline)
cargo run -p word-games-cli -- spelling-bee --solver filter
```

### In-Game Commands

- Guess words directly or enter comma/space-delimited batches (e.g. `ACACIA, AROMATIC, COMIC`).
- `/help`: Display game rules and scoring details.
- `/hints`: Display initial-letter × word-length hint grid and two-letter prefix counts.
- `/guesses`: Display all currently discovered words sorted alphabetically in three columns.
- `/quit`: Forfeit the round, display final score, and reveal all missed words.

---

## 7. Boggle Interactive Game (`word-games-cli -- boggle`)

The `word-games-cli` crate provides an interactive terminal driver for Boggle, supporting 4x4, 5x5, and 6x6 board grids, three selectable board generation strategies (uniform, frequency-weighted, and synthetic 6-face dice), on-demand and trie-guided solvers, extended checked scoring, and clean early forfeits.

### CLI Usage

```bash
# Launch interactive 4x4 game with default uniform board generation
cargo run -p word-games-cli -- boggle

# Launch 5x5 board with frequency-weighted letter generation
cargo run -p word-games-cli -- boggle --size 5 --generator weighted

# Launch 6x6 board with synthetic 6-face dice generation and fixed seed
cargo run -p word-games-cli -- boggle --size 6 --generator dice --seed 42

# Launch with fixed board fixture
cargo run -p word-games-cli -- boggle --board "CATSDOGSQREWBIRD"

# Select solver strategy (trie default, or scan baseline)
cargo run -p word-games-cli -- boggle --solver scan
```

### In-Game Commands

- Guess words directly or enter comma/space-delimited batches (e.g. `CATS, DOGS, BIRD`).
- `/board`: Redisplay the current board grid.
- `/guesses`: Display all currently discovered words on this board.
- `/help`: Display game rules and scoring details.
- `/quit`: Forfeit the round, display final score, and reveal all missed words.

### Benchmarks

Run the reproducible benchmark suite comparing scan vs trie solvers, query amortization traces, and generator quality distributions across sizes 4, 5, and 6:

```bash
# Quick smoke mode (3 boards per size)
cargo bench -p boggle --bench algorithms -- --quick --suite all

# Full comparative verification (100 boards per generator/size combo = 900 boards)
cargo bench -p boggle --bench algorithms -- --full --suite all
```

Evidence reports are recorded under `docs/benchmarks/boggle/run-20261102/`.

### 7.3 Wordle CLI

Launch an interactive Wordle session:

```bash
# Launch interactive game with default dictionary and curated answer subset
cargo run -p word-games-cli -- wordle

# Launch with deterministic random seed and marker-based feedback (WL-F1 baseline)
cargo run -p word-games-cli -- wordle --seed 42 --feedback marker

# Launch with 26-entry frequency feedback (WL-F2 production default)
cargo run -p word-games-cli -- wordle --seed 42 --feedback frequency

# Launch with custom validated answer override file
cargo run -p word-games-cli -- wordle --answers assets/wordle_curated_answers.txt

# Launch in plain text mode (no ANSI escape codes)
cargo run -p word-games-cli -- wordle --color never
```

#### In-Game Commands
- Enter single 5-letter word guesses.
- Malformed words or non-dictionary entries report friendly errors without consuming attempts.
- Repeated valid guesses consume an attempt and reproduce feedback.
- `/keyboard`: Redisplay the monotonic 3-row QWERTY keyboard status.
- `/history`: Redisplay past evaluated guess rows.
- `/help`: Display game rules and scoring details.
- `/quit`: Forfeit the round, display attempts used, and reveal the secret word.

#### Benchmarks

Run the reproducible benchmark suite comparing WL-F1 (marker matching) and WL-F2 (26-entry frequency counter) across 7 strata:

```bash
# Quick smoke mode
cargo bench -p wordle --bench algorithms -- --quick

# Full comparative verification (30 batches x 1000 pairs = 30,000 evaluations per stratum)
cargo bench -p wordle --bench algorithms -- --full
```

Evidence reports are recorded under `docs/benchmarks/wordle/run-20261102/`.

---

## 8. Real Dictionary Facts (Verified)

Testing against the provided `assets/dictionary.txt` verifies:
- **Unique words:** exactly 178,691
- **Standard uncompressed trie reachable nodes (incl. root):** exactly 395,185
- **Word lengths:** minimum 2, maximum 15, mean 8.8671
- **Trailing newline:** file does not end with `\n` (last line is `ZZZ`); handled cleanly without word truncation.
- **Load performance:** loads full dictionary in ~340ms (debug) / ~190ms (release).

---

## 9. Next Steps and Implementation Prompts

The current executable features include `trie`, `trie-cli`, `spelling-bee`, `boggle`, `wordle`, and `word-games-cli`. Prompts are sequenced in the sibling-repository layout `Assignment 3/word-games` and `Assignment 3/mastermind`:

| Order | Prompt | Branch in `word-games` | Prerequisite |
|---|---|---|---|
| 1 | [Spelling Bee](prompts/SPELLING_BEE_AGENT_PROMPT.md) | `feat/spelling-bee` | **Completed & Merged** into `main` |
| 2 | [Boggle](prompts/BOGGLE_AGENT_PROMPT.md) | `feat/boggle` | **Completed & Merged** into `main` |
| 3 | [Wordle](prompts/WORDLE_AGENT_PROMPT.md) | `feat/wordle` | **Completed & Merged** into `main` |

Each prompt implements a working baseline, adds and retains its alternative algorithms, runs correctness comparisons and reproducible benchmarks, and then requires a fresh subagent to review the code independently. Findings must be resolved before the game's branch merges locally into `main`. The agent then stops. Remote pushes are not part of these prompts.

The terminal entry point is `word-games-cli`, providing an interactive launcher and the implemented `spelling-bee`, `boggle`, and `wordle` game subcommands. Mastermind remains independently buildable; selected code is adapted into this repository without a sibling-path runtime dependency.

### Additional planning references

- [Design specification](docs/DESIGN_SPECIFICATION.md): existing design plus the approved game contracts and staged algorithms.
- [Implementation plan](docs/IMPLEMENTATION_PLAN.md): existing milestones plus Wordle, branch/merge gates, and independent review.
- [Testing record](docs/TESTING.md): preserved Assignment 1 evidence, with game cases and verified test results.
- [Benchmark protocol](docs/BENCHMARKS.md): version matrix, fair workloads, raw results, and interpretation requirements.
- [Mastermind reuse map](docs/MASTERMIND_REUSE_MAP.md): source symbols, planned destinations, adaptation requirements, and verification evidence.
- [Prompt router](AGENT_PROMPT.md): retains the original Assignment 1 prompt as an archive and points to the three new prompts.

Spelling Bee, Boggle, and Wordle implementations, benchmarks, tests, and reviews are completed in this repository.
