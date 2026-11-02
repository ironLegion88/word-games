//! Boggle solution enumeration algorithms.
//!
//! Provides two distinct algorithms:
//! - `BG-S1E`: Dictionary scan baseline (`solve_scan`) using the independent per-word board DFS.
//! - `BG-S2`: Trie-guided board DFS (`solve_trie`) pruning nonexistent prefixes at each tile.

use trie::Trie;

use crate::board::Board;
use crate::error::SolverError;
use crate::path::{contains_word, MIN_WORD_LENGTH};

/// Strategy selector for the Boggle solution solver.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SolverStrategy {
    /// Full dictionary scan and independent path DFS baseline (BG-S1E).
    #[default]
    Scan,
    /// Trie-guided board DFS with prefix pruning (BG-S2).
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

/// Temporary placeholder for BG-S2 until Milestone BG-S2 is implemented.
/// Falls back to `solve_scan`.
pub fn solve_trie(board: &Board, trie: &Trie) -> Result<Vec<String>, SolverError> {
    solve_scan(board, trie)
}
