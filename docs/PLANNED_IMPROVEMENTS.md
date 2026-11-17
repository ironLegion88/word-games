# Planned Improvements and Roadmap

**Status:** Approved Roadmap & Technical Backlog
**Target Repository:** [`ironLegion88/word-games`](https://github.com/ironLegion88/word-games)
**Scope Firewall:** This document records future enhancements only. **None** of the features below are implemented in the current milestone, ensuring the stability and boundaries of the existing crates and verified test suites.

---

## Roadmap Overview and Classification

Improvements are classified by priority and approval status:

- **Approved Plans (Near/Medium-Term)**: Concrete architectural steps and assignment extensions agreed upon in project specifications and pending execution in upcoming milestones.
- **Proposed Improvements (Long-Term)**: Exploratory features, UX enhancements, and ergonomic upgrades under evaluation for future releases.

| Category | Item | Classification | Target Milestone |
|---|---|---|---|
| **Dev & CI** | Synchronized Rust Toolchain Upgrade (1.98.1 → Latest Stable) | Approved Plan | Next Maintenance Pass |
| **Dev & CI** | Resolution of Newer Clippy Diagnostics | Approved Plan | Next Maintenance Pass |
| **Dev & CI** | CI Matrix Expansion (macOS) & Automated Releases | Proposed Improvement | Future Infrastructure |
| **Timer** | Word Game Integration (Boggle, Spelling Bee, Wordle) | Approved Plan | Feature Milestone 2 |
| **Timer** | Concurrent Independent Countdowns | Approved Plan | Feature Milestone 2 |
| **Timer** | Sequential Programmed Timers (Exercise/Rest Routines) | Approved Plan | Feature Milestone 2 |
| **Timer** | Optional 100 ms Sub-second Display Mode | Proposed Improvement | Feature Milestone 3 |
| **Timer** | Audio Alerts, Sound FX, Duration Presets & Accessibility | Proposed Improvement | Feature Milestone 3 |
| **Timer** | Future Desktop GUI Integration (egui / Iced / Slint) | Proposed Improvement | Future Platform |
| **Word Games** | Unified Launcher & "Ghostlight" Terminal Design Language | Proposed Improvement | UX Overhaul |
| **Word Games** | Consistent Auditory Feedback & Keyboard Accessibility | Proposed Improvement | UX Overhaul |
| **Word Games** | Alternative Trie Representations & Comparative Benchmarks | Approved Plan | Algorithm Research |
| **Word Games** | Spelling Bee Pruned-Trie vs. Baseline Differential Suite | Approved Plan | Algorithm Research |
| **Word Games** | Boggle & Wordle Enhancements (Hard Mode, Grid Expansions) | Proposed Improvement | Game Feature Pass |
| **Word Games** | Cross-Platform Distribution (Homebrew, Crates.io, Prebuilts) | Proposed Improvement | Release Engineering |

---

## 1. Development and Continuous Integration

### 1.1 Synchronized Rust Toolchain Upgrade (Approved Plan — High Priority)
- **Background**: GitHub Actions workflows currently pin Rust, Cargo, Clippy, and Rustfmt to `1.98.1` (`.github/workflows/ci.yml`). This pin was introduced because unpinned `stable` runners in GitHub Actions automatically resolved to Rust 1.100.0, which emits newer Clippy lints that fail strict `-D warnings` checks while local engineering environments use Rust 1.98.1.
- **Action Plan**:
  1. Coordinate a synchronized upgrade of local development machines, developer container environments, and GitHub Actions runners to the latest stable toolchain (Rust 1.100+).
  2. Unpin the fixed `toolchain: 1.98.1` in CI back to `stable` once local environments match.
  3. Ensure all workspace members compile with zero warnings under the new compiler version.

### 1.2 Resolution of Newer Clippy Diagnostics (Approved Plan — High Priority)
- **Background**: Newer Rust releases introduce more refined static analysis rules (such as stricter question-mark usage patterns, collapsible matches, and refined slice indexing diagnostics).
- **Action Plan**:
  1. Audit each crate across the workspace under the new compiler's Clippy suite.
  2. Implement behavior-preserving refactors that satisfy the updated lint rules without altering algorithmic complexity, memory layout, or public API contracts.
  3. Retain the strict `cargo clippy --workspace --all-targets --locked -- -D warnings` gate in CI.

### 1.3 Version Compatibility, Coverage, and Release Procedures (Proposed Improvement — Medium Priority)
- **MSRV Policy**: Formalize and enforce a Minimum Supported Rust Version (MSRV) across the workspace using `cargo-msrv`.
- **Platform Matrix Expansion**: Add a macOS runner (`macos-latest`) to the existing Ubuntu and Windows CI matrix to verify Unix terminal handling and BSD-derived clock behavior.
- **Release Automation**: Introduce `cargo-dist` or GitHub Actions release workflows to build, sign, and publish pre-compiled release binaries for Windows, Linux, and macOS upon tagging release commits.

---

## 2. Countdown Timer Extensions

### 2.1 Word Game Integration (Approved Plan — High Priority)
- **Boggle Blitz Mode**: Introduce a standard 3-minute countdown round in Boggle. The game loop will poll an integrated `CountdownTimer` instance between word submissions, freezing input and tabulating final scores when the timer expires.
- **Spelling Bee Timed Challenges**: Provide optional timed puzzle solving (e.g. 5 minutes to find as many pangrams and four-letter words as possible).
- **Wordle Speedrun Mode**: Add an optional turn timer (e.g. 30 seconds per guess) or round timer with visual urgency indicators.
- **Architectural Boundary**: Maintain strict separation of concerns. Game crates (`boggle`, `spelling-bee`, `wordle`) will remain independent of terminal UI or timer crates; integration will occur exclusively within `word-games-cli` through component composition.

### 2.2 Concurrent Independent Countdowns (Approved Plan — High Priority)
- **Capability**: Extend the timer architecture to manage multiple independent countdown timers running concurrently.
- **Design Strategy**:
  - Implement a `TimerPool` or `TimerScheduler` managing a map of active timers indexed by unique IDs (`TimerId`).
  - Single-threaded monotonic event multiplexing: the scheduler computes the next nearest expiration deadline and polls all running timers during each event loop pass.
  - Multi-threaded timer worker support (as described in the Rust Book concurrency guidelines) for asynchronous background dispatch where needed.

### 2.3 Sequential Programmed Timers and Exercise Routines (Approved Plan — High Priority)
- **Capability**: Implement sequential countdown routines configured in tabular formats, directly realizing the exercise routine feature outlined in the assignment specification:
  - Table of exercises, active durations, and inter-exercise rest intervals (e.g. Jumping Jacks 30s + 5s rest, Wall Sit 30s + 5s rest, Push-ups 45s + 15s rest, Crunches 45s + 15s rest, Plank 1m + 10s rest).
- **Design Strategy**:
  - Model routines as a finite state machine: `RoutineState { InExercise(index), InRest(index), Completed }`.
  - Automatic progression from exercise to rest to next exercise upon expiration.
  - Emitting distinct announcement events and screen messages at step transitions (stopping, resting, starting next exercise).

### 2.4 Sub-second Precision (Optional 100 ms Display Mode) (Proposed Improvement — Medium Priority)
- **Capability**: Provide an optional sub-second display mode showing deciseconds (`SS.s`), as permitted in the assignment specification.
- **Design Strategy**:
  - Keep internal timekeeping on `Duration` (nanosecond precision).
  - Add formatting helpers for tenths of a second (`MM:SS.d`).
  - Adjust Crossterm polling interval from 50 ms to 25–30 ms to ensure smooth 100 ms visual frame rendering without aliasing or tearing.

### 2.5 Audio Alerts, Sound Effects, Presets, and Accessibility (Proposed Improvement — Medium Priority)
- **Audio Feedback**: Support terminal bell (`\x07`) and optional audio library integration (e.g. `rodio`) to play distinct tones on timer start, pause, resume, and zero-reach completion.
- **Quick Presets**: Provide shortcut keys for common durations (e.g. `1` for 1 min, `3` for 3 min, `5` for 5 min, `P` for Pomodoro 25 min).
- **Accessibility**: Provide high-contrast color schemes, reduced-motion text modes, and screen-reader-friendly semantic status lines.

### 2.6 Graphical User Interface (GUI) and Portable Packaging (Proposed Improvement — Long-Term)
- **Decoupled Architecture**: Keep `countdown-timer` strictly `std`-only so it can serve as the engine for GUI frontends.
- **GUI Implementations**: Explore lightweight desktop UI frontends using `egui`, `Iced`, or `Slint`.

---

## 3. Word Games and Trie Enhancements

### 3.1 Unified Interface, Launcher, and "Ghostlight" Design Language (Proposed Improvement — Medium Priority)
- **"Ghostlight" Terminal Design System**: Establish a shared design system for all terminal applications in the workspace:
  - Consistent box-drawing borders (rounded Unicode corners vs. ASCII fallback).
  - Standard color palette tokens (Primary, Secondary, Success, Warning, Muted).
  - Shared status line and navigation controls across Wordle, Boggle, Spelling Bee, and Timer.
- **Unified Launcher**: Enhance `word-games-cli` with an interactive menu launcher allowing players to switch between games and the countdown timer seamlessly.

### 3.2 Consistent Sound Design and Keyboard Ergonomics (Proposed Improvement — Medium Priority)
- **Sound Effects**: Optional non-intrusive auditory cues for valid word submissions, invalid inputs, pangram discovery in Spelling Bee, and victory/defeat in Wordle.
- **Keybindings**: Standardized `/help`, `/quit`, `/restart`, `/settings` slash commands and Vim-friendly (`h/j/k/l`) cursor navigation.

### 3.3 Alternative Trie Representations and Benchmark Ladder (Approved Plan — High Priority)
- **Background**: The repository's architecture specification (`docs/DESIGN_SPECIFICATION.md`) documents an alternative-trie ladder to compare the current indexed arena against other data structures.
- **Implementations to Compare**:
  1. *Baseline*: Arena-allocated `Vec<Node>` with 26-way child index arrays (`Option<NodeId>`).
  2. *Variant A*: Boxed child arrays (`Box<[Node]>`).
  3. *Variant B*: Adaptive child storage utilizing `SmallVec<[Child; K]>` for sparse nodes.
  4. *Variant C*: Associative child maps using `HashMap<u8, NodeId>` or `BTreeMap<u8, NodeId>`.
- **Benchmarking Suite**: Run rigorous memory footprint, cache locality, insertion throughput, and prefix traversal benchmarks under `benches/` using synthetic dictionaries and the bundled 178,691-word corpus.

### 3.4 Spelling Bee Baseline vs. Pruned-Trie Comparison (Approved Plan — Medium Priority)
- Expand the existing Spelling Bee benchmark harness to systematically contrast complete linear dictionary filtering against trie-guided depth-first search across all difficulty presets.
- Measure peak memory allocation, search speed, and pangram identification latency.

### 3.5 Boggle and Wordle Enhancements (Proposed Improvement — Medium Priority)
- **Boggle**:
  - Support larger board configurations (e.g. 8×8 grids for custom word search).
  - Add interactive word-path backtracking visualization showing letter paths on the board.
  - Implement alternative board generators with configurable letter frequency profiles.
- **Wordle**:
  - Support "Hard Mode" rules (revealed green and yellow hints must be used in subsequent guesses).
  - Variable word lengths (support for 4-letter, 6-letter, and 7-letter words using trie-curated lexicons).
  - Seeded daily challenge generator with deterministic calendar date mapping.

### 3.6 Mastermind-Related Reuse (Proposed Improvement — Medium Priority)
- Review constraint-satisfaction heuristics and letter-scoring pipelines to generalize the color-code matching logic from Mastermind for multi-character word game solvers and clue analysis.

### 3.7 Packaging, Documentation, and GitHub Presentation (Proposed Improvement — Long-Term)
- Prepare `trie` and `countdown-timer` for potential publication on [crates.io](https://crates.io) as standalone zero-dependency libraries.
- Automate terminal recording generation (SVG or asciinema) for documentation and GitHub social preview assets.
