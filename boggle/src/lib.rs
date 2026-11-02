//! Boggle game engine crate.
//!
//! A high-performance, modular implementation of the Boggle word game:
//! - Flexible `N × N` grid support (`N ∈ {4, 5, 6}`) with precomputed 8-neighbor adjacency.
//! - Visited-cell tracking via copied `u64` bitmasks (bit 35 included).
//! - Independent per-word path validation (`contains_word`, `find_word_path`, `validate_on_demand`).
//! - Retained solution solvers: `scan` (BG-S1E) and `trie` (BG-S2).
//! - Three selectable board generators: `uniform` (BG-G1), `weighted` (BG-G2), and `dice` (BG-G3).
//! - Extended Boggle scoring schedule with custom-dictionary support (`≥15` letters at `4 × length`).
//! - Self-contained owned game session with cached-score invariant and graceful early exit.

pub mod board;
pub mod error;
pub mod generation;
pub mod path;
pub mod scoring;
pub mod session;
pub mod solver;

pub use board::{Board, Tile, LETTER_Q, LETTER_U, MAX_BOARD_SIZE, MIN_BOARD_SIZE};
pub use error::{
    BoardError, BoggleError, GenerationError, ScoringError, SessionError, SolverError,
};
pub use generation::{
    generate_playable_board, generate_raw_board, generate_uniform, BoardGenerator,
    DEFAULT_RETRY_BUDGET,
};
pub use path::{
    contains_word, find_word_path, validate_on_demand, OnDemandResult, MIN_WORD_LENGTH,
};
pub use scoring::{calculate_total_score, word_score};
pub use session::{BoggleSession, GuessOutcome, QuitSummary};
pub use solver::{solve, solve_scan, solve_trie, SolverStrategy};
