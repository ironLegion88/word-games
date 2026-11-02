//! Solution enumeration algorithms for Spelling Bee.
//!
//! Provides two distinct algorithms:
//! - `SB-S1`: Full dictionary filtering baseline (visits every dictionary word and checks letter masks).
//! - `SB-S2`: Trie-pruned edge traversal (follows only the 7 allowed letter branches).

use crate::puzzle::{SpellingBeePuzzle, MIN_WORD_LENGTH};
use trie::{Letter, Trie, TrieCursor};

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
/// Starting from the root of the trie, performs a depth-first search exploring
/// ONLY the 7 allowed letter branches. When a terminal node representing a word of
/// length >= 4 is reached, verifies that the central letter was included along the
/// path before recording the word.
///
/// This explores only O(7^L) branches bounded by dictionary prefixes, dramatically
/// outperforming whole-dictionary filtering without requiring any dictionary mutations.
pub fn solve_trie_pruned(puzzle: &SpellingBeePuzzle, trie: &Trie) -> Vec<String> {
    let mut solutions = Vec::new();
    let root_cursor = trie.cursor();

    // Collect the 7 distinct allowed letters sorted alphabetically for deterministic DFS
    let mut allowed_letters = puzzle.letters();
    allowed_letters.sort_unstable_by_key(|l| l.to_char());

    let center_mask = puzzle.center_mask();
    let mut current_word = String::with_capacity(32);

    dfs_pruned(
        root_cursor,
        &allowed_letters,
        center_mask,
        0,
        &mut current_word,
        &mut solutions,
    );

    solutions.sort_unstable();
    solutions.dedup();
    solutions
}

fn dfs_pruned(
    cursor: TrieCursor<'_>,
    allowed_letters: &[Letter; 7],
    center_mask: u32,
    path_mask: u32,
    current_word: &mut String,
    solutions: &mut Vec<String>,
) {
    if current_word.len() >= MIN_WORD_LENGTH && cursor.is_word() && (path_mask & center_mask) != 0 {
        solutions.push(current_word.clone());
    }

    if !cursor.has_children() {
        return;
    }

    for &letter in allowed_letters {
        if let Some(next_cursor) = cursor.advance(letter) {
            current_word.push(letter.to_char());
            dfs_pruned(
                next_cursor,
                allowed_letters,
                center_mask,
                path_mask | (1 << letter.index()),
                current_word,
                solutions,
            );
            current_word.pop();
        }
    }
}
