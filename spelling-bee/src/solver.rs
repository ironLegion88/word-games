//! Solution enumeration algorithms for Spelling Bee.
//!
//! Provides two distinct algorithms:
//! - `SB-S1`: Full dictionary filtering baseline (visits every dictionary word and checks letter masks).
//! - `SB-S2`: Trie-pruned edge traversal (follows only the 7 allowed letter branches).

use crate::puzzle::{SpellingBeePuzzle, MIN_WORD_LENGTH};
use trie::Trie;

/// Strategy selector for the Spelling Bee solution solver.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SolverStrategy {
    /// Exhaustive dictionary filtering baseline (SB-S1).
    Filter,
    /// Trie-pruned edge traversal (SB-S2). Default production solver.
    #[default]
    TriePruned,
}

impl SolverStrategy {
    /// Parses a strategy from a CLI string argument.
    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "filter" => Some(Self::Filter),
            "trie" | "pruned" => Some(Self::TriePruned),
            _ => None,
        }
    }
}

/// Solves a Spelling Bee puzzle against the given trie using the specified strategy.
///
/// Returns a sorted, unique vector of all valid solution words.
pub fn solve(puzzle: &SpellingBeePuzzle, trie: &Trie, strategy: SolverStrategy) -> Vec<String> {
    match strategy {
        SolverStrategy::Filter => solve_filter(puzzle, trie),
        SolverStrategy::TriePruned => solve_trie_pruned(puzzle, trie),
    }
}

/// SB-S1: Exhaustive dictionary filtering solver.
///
/// Visits every word in the dictionary trie in lexicographic order, validating that:
/// 1. Length is at least `MIN_WORD_LENGTH` (4).
/// 2. Every letter belongs to the puzzle's allowed 7-letter set.
/// 3. The center letter is present.
///
/// Returns an owned, sorted, deduplicated list of solutions.
pub fn solve_filter(puzzle: &SpellingBeePuzzle, trie: &Trie) -> Vec<String> {
    let mut solutions = Vec::new();
    let allowed_mask = puzzle.allowed_mask();
    let center_mask = puzzle.center_mask();

    trie.visit_words(|word| {
        if word.len() < MIN_WORD_LENGTH {
            return;
        }

        let mut mask = 0u32;
        for b in word.bytes() {
            let idx = (b - b'A') as u32;
            let bit = 1 << idx;
            if (allowed_mask & bit) == 0 {
                return;
            }
            mask |= bit;
        }

        if (mask & center_mask) != 0 {
            solutions.push(word.to_string());
        }
    });

    solutions
}

/// SB-S2: Trie-pruned edge traversal solver.
///
/// Placeholder for Milestone SB-S2. For now in SB-S1, falls back to `solve_filter`.
/// Will be replaced with independent cursor-based edge DFS in the SB-S2 milestone.
pub fn solve_trie_pruned(puzzle: &SpellingBeePuzzle, trie: &Trie) -> Vec<String> {
    // In Milestone SB-S1, temporary delegation until SB-S2 commit
    solve_filter(puzzle, trie)
}
