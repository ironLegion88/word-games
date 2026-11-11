# CLI guide

Run these commands from the repository root with a stable Rust toolchain. The supplied dictionary and Wordle answer list are in `assets/`.

## Using `trie-cli`

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

## Spelling Bee Interactive Game (`word-games-cli`)

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

## Boggle Interactive Game (`word-games-cli -- boggle`)

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

## Real Dictionary Facts (Verified)

Testing against the provided `assets/dictionary.txt` verifies:
- **Unique words:** exactly 178,691
- **Standard uncompressed trie reachable nodes (incl. root):** exactly 395,185
- **Word lengths:** minimum 2, maximum 15, mean 8.8671
- **Trailing newline:** file does not end with `\n` (last line is `ZZZ`); handled cleanly without word truncation.
- **Load performance:** loads full dictionary in ~340ms (debug) / ~190ms (release).

---


## Wordle

```sh
cargo run --release --locked -p word-games-cli -- wordle --seed 42
cargo run --release --locked -p word-games-cli -- wordle --feedback marker --color never
```

The default scorer uses a 26-entry frequency array. `--feedback marker` selects the independent marker scorer. Both evaluate exact matches before allocating remaining letters to partial matches.

Use `/keyboard`, `/history`, `/help`, or `/quit` during a round. Six valid guesses are allowed. Invalid input leaves the attempt count unchanged.

To supply a different answer subset, use `--answers <path>`. Each entry must be a five-letter ASCII word present in the dictionary. The loader accepts small custom lists and validates the complete list before starting a round. See [answer curation](WORDLE_CURATION_PROVENANCE.md).

For the complete option list, run `cargo run --locked -p word-games-cli -- --help`.
