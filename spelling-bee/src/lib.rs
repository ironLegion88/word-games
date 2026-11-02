//! Spelling Bee game and solver engine crate.
//!
//! Provides puzzle representation, solution enumeration via dictionary filtering (SB-S1)
//! and trie-pruned DFS (SB-S2), session state management with checked score invariants,
//! hint generation, and procedural anchored puzzle generation.

pub mod error;
pub mod generation;
pub mod hints;
pub mod puzzle;
pub mod session;
pub mod solver;

pub use error::{GenerationError, GuessError, PuzzleError};
pub use generation::{generate_puzzle, AnchorIndex, DifficultyPreset, GenerationConfig};
pub use hints::{HintGrid, TwoLetterCounts};
pub use puzzle::{SpellingBeePuzzle, MIN_WORD_LENGTH, PUZZLE_LETTER_COUNT};
pub use session::{
    calculate_word_score, format_three_columns, GuessOutcome, QuitSummary, SpellingBeeSession,
    PANGRAM_BONUS_POINTS,
};
pub use solver::{solve, solve_filter, solve_trie_pruned, SolverStrategy};
