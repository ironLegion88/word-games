//! Boggle solution enumeration algorithms.
//!
//! Provides two distinct algorithms:
//! - `BG-S1E`: Dictionary scan baseline (`solve_scan`) using the independent per-word board DFS.
//! - `BG-S2`: Trie-guided board DFS (`solve_trie`) pruning nonexistent prefixes at each tile.

use std::collections::BTreeSet;
use trie::{Trie, TrieCursor};

use crate::board::{Board, Tile, LETTER_Q, LETTER_U};
use crate::error::SolverError;
use crate::path::{contains_word, MIN_WORD_LENGTH};

/// Strategy selector for the Boggle solution solver.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SolverStrategy {
    /// Full dictionary scan and independent path DFS baseline (BG-S1E).
    Scan,
    /// Trie-guided board DFS with prefix pruning (BG-S2). Default production solver.
    #[default]
    Trie,
}

impl SolverStrategy {
    /// Parses a strategy from a CLI string argument.
    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "scan" | "filter" => Some(Self::Scan),
            "trie" | "dfs" => Some(Self::Trie),
            _ => None,
        }
    }
}

/// Solves a Boggle board against the given trie using the specified strategy.
///
/// Returns a sorted, unique vector of all valid words that can be spelled on the board.
pub fn solve(
    board: &Board,
    trie: &Trie,
    strategy: SolverStrategy,
) -> Result<Vec<String>, SolverError> {
    match strategy {
        SolverStrategy::Scan => solve_scan(board, trie),
        SolverStrategy::Trie => solve_trie(board, trie),
    }
}

/// BG-S1E: Full dictionary scan solver.
///
/// Iterates every word in the dictionary via `trie.visit_words` in lexicographical order,
/// checking whether each word of length >= 3 can be formed on the board via `contains_word`.
///
/// Because `trie.visit_words` yields words in sorted order, the resulting vector is
/// naturally sorted and unique.
pub fn solve_scan(board: &Board, trie: &Trie) -> Result<Vec<String>, SolverError> {
    let mut solutions = Vec::new();

    trie.visit_words(|word| {
        if word.len() >= MIN_WORD_LENGTH && contains_word(board, word) {
            solutions.push(word.to_string());
        }
    });

    Ok(solutions)
}

/// BG-S2: Trie-guided board DFS solver (production default).
///
/// Starts a search at each tile of the board, carrying an immutable `TrieCursor`
/// and a copied `u64` visited mask. Nonexistent prefixes are pruned immediately,
/// preventing exploration of dead branches.
///
/// Deduplicates words by value (using a `BTreeSet`), producing a strictly sorted,
/// unique solution list.
pub fn solve_trie(board: &Board, trie: &Trie) -> Result<Vec<String>, SolverError> {
    solve_trie_with_limit(board, trie, None)
}

/// BG-S2 with an optional expansion limit guard.
///
/// If `max_expansions` is exceeded, terminates immediately with `Err(SolverError::SearchLimitExceeded)`
/// and publishes no partial session.
pub fn solve_trie_with_limit(
    board: &Board,
    trie: &Trie,
    max_expansions: Option<usize>,
) -> Result<Vec<String>, SolverError> {
    let mut found_words = BTreeSet::new();
    let mut expansions = 0usize;
    let mut word_buf = String::with_capacity(36);
    let root_cursor = trie.cursor();
    let total_cells = board.cell_count();

    for start_cell in 0..total_cells {
        let tile = board.tile(start_cell);
        match tile {
            Tile::Letter(l) => {
                if let Some(cursor) = root_cursor.advance(l) {
                    word_buf.push(l.to_char());
                    let mask = 1u64 << start_cell;
                    dfs_trie(
                        board,
                        start_cell,
                        cursor,
                        mask,
                        &mut word_buf,
                        &mut found_words,
                        &mut expansions,
                        max_expansions,
                    )?;
                    word_buf.pop();
                }
            }
            Tile::Qu => {
                if let Some(q_cursor) = root_cursor.advance(LETTER_Q) {
                    if let Some(u_cursor) = q_cursor.advance(LETTER_U) {
                        word_buf.push('Q');
                        word_buf.push('U');
                        let mask = 1u64 << start_cell;
                        dfs_trie(
                            board,
                            start_cell,
                            u_cursor,
                            mask,
                            &mut word_buf,
                            &mut found_words,
                            &mut expansions,
                            max_expansions,
                        )?;
                        word_buf.pop();
                        word_buf.pop();
                    }
                }
            }
        }
    }

    Ok(found_words.into_iter().collect())
}

#[allow(clippy::too_many_arguments)]
fn dfs_trie(
    board: &Board,
    current_cell: usize,
    cursor: TrieCursor<'_>,
    visited_mask: u64,
    word_buf: &mut String,
    found_words: &mut BTreeSet<String>,
    expansions: &mut usize,
    max_expansions: Option<usize>,
) -> Result<(), SolverError> {
    *expansions += 1;
    if let Some(limit) = max_expansions {
        if *expansions > limit {
            return Err(SolverError::SearchLimitExceeded {
                expansions: *expansions,
                limit,
            });
        }
    }

    if word_buf.len() >= MIN_WORD_LENGTH && cursor.is_word() {
        found_words.insert(word_buf.clone());
    }

    if !cursor.has_children() {
        return Ok(());
    }

    for &neighbor in board.neighbors(current_cell) {
        if (visited_mask & (1u64 << neighbor)) != 0 {
            continue;
        }

        match board.tile(neighbor) {
            Tile::Letter(l) => {
                if let Some(next_cursor) = cursor.advance(l) {
                    word_buf.push(l.to_char());
                    let next_mask = visited_mask | (1u64 << neighbor);
                    dfs_trie(
                        board,
                        neighbor,
                        next_cursor,
                        next_mask,
                        word_buf,
                        found_words,
                        expansions,
                        max_expansions,
                    )?;
                    word_buf.pop();
                }
            }
            Tile::Qu => {
                if let Some(q_cursor) = cursor.advance(LETTER_Q) {
                    if let Some(u_cursor) = q_cursor.advance(LETTER_U) {
                        word_buf.push('Q');
                        word_buf.push('U');
                        let next_mask = visited_mask | (1u64 << neighbor);
                        dfs_trie(
                            board,
                            neighbor,
                            u_cursor,
                            next_mask,
                            word_buf,
                            found_words,
                            expansions,
                            max_expansions,
                        )?;
                        word_buf.pop();
                        word_buf.pop();
                    }
                }
            }
        }
    }

    Ok(())
}
