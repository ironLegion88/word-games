use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::{LoadError, WordError};
use crate::letter::Letter;
use crate::Trie;

/// Summary report of a dictionary load or extend operation.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct LoadReport {
    /// Total number of valid word tokens encountered in the input.
    pub tokens_seen: usize,
    /// Number of words that were newly added to the dictionary.
    pub inserted: usize,
    /// Number of words that were already present in the dictionary.
    pub already_present: usize,
}

/// Summary report of a batch deletion operation.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct BatchReport {
    /// Total number of word tokens requested for deletion.
    pub requested: usize,
    /// Number of distinct words successfully removed from the dictionary.
    pub removed: usize,
    /// Number of tokens that were not found in the dictionary (including duplicate removal requests).
    pub not_found: usize,
}

/// A parsed and normalized word token with its origin location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedToken {
    /// Normalized uppercase ASCII word.
    pub word: String,
    /// 1-indexed line number where the word was found.
    pub line: usize,
    /// 1-indexed column number where the first character of the word appeared.
    pub column: usize,
}

/// Reads all lines from a `BufRead`, validates every token, and returns a list of normalized tokens.
///
/// Delimiters are commas (`','`) and ASCII whitespace (`' '`, `'\t'`, `'\r'`, `'\n'`).
/// Repeated separators and blank lines are ignored.
/// A leading UTF-8 Byte Order Mark (BOM) on line 1 is automatically tolerated and stripped.
/// Any invalid character (punctuation such as semicolons, hyphens, digits, or non-ASCII characters)
/// immediately returns a detailed `LoadError` identifying the offending character and its exact line and column.
pub fn parse_tokens<R: BufRead>(mut reader: R) -> Result<Vec<ParsedToken>, LoadError> {
    let mut tokens = Vec::new();
    let mut line_buf = String::new();
    let mut line_number = 1usize;

    loop {
        line_buf.clear();
        let bytes_read = match reader.read_line(&mut line_buf) {
            Ok(n) => n,
            Err(e) => {
                return Err(LoadError::Io {
                    source: e,
                    path: None,
                })
            }
        };

        if bytes_read == 0 {
            break; // EOF
        }

        let mut line_str = line_buf.as_str();

        // Strip UTF-8 BOM on the very first line if present
        if line_number == 1 && line_str.starts_with('\u{FEFF}') {
            line_str = &line_str['\u{FEFF}'.len_utf8()..];
        }

        let mut current_token = String::new();
        let mut token_start_col = 1usize;

        for (col, ch) in (1usize..).zip(line_str.chars()) {
            if ch == ',' || ch.is_ascii_whitespace() {
                // Separator encountered
                if !current_token.is_empty() {
                    tokens.push(ParsedToken {
                        word: current_token.clone(),
                        line: line_number,
                        column: token_start_col,
                    });
                    current_token.clear();
                }
            } else if let Some(letter) = Letter::from_char(ch) {
                // Valid ASCII alphabetic character
                if current_token.is_empty() {
                    token_start_col = col;
                }
                current_token.push(letter.to_char());
            } else {
                // Invalid character encountered
                let err_col = col;
                current_token.push(ch);
                return Err(LoadError::InvalidWord {
                    source: WordError::InvalidCharacter {
                        character: ch,
                        position: current_token.len().saturating_sub(1),
                    },
                    line: line_number,
                    column: err_col,
                    token: current_token,
                });
            }
        }

        // Flush any trailing token on this line (e.g. final line without newline)
        if !current_token.is_empty() {
            tokens.push(ParsedToken {
                word: current_token,
                line: line_number,
                column: token_start_col,
            });
        }

        line_number += 1;
    }

    Ok(tokens)
}

impl Trie {
    /// Builds a new `Trie` by reading, validating, and inserting words from a buffered reader.
    ///
    /// Accepts comma-separated, whitespace-separated, one-word-per-line, or mixed files.
    /// Fails atomically: if an I/O error or invalid character is encountered, no partial trie is created.
    pub fn from_reader<R: BufRead>(reader: R) -> Result<Self, LoadError> {
        let mut trie = Self::new();
        trie.extend_from_reader(reader)?;
        Ok(trie)
    }

    /// Extends this `Trie` with all words read from a buffered reader.
    ///
    /// All tokens are validated before any modification to `self`. If an invalid token
    /// or I/O error occurs, `self` remains completely unchanged.
    pub fn extend_from_reader<R: BufRead>(&mut self, reader: R) -> Result<LoadReport, LoadError> {
        let tokens = parse_tokens(reader)?;
        let mut report = LoadReport {
            tokens_seen: tokens.len(),
            inserted: 0,
            already_present: 0,
        };

        for token in tokens {
            if self
                .insert(&token.word)
                .map_err(|e| LoadError::InvalidWord {
                    source: e,
                    line: token.line,
                    column: token.column,
                    token: token.word.clone(),
                })?
            {
                report.inserted += 1;
            } else {
                report.already_present += 1;
            }
        }

        Ok(report)
    }

    /// Loads a dictionary from a file path, replacing any existing content.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, LoadError> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref).map_err(|e| LoadError::Io {
            source: e,
            path: Some(path_ref.display().to_string()),
        })?;
        let reader = BufReader::new(file);
        Self::from_reader(reader)
    }

    /// Extends this dictionary from a file path.
    pub fn extend_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<LoadReport, LoadError> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref).map_err(|e| LoadError::Io {
            source: e,
            path: Some(path_ref.display().to_string()),
        })?;
        let reader = BufReader::new(file);
        self.extend_from_reader(reader)
    }

    /// Removes a sequence of words from the trie.
    ///
    /// All words are validated before modification starts. If any word is invalid,
    /// returns `Err(WordError)` and `self` remains completely unchanged.
    /// Automatic compaction is deferred until after all removals in the batch complete.
    pub fn remove_words<'a, I>(&mut self, words: I) -> Result<BatchReport, WordError>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let word_list: Vec<&'a str> = words.into_iter().collect();

        // Validate all words upfront to guarantee atomicity
        for &w in &word_list {
            crate::letter::normalize_word(w)?;
        }

        let mut report = BatchReport {
            requested: word_list.len(),
            removed: 0,
            not_found: 0,
        };

        for &w in &word_list {
            if self.remove_internal(w, false)? {
                report.removed += 1;
            } else {
                report.not_found += 1;
            }
        }

        self.maybe_compact();
        Ok(report)
    }

    /// Removes all words listed in a reader from the trie.
    ///
    /// All tokens are validated before modification starts.
    pub fn remove_from_reader<R: BufRead>(&mut self, reader: R) -> Result<BatchReport, LoadError> {
        let tokens = parse_tokens(reader)?;
        let mut report = BatchReport {
            requested: tokens.len(),
            removed: 0,
            not_found: 0,
        };

        for token in tokens {
            if self
                .remove_internal(&token.word, false)
                .map_err(|e| LoadError::InvalidWord {
                    source: e,
                    line: token.line,
                    column: token.column,
                    token: token.word.clone(),
                })?
            {
                report.removed += 1;
            } else {
                report.not_found += 1;
            }
        }

        self.maybe_compact();
        Ok(report)
    }

    /// Removes all words listed in a file from the trie.
    pub fn remove_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<BatchReport, LoadError> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref).map_err(|e| LoadError::Io {
            source: e,
            path: Some(path_ref.display().to_string()),
        })?;
        let reader = BufReader::new(file);
        self.remove_from_reader(reader)
    }

    /// Reads tokens from a reader and returns an ordered `Vec<String>` of all tokens present in the trie.
    ///
    /// Input ordering and duplicates are preserved. All words are returned in uppercase.
    /// Fails atomically on malformed tokens or I/O errors.
    pub fn filter_present_from_reader<R: BufRead>(
        &self,
        reader: R,
    ) -> Result<Vec<String>, LoadError> {
        let tokens = parse_tokens(reader)?;
        let present = tokens
            .into_iter()
            .filter(|t| self.contains(&t.word))
            .map(|t| t.word)
            .collect();
        Ok(present)
    }

    /// Reads tokens from a file and returns an ordered `Vec<String>` of all tokens present in the trie.
    pub fn filter_present_from_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Vec<String>, LoadError> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref).map_err(|e| LoadError::Io {
            source: e,
            path: Some(path_ref.display().to_string()),
        })?;
        let reader = BufReader::new(file);
        self.filter_present_from_reader(reader)
    }

    /// Reads tokens from a reader and returns an ordered `Vec<String>` of all tokens absent from the trie.
    ///
    /// Input ordering and duplicates are preserved. All words are returned in uppercase.
    /// Fails atomically on malformed tokens or I/O errors.
    pub fn filter_absent_from_reader<R: BufRead>(
        &self,
        reader: R,
    ) -> Result<Vec<String>, LoadError> {
        let tokens = parse_tokens(reader)?;
        let absent = tokens
            .into_iter()
            .filter(|t| !self.contains(&t.word))
            .map(|t| t.word)
            .collect();
        Ok(absent)
    }

    /// Reads tokens from a file and returns an ordered `Vec<String>` of all tokens absent from the trie.
    pub fn filter_absent_from_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Vec<String>, LoadError> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref).map_err(|e| LoadError::Io {
            source: e,
            path: Some(path_ref.display().to_string()),
        })?;
        let reader = BufReader::new(file);
        self.filter_absent_from_reader(reader)
    }
}
