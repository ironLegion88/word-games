//! High-performance Wordle game engine and feedback algorithms.
//!
//! Wordle is a 5-letter word deduction game with 6 valid attempts.
//! This crate provides:
//! - Validated 5-letter word representations (`Word`).
//! - Feedback algorithms: WL-F1 marker-based scorer (adapted from sibling Mastermind)
//!   and WL-F2 frequency-count scorer (production default).
//! - Owned lexicon management distinguishing allowed guesses from curated answers,
//!   with fail-safe `--answers` override loading.
//! - Monotonic letter-status keyboard tracking.
//! - I/O-free game session lifecycle with strict state transition invariants.

pub mod error;
pub mod feedback;
pub mod keyboard;
pub mod lexicon;
pub mod session;
pub mod word;

// Re-export common public types
pub use error::WordleError;
pub use feedback::{
    evaluate, evaluate_frequency, evaluate_markers, Feedback, FeedbackStrategy, LetterFeedback,
};
pub use keyboard::{KeyStatus, Keyboard};
pub use lexicon::{AnswersOverrideReport, WordleLexicon, BUNDLED_CURATED_ANSWERS_SRC};
pub use session::{
    WordleGame, WordleGuessEvaluation, WordleStatus, WordleSubmissionResult, MAX_ATTEMPTS,
};
pub use word::{Word, WORD_LENGTH};
