use std::fmt;

/// Errors that can occur when validating or normalizing a word for the trie.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WordError {
    /// Word is empty (zero characters).
    EmptyWord,
    /// Invalid character encountered in a word.
    InvalidCharacter {
        /// The invalid character encountered.
        character: char,
        /// The byte position where the invalid character was found.
        position: usize,
    },
    /// The trie capacity limit has been exceeded.
    CapacityExceeded {
        /// Detail about the capacity limit reached.
        message: String,
    },
}

impl std::error::Error for WordError {}

impl fmt::Display for WordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyWord => write!(f, "word cannot be empty"),
            Self::InvalidCharacter {
                character,
                position,
            } => {
                write!(
                    f,
                    "invalid character '{character}' at byte index {position}: words must contain only ASCII letters (A-Z, a-z)",
                )
            }
            Self::CapacityExceeded { message } => {
                write!(f, "trie capacity exceeded: {message}")
            }
        }
    }
}

/// Errors that can occur when reading, parsing, or loading a word list or file.
#[derive(Debug)]
pub enum LoadError {
    /// An I/O error occurred while reading from the stream or file.
    Io {
        source: std::io::Error,
        path: Option<String>,
    },
    /// A word parsing or validation error occurred.
    InvalidWord {
        source: WordError,
        line: usize,
        column: usize,
        token: String,
    },
    /// The input text was not valid UTF-8.
    InvalidUtf8 {
        source: std::str::Utf8Error,
        line: usize,
    },
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidWord { source, .. } => Some(source),
            Self::InvalidUtf8 { source, .. } => Some(source),
        }
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { source, path } => match path {
                Some(p) => write!(f, "I/O error reading '{p}': {source}"),
                None => write!(f, "I/O error: {source}"),
            },
            Self::InvalidWord {
                source,
                line,
                column,
                token,
            } => {
                write!(
                    f,
                    "invalid word \"{token}\" at line {line}, column {column}: {source}",
                )
            }
            Self::InvalidUtf8 { source, line } => {
                write!(f, "invalid UTF-8 encoding at line {line}: {source}")
            }
        }
    }
}

impl From<std::io::Error> for LoadError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            source: err,
            path: None,
        }
    }
}
