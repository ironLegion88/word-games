//! Hint generation and formatted rendering for Spelling Bee.
//!
//! Provides:
//! - Initial-letter × word-length hint grid with marginal row/column sums.
//! - Two-letter prefix count breakdown.

use crate::puzzle::SpellingBeePuzzle;
use crate::session::calculate_word_score;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// An initial-letter by word-length hint matrix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HintGrid {
    /// Distinct starting letters present in the solution set, sorted alphabetically.
    pub row_letters: Vec<char>,
    /// Distinct lengths spanning from min to max length present in solutions.
    pub lengths: Vec<usize>,
    /// Word counts indexed by `(first_letter, length)`.
    pub counts: BTreeMap<(char, usize), usize>,
    /// Total words starting with each letter.
    pub row_totals: BTreeMap<char, usize>,
    /// Total words for each length.
    pub col_totals: BTreeMap<usize, usize>,
    /// Total word count across all solutions.
    pub grand_total_words: usize,
    /// Total score potential of the puzzle.
    pub total_points: u32,
    /// Total number of pangrams in the puzzle.
    pub total_pangrams: usize,
}

impl HintGrid {
    /// Computes the hint grid directly from the puzzle's frozen solution set.
    pub fn from_solutions(puzzle: &SpellingBeePuzzle, solutions: &[String]) -> Self {
        let mut row_letters_set = BTreeSet::new();
        let mut min_len = usize::MAX;
        let mut max_len = 0;
        let mut counts = BTreeMap::new();
        let mut row_totals = BTreeMap::new();
        let mut col_totals = BTreeMap::new();
        let mut total_points = 0u32;
        let mut total_pangrams = 0usize;

        for word in solutions {
            let first_char = word.chars().next().unwrap_or('A');
            let len = word.len();

            row_letters_set.insert(first_char);
            if len < min_len {
                min_len = len;
            }
            if len > max_len {
                max_len = len;
            }

            *counts.entry((first_char, len)).or_insert(0) += 1;
            *row_totals.entry(first_char).or_insert(0) += 1;
            *col_totals.entry(len).or_insert(0) += 1;

            let is_pangram = puzzle.is_pangram(word);
            if is_pangram {
                total_pangrams += 1;
            }
            total_points += calculate_word_score(word, is_pangram);
        }

        let lengths: Vec<usize> = if min_len <= max_len {
            (min_len..=max_len).collect()
        } else {
            Vec::new()
        };

        let row_letters: Vec<char> = row_letters_set.into_iter().collect();

        Self {
            row_letters,
            lengths,
            counts,
            row_totals,
            col_totals,
            grand_total_words: solutions.len(),
            total_points,
            total_pangrams,
        }
    }

    /// Formats the hint grid into an aligned plain text table matching the PDF specification.
    pub fn format_table(&self) -> String {
        if self.grand_total_words == 0 {
            return "No words in solution set.\n".to_string();
        }

        let mut out = String::new();
        out.push_str(&format!(
            "WORDS: {}, POINTS: {}, PANGRAMS: {}\n\n",
            self.grand_total_words, self.total_points, self.total_pangrams
        ));

        // Header row
        out.push_str("    ");
        for &len in &self.lengths {
            out.push_str(&format!("{len:>4}"));
        }
        out.push_str("   Σ\n");

        // Data rows
        for &letter in &self.row_letters {
            out.push_str(&format!("{letter}:  "));
            for &len in &self.lengths {
                match self.counts.get(&(letter, len)) {
                    Some(&cnt) if cnt > 0 => out.push_str(&format!("{cnt:>4}")),
                    _ => out.push_str("   -"),
                }
            }
            let row_sum = self.row_totals.get(&letter).copied().unwrap_or(0);
            out.push_str(&format!("{row_sum:>4}\n"));
        }

        // Summary row
        out.push_str("Σ:  ");
        for &len in &self.lengths {
            let col_sum = self.col_totals.get(&len).copied().unwrap_or(0);
            out.push_str(&format!("{col_sum:>4}"));
        }
        out.push_str(&format!("{:>4}\n", self.grand_total_words));

        out
    }
}

impl fmt::Display for HintGrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_table())
    }
}

/// Two-letter prefix word counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TwoLetterCounts {
    /// Map of two-letter prefix (uppercase) to word count.
    pub counts: BTreeMap<String, usize>,
}

impl TwoLetterCounts {
    /// Derives two-letter prefix counts from the solution set.
    pub fn from_solutions(solutions: &[String]) -> Self {
        let mut counts = BTreeMap::new();
        for word in solutions {
            if word.len() >= 2 {
                let prefix: String = word.chars().take(2).collect();
                *counts.entry(prefix).or_insert(0) += 1;
            }
        }
        Self { counts }
    }

    /// Formats the two-letter list grouped by starting letter matching the PDF specification.
    pub fn format_list(&self) -> String {
        if self.counts.is_empty() {
            return "No two-letter prefixes available.\n".to_string();
        }

        let mut out = String::new();
        let mut current_first = '\0';
        let mut line_items = Vec::new();

        for (prefix, count) in &self.counts {
            let first = prefix.chars().next().unwrap_or('A');
            if first != current_first {
                if !line_items.is_empty() {
                    out.push_str(&line_items.join(" "));
                    out.push('\n');
                    line_items.clear();
                }
                current_first = first;
            }
            line_items.push(format!("{prefix}-{count}"));
        }

        if !line_items.is_empty() {
            out.push_str(&line_items.join(" "));
            out.push('\n');
        }

        out
    }
}

impl fmt::Display for TwoLetterCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_list())
    }
}
