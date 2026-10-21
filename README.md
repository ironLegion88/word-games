# Word Trie and Word Games in Rust

> **Repository Status — Assignment 1 Complete & Verified.**
>
> The `trie` core library crate and `trie-cli` interactive command-line binary crate are fully implemented, passing 100% of unit, invariant, randomized reference-model, real-dictionary regression, and end-to-end integration test suites.
>
> *Future Milestones (Spelling Bee, Boggle) remain in the planned roadmap and are not yet authorized or implemented.*

---

## 1. Project Overview

This project implements a reusable, high-performance, memory-resident **Word Trie** in safe Rust. It serves as the dictionary engine for word processing applications and upcoming word games.

### Milestones & Scope

| Milestone | Scope | Status |
|---|---|---|
| **Assignment 1** | `trie` library + `trie-cli` interactive binary: arena allocation, branch pruning, free-list reuse, compaction, file loading, batch deletion, present/absent filtering, CLI REPL | **Completed & Verified** |
| **Assignment 2** | Spelling Bee puzzle engine + terminal driver | *Planned; awaiting explicit authorization* |
| **Assignment 3** | Boggle grid solver + terminal driver | *Planned; awaiting explicit authorization* |
| **Future Extensions** | Wordle, rich TUI/GUI, adaptive node structures | *Future roadmap; out of scope* |

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
word-trie/
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
└── trie-cli/                          # Interactive command-line executable
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs                    # CLI entry point & argument handling
    │   ├── commands.rs                # Command enum & line parser
    │   └── repl.rs                    # Interactive terminal loop
    └── tests/
        └── cli.rs                     # Subprocess integration tests with piped stdin
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

## 6. Real Dictionary Facts (Verified)

Testing against the provided `assets/dictionary.txt` verifies:
- **Unique words:** exactly 178,691
- **Standard uncompressed trie reachable nodes (incl. root):** exactly 395,185
- **Word lengths:** minimum 2, maximum 15, mean 8.8671
- **Trailing newline:** file does not end with `\n` (last line is `ZZZ`); handled cleanly without word truncation.
- **Load performance:** loads full dictionary in ~340ms (debug) / ~190ms (release).

---

## 7. Next Steps (Awaiting Authorization)

Per project guidelines, work on subsequent game implementations is paused.
- **Stage B (Spelling Bee):** will begin only upon explicit user instruction.
- **Stage C (Boggle):** will begin only upon subsequent explicit instruction.
