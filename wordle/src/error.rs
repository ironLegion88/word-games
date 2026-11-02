//! Typed errors for the Wordle game engine.

use std::fmt;

/// Errors that can occur during Wordle input validation, lexicon loading, and gameplay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordleError {
    /// Input string was empty or contained only whitespace.
    EmptyInput,
    /// Word length did not match the expected 5 ASCII letters.
    InvalidLength { expected: usize, actual: usize },
    /// Non-ASCII or non-alphabetic character encountered.
    InvalidCharacter { character: char, position: usize },
    /// Word is not present in the allowed dictionary.
    NotInDictionary(String),
    /// Proposed secret is not in the curated/allowed answer subset.
    NotAnAnswer(String),
    /// Answer candidate list was empty.
    EmptyAnswers,
    /// Answer override file contains an invalid or malformed line.
    InvalidAnswerOverride {
        line_number: usize,
        line: String,
        reason: String,
    },
    /// An action or submission was attempted on a completed/terminal game session.
    GameFinished,
}

impl fmt::Display for WordleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WordleError::EmptyInput => write!(f, "Input cannot be empty. Enter a 5-letter word."),
            WordleError::InvalidLength { expected, actual } => {
                write!(
                    f,
                    "Invalid word length: expected {expected} letters, got {actual}."
                )
            }
            WordleError::InvalidCharacter {
                character,
                position,
            } => {
                write!(
                    f,
                    "Invalid character '{character}' at position {position}: only ASCII letters (A-Z) are allowed."
                )
            }
            WordleError::NotInDictionary(word) => {
                write!(f, "Word '{word}' is not in the allowed dictionary.")
            }
            WordleError::NotAnAnswer(word) => {
                write!(f, "Word '{word}' is not in the allowed answer subset.")
            }
            WordleError::EmptyAnswers => {
                write!(f, "Answer list cannot be empty.")
            }
            WordleError::InvalidAnswerOverride {
                line_number,
                line,
                reason,
            } => {
                write!(
                    f,
                    "Invalid answer override at line {line_number} ('{line}'): {reason}"
                )
            }
            WordleError::GameFinished => {
                write!(
                    f,
                    "Game has already finished; no further submissions accepted."
                )
            }
        }
    }
}

impl std::error::Error for WordleError {}
