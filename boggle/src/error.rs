//! Error types for the Boggle engine.

use std::fmt;

/// Errors that can occur during Boggle board construction and validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoardError {
    /// Board dimension must be 4, 5, or 6.
    InvalidSize { size: usize },
    /// Total number of tiles does not match `size * size`.
    InvalidTileCount { expected: usize, actual: usize },
    /// An invalid (non-ASCII or non-alphabetic) character was supplied.
    InvalidCharacter { ch: char },
    /// Tile input string or slice was empty.
    EmptyInput,
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize { size } => {
                write!(f, "invalid board dimension {size}: must be 4, 5, or 6")
            }
            Self::InvalidTileCount { expected, actual } => {
                write!(f, "expected {expected} tiles, but received {actual} tiles")
            }
            Self::InvalidCharacter { ch } => {
                write!(f, "invalid character '{ch}': must be ASCII letter A-Z")
            }
            Self::EmptyInput => write!(f, "tile input cannot be empty"),
        }
    }
}

impl std::error::Error for BoardError {}

/// Errors that can occur during scoring.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScoringError {
    /// Score computation encountered an arithmetic overflow.
    ArithmeticOverflow,
}

impl fmt::Display for ScoringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArithmeticOverflow => write!(f, "score computation overflowed u32 range"),
        }
    }
}

impl std::error::Error for ScoringError {}

/// Errors that can occur during solution solving.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SolverError {
    /// Search expansion budget or recursion guard was exceeded.
    SearchLimitExceeded { expansions: usize, limit: usize },
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SearchLimitExceeded { expansions, limit } => {
                write!(
                    f,
                    "search limit exceeded: {expansions} expansions reached limit of {limit}"
                )
            }
        }
    }
}

impl std::error::Error for SolverError {}

/// Errors that can occur during board generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenerationError {
    /// Normal play retry budget exhausted without finding a playable (non-empty) board.
    NoPlayableBoard { retries: usize },
    /// Unsupported board size for generation.
    InvalidSize { size: usize },
    /// Dice configuration or table generation error.
    DiceConfigError { reason: String },
}

impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoPlayableBoard { retries } => {
                write!(
                    f,
                    "failed to generate a playable board with valid words after {retries} retries"
                )
            }
            Self::InvalidSize { size } => {
                write!(f, "cannot generate board for invalid size {size}")
            }
            Self::DiceConfigError { reason } => {
                write!(f, "dice configuration error: {reason}")
            }
        }
    }
}

impl std::error::Error for GenerationError {}

/// Errors that can occur during session setup or management.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionError {
    /// Solutions contain duplicate words.
    DuplicateWordsInSolutions,
    /// Solutions are not in strictly ascending lexicographic order.
    UnsortedSolutions,
    /// An invalid word was encountered.
    InvalidWord { word: String, reason: String },
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateWordsInSolutions => {
                write!(f, "solution set contains duplicate words")
            }
            Self::UnsortedSolutions => {
                write!(f, "solution set is not sorted lexicographically")
            }
            Self::InvalidWord { word, reason } => {
                write!(f, "invalid word '{word}': {reason}")
            }
        }
    }
}

impl std::error::Error for SessionError {}

/// Unified error enum for the Boggle engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoggleError {
    /// Board construction or validation error.
    Board(BoardError),
    /// Scoring error.
    Scoring(ScoringError),
    /// Solver error.
    Solver(SolverError),
    /// Generation error.
    Generation(GenerationError),
    /// Session error.
    Session(SessionError),
    /// Underlying trie word error.
    Word(trie::WordError),
}

impl fmt::Display for BoggleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Board(e) => write!(f, "board error: {e}"),
            Self::Scoring(e) => write!(f, "scoring error: {e}"),
            Self::Solver(e) => write!(f, "solver error: {e}"),
            Self::Generation(e) => write!(f, "generation error: {e}"),
            Self::Session(e) => write!(f, "session error: {e}"),
            Self::Word(e) => write!(f, "word error: {e}"),
        }
    }
}

impl std::error::Error for BoggleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Board(e) => Some(e),
            Self::Scoring(e) => Some(e),
            Self::Solver(e) => Some(e),
            Self::Generation(e) => Some(e),
            Self::Session(e) => Some(e),
            Self::Word(e) => Some(e),
        }
    }
}

impl From<BoardError> for BoggleError {
    fn from(e: BoardError) -> Self {
        Self::Board(e)
    }
}

impl From<ScoringError> for BoggleError {
    fn from(e: ScoringError) -> Self {
        Self::Scoring(e)
    }
}

impl From<SolverError> for BoggleError {
    fn from(e: SolverError) -> Self {
        Self::Solver(e)
    }
}

impl From<GenerationError> for BoggleError {
    fn from(e: GenerationError) -> Self {
        Self::Generation(e)
    }
}

impl From<SessionError> for BoggleError {
    fn from(e: SessionError) -> Self {
        Self::Session(e)
    }
}

impl From<trie::WordError> for BoggleError {
    fn from(e: trie::WordError) -> Self {
        Self::Word(e)
    }
}
