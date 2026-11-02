//! Error types for the Spelling Bee engine.

use std::fmt;

/// Errors that can occur during puzzle construction and validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PuzzleError {
    /// Puzzle must have exactly seven distinct letters.
    InvalidDistinctCount {
        expected: usize,
        actual: usize,
        letters: String,
    },
    /// Designated central letter is not one of the seven puzzle letters.
    CenterNotInLetters { center: char, letters: String },
    /// An invalid (non-ASCII or non-alphabetic) character was supplied.
    InvalidCharacter { ch: char },
    /// Empty input was provided.
    EmptyInput,
}

impl fmt::Display for PuzzleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDistinctCount {
                expected,
                actual,
                letters,
            } => write!(
                f,
                "expected {expected} distinct letters, got {actual} from '{letters}'"
            ),
            Self::CenterNotInLetters { center, letters } => {
                write!(
                    f,
                    "center letter '{center}' is not among puzzle letters '{letters}'"
                )
            }
            Self::InvalidCharacter { ch } => {
                write!(f, "invalid character '{ch}': must be ASCII letter A-Z")
            }
            Self::EmptyInput => write!(f, "letter input cannot be empty"),
        }
    }
}

impl std::error::Error for PuzzleError {}

/// Errors that explain why a submitted guess is invalid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuessError {
    /// Word is shorter than the minimum required length (4).
    TooShort {
        min_length: usize,
        actual_length: usize,
    },
    /// Word does not contain the mandatory center letter.
    MissingCenter { center: char },
    /// Word contains a character not in the puzzle's 7 allowed letters.
    InvalidLetter { letter: char },
    /// Word uses allowed letters and center, but is not in the dictionary.
    NotInDictionary,
    /// Malformed characters in guess.
    MalformedInput { ch: char },
}

impl fmt::Display for GuessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort {
                min_length,
                actual_length,
            } => write!(
                f,
                "too short: word has {actual_length} letters, minimum is {min_length}"
            ),
            Self::MissingCenter { center } => {
                write!(f, "missing center letter '{center}'")
            }
            Self::InvalidLetter { letter } => {
                write!(f, "letter '{letter}' is not in the puzzle")
            }
            Self::NotInDictionary => write!(f, "not in dictionary"),
            Self::MalformedInput { ch } => {
                write!(f, "invalid character '{ch}': must be ASCII letter A-Z")
            }
        }
    }
}

impl std::error::Error for GuessError {}

/// Errors that can occur during procedural puzzle generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenerationError {
    /// No anchor words with 7 distinct letters were found in the dictionary.
    EmptyAnchorIndex,
    /// Failed to find a puzzle meeting constraints within retry budget.
    NoSuitablePuzzle { retries: usize },
    /// The specified min_words bound exceeds max_words.
    InvalidBounds { min: usize, max: usize },
    /// The specified center letter is not in the anchor word.
    CenterNotInAnchor { center: char, anchor: String },
    /// Unknown or invalid difficulty preset name.
    InvalidPreset(String),
}

impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAnchorIndex => write!(
                f,
                "dictionary contains no words with exactly 7 distinct letters"
            ),
            Self::NoSuitablePuzzle { retries } => write!(
                f,
                "no suitable puzzle found satisfying constraints within {retries} retries"
            ),
            Self::InvalidBounds { min, max } => {
                write!(f, "invalid word count bounds: min ({min}) > max ({max})")
            }
            Self::CenterNotInAnchor { center, anchor } => write!(
                f,
                "center letter '{center}' is not present in anchor '{anchor}'"
            ),
            Self::InvalidPreset(name) => write!(f, "invalid difficulty preset: '{name}'"),
        }
    }
}

impl std::error::Error for GenerationError {}
