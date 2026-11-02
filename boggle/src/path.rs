//! Independent per-word board DFS and on-demand validation oracle (Milestone BG-S1).
//!
//! Provides an independent path search on the Boggle board graph:
//! - Validates whether a candidate word can be spelled on the board adhering to Boggle rules.
//! - Uses a copied `u64` bitmask to ensure no cell is visited twice in the same word.
//! - Permits using different cells with the same letter.
//! - Handles `Qu` tiles consuming one cell while advancing two spelled letters (`Q` then `U`).
//! - Does NOT use trie-prefix pruning or call the production solver.

use trie::{Letter, Trie};

use crate::board::{Board, Tile, LETTER_Q, LETTER_U};

/// Minimum word length required for Boggle words.
pub const MIN_WORD_LENGTH: usize = 3;

/// Result of evaluating a single word against the on-demand validator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OnDemandResult {
    /// Word is in the dictionary, on the board, and satisfies all Boggle rules.
    Valid { word: String, path: Vec<usize> },
    /// Word is shorter than the minimum required length (3).
    TooShort {
        word: String,
        min_length: usize,
        actual_length: usize,
    },
    /// Word contains invalid (non-ASCII) characters.
    InvalidWord { word: String, reason: String },
    /// Word is not present in the dictionary trie.
    NotInDictionary { word: String },
    /// Word is in the dictionary, but cannot be formed on the board.
    NotOnBoard { word: String },
}

/// Checks whether a normalized word can be formed on the board via valid 8-neighbor moves
/// without reusing any cell.
///
/// Returns `true` if at least one valid path exists on the board.
pub fn contains_word(board: &Board, word: &str) -> bool {
    let letters = match parse_word_letters(word) {
        Some(letters) if !letters.is_empty() => letters,
        _ => return false,
    };

    let total_cells = board.cell_count();
    for start_cell in 0..total_cells {
        let tile = board.tile(start_cell);
        match tile {
            Tile::Letter(l) => {
                if letters[0] == l {
                    let mask = 1u64 << start_cell;
                    if dfs_contains(board, &letters, 1, start_cell, mask) {
                        return true;
                    }
                }
            }
            Tile::Qu => {
                if letters.len() >= 2 && letters[0] == LETTER_Q && letters[1] == LETTER_U {
                    let mask = 1u64 << start_cell;
                    if dfs_contains(board, &letters, 2, start_cell, mask) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Recursive DFS helper for `contains_word` using copied `u64` bitmask.
fn dfs_contains(
    board: &Board,
    letters: &[Letter],
    letter_pos: usize,
    current_cell: usize,
    visited_mask: u64,
) -> bool {
    if letter_pos == letters.len() {
        return true;
    }

    for &neighbor in board.neighbors(current_cell) {
        if (visited_mask & (1u64 << neighbor)) != 0 {
            continue;
        }

        match board.tile(neighbor) {
            Tile::Letter(l) => {
                if letters[letter_pos] == l {
                    let next_mask = visited_mask | (1u64 << neighbor);
                    if dfs_contains(board, letters, letter_pos + 1, neighbor, next_mask) {
                        return true;
                    }
                }
            }
            Tile::Qu => {
                if letter_pos + 1 < letters.len()
                    && letters[letter_pos] == LETTER_Q
                    && letters[letter_pos + 1] == LETTER_U
                {
                    let next_mask = visited_mask | (1u64 << neighbor);
                    if dfs_contains(board, letters, letter_pos + 2, neighbor, next_mask) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Finds an exact cell path for the given word on the board, if one exists.
///
/// Returns `Some(Vec<usize>)` containing the sequential linear cell indices from start to end,
/// or `None` if the word cannot be formed on the board.
pub fn find_word_path(board: &Board, word: &str) -> Option<Vec<usize>> {
    let letters = match parse_word_letters(word) {
        Some(letters) if !letters.is_empty() => letters,
        _ => return None,
    };

    let total_cells = board.cell_count();
    let mut path = Vec::with_capacity(letters.len());

    for start_cell in 0..total_cells {
        let tile = board.tile(start_cell);
        match tile {
            Tile::Letter(l) => {
                if letters[0] == l {
                    path.push(start_cell);
                    let mask = 1u64 << start_cell;
                    if dfs_path(board, &letters, 1, start_cell, mask, &mut path) {
                        return Some(path);
                    }
                    path.pop();
                }
            }
            Tile::Qu => {
                if letters.len() >= 2 && letters[0] == LETTER_Q && letters[1] == LETTER_U {
                    path.push(start_cell);
                    let mask = 1u64 << start_cell;
                    if dfs_path(board, &letters, 2, start_cell, mask, &mut path) {
                        return Some(path);
                    }
                    path.pop();
                }
            }
        }
    }

    None
}

/// Recursive DFS helper for `find_word_path` recording linear cell indices.
fn dfs_path(
    board: &Board,
    letters: &[Letter],
    letter_pos: usize,
    current_cell: usize,
    visited_mask: u64,
    path: &mut Vec<usize>,
) -> bool {
    if letter_pos == letters.len() {
        return true;
    }

    for &neighbor in board.neighbors(current_cell) {
        if (visited_mask & (1u64 << neighbor)) != 0 {
            continue;
        }

        match board.tile(neighbor) {
            Tile::Letter(l) => {
                if letters[letter_pos] == l {
                    path.push(neighbor);
                    let next_mask = visited_mask | (1u64 << neighbor);
                    if dfs_path(board, letters, letter_pos + 1, neighbor, next_mask, path) {
                        return true;
                    }
                    path.pop();
                }
            }
            Tile::Qu => {
                if letter_pos + 1 < letters.len()
                    && letters[letter_pos] == LETTER_Q
                    && letters[letter_pos + 1] == LETTER_U
                {
                    path.push(neighbor);
                    let next_mask = visited_mask | (1u64 << neighbor);
                    if dfs_path(board, letters, letter_pos + 2, neighbor, next_mask, path) {
                        return true;
                    }
                    path.pop();
                }
            }
        }
    }

    false
}

/// On-demand validation of a guess word: checks minimum length, dictionary membership,
/// and board path availability independently.
pub fn validate_on_demand(board: &Board, trie: &Trie, word: &str) -> OnDemandResult {
    let normalized = match trie::normalize_word(word) {
        Ok(norm) => norm.into_owned(),
        Err(e) => {
            return OnDemandResult::InvalidWord {
                word: word.to_string(),
                reason: e.to_string(),
            }
        }
    };

    if normalized.len() < MIN_WORD_LENGTH {
        return OnDemandResult::TooShort {
            word: normalized.clone(),
            min_length: MIN_WORD_LENGTH,
            actual_length: normalized.len(),
        };
    }

    if !trie.contains(&normalized) {
        return OnDemandResult::NotInDictionary { word: normalized };
    }

    match find_word_path(board, &normalized) {
        Some(path) => OnDemandResult::Valid {
            word: normalized,
            path,
        },
        None => OnDemandResult::NotOnBoard { word: normalized },
    }
}

/// Parses a word string into a sequence of `Letter`s, returning `None` if non-ASCII letters occur.
fn parse_word_letters(word: &str) -> Option<Vec<Letter>> {
    let mut letters = Vec::with_capacity(word.len());
    for b in word.bytes() {
        letters.push(Letter::from_ascii_byte(b)?);
    }
    Some(letters)
}
