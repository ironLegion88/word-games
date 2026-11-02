//! Session management, guess processing, and scoring invariants for Spelling Bee.

use crate::error::GuessError;
use crate::hints::{HintGrid, TwoLetterCounts};
use crate::puzzle::SpellingBeePuzzle;
use std::collections::BTreeSet;

/// Bonus points awarded for a pangram (using all 7 letters).
pub const PANGRAM_BONUS_POINTS: u32 = 7;

/// Computes the point value for a valid word.
///
/// Rules:
/// - 4 letters: 1 point
/// - 5 or more letters: 1 point per letter
/// - Pangram bonus: +7 points
#[inline]
pub fn calculate_word_score(word: &str, is_pangram: bool) -> u32 {
    let len = word.len();
    if len < 4 {
        0
    } else {
        let base = if len == 4 { 1 } else { len as u32 };
        if is_pangram {
            base + PANGRAM_BONUS_POINTS
        } else {
            base
        }
    }
}

/// The typed result of submitting a guess to the session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuessOutcome {
    /// Guess is valid, newly discovered, and scored.
    Accepted {
        word: String,
        points: u32,
        is_pangram: bool,
        queen_bee: bool,
    },
    /// Word was already guessed earlier in this session.
    AlreadyGuessed { word: String },
    /// Word violates spelling rules or is not in the dictionary.
    InvalidWord(GuessError),
    /// Round has finished (Queen Bee or Quit); no further guesses accepted.
    GameFinished,
}

/// Summary report returned when early quitting the game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuitSummary {
    /// Player's final score.
    pub score: u32,
    /// Words player found.
    pub found_count: usize,
    /// Total possible words.
    pub total_solutions: usize,
    /// Sorted list of solution words the player missed.
    pub missed_words: Vec<String>,
}

/// An active or completed Spelling Bee game session.
///
/// Owns its validated puzzle and frozen solution vector.
/// Maintains accepted guesses, cached score, and explicit terminal state.
#[derive(Clone, Debug)]
pub struct SpellingBeeSession {
    puzzle: SpellingBeePuzzle,
    solutions: Vec<String>,
    accepted_guesses: BTreeSet<String>,
    cached_score: u32,
    is_terminated: bool,
    is_quit: bool,
}

impl SpellingBeeSession {
    /// Creates a new session from a puzzle and an owned solution set.
    ///
    /// Solutions must be sorted and deduplicated.
    pub fn new(puzzle: SpellingBeePuzzle, mut solutions: Vec<String>) -> Self {
        solutions.sort_unstable();
        solutions.dedup();

        Self {
            puzzle,
            solutions,
            accepted_guesses: BTreeSet::new(),
            cached_score: 0,
            is_terminated: false,
            is_quit: false,
        }
    }

    /// Returns the underlying puzzle.
    #[inline]
    pub fn puzzle(&self) -> &SpellingBeePuzzle {
        &self.puzzle
    }

    /// Returns the complete list of frozen solutions.
    #[inline]
    pub fn solutions(&self) -> &[String] {
        &self.solutions
    }

    /// Returns the sorted set of accepted guesses.
    #[inline]
    pub fn accepted_guesses(&self) -> &BTreeSet<String> {
        &self.accepted_guesses
    }

    /// Returns the player's current cumulative score.
    #[inline]
    pub fn current_score(&self) -> u32 {
        self.cached_score
    }

    /// Returns `true` if the session is terminated (either via Queen Bee or quit).
    #[inline]
    pub fn is_terminated(&self) -> bool {
        self.is_terminated
    }

    /// Returns `true` if all solutions have been found (Queen Bee accolade earned).
    #[inline]
    pub fn is_queen_bee(&self) -> bool {
        !self.solutions.is_empty() && self.accepted_guesses.len() == self.solutions.len()
    }

    /// Returns the total points possible for this puzzle.
    pub fn total_possible_score(&self) -> u32 {
        self.solutions
            .iter()
            .map(|w| calculate_word_score(w, self.puzzle.is_pangram(w)))
            .sum()
    }

    /// Returns the total number of pangrams in this puzzle.
    pub fn total_pangrams(&self) -> usize {
        self.solutions
            .iter()
            .filter(|w| self.puzzle.is_pangram(w))
            .count()
    }

    /// Evaluates a submitted guess, updating state and cached score on acceptance.
    pub fn submit_guess(&mut self, raw_guess: &str) -> GuessOutcome {
        if self.is_terminated {
            return GuessOutcome::GameFinished;
        }

        let trimmed = raw_guess.trim();
        if trimmed.is_empty() {
            return GuessOutcome::InvalidWord(GuessError::TooShort {
                min_length: 4,
                actual_length: 0,
            });
        }

        // 1. Syntactic candidate validation (length, characters, center, allowed letters)
        if let Err(err) = self.puzzle.validate_candidate(trimmed) {
            return GuessOutcome::InvalidWord(err);
        }

        let normalized: String = trimmed.chars().map(|c| c.to_ascii_uppercase()).collect();

        // 2. Duplicate check
        if self.accepted_guesses.contains(&normalized) {
            return GuessOutcome::AlreadyGuessed { word: normalized };
        }

        // 3. Solution membership check
        if self.solutions.binary_search(&normalized).is_err() {
            return GuessOutcome::InvalidWord(GuessError::NotInDictionary);
        }

        // 4. Scoring and state mutation
        let is_pangram = self.puzzle.is_pangram(&normalized);
        let points = calculate_word_score(&normalized, is_pangram);

        self.accepted_guesses.insert(normalized.clone());
        self.cached_score += points;

        let queen_bee = self.is_queen_bee();
        if queen_bee {
            self.is_terminated = true;
        }

        debug_assert_eq!(
            self.cached_score,
            self.recalculate_score(),
            "cached_score invariant violated"
        );

        GuessOutcome::Accepted {
            word: normalized,
            points,
            is_pangram,
            queen_bee,
        }
    }

    /// Forfeits the game, revealing missed words and final score.
    pub fn quit(&mut self) -> QuitSummary {
        self.is_terminated = true;
        self.is_quit = true;

        let missed_words: Vec<String> = self
            .solutions
            .iter()
            .filter(|w| !self.accepted_guesses.contains(*w))
            .cloned()
            .collect();

        QuitSummary {
            score: self.cached_score,
            found_count: self.accepted_guesses.len(),
            total_solutions: self.solutions.len(),
            missed_words,
        }
    }

    /// Recomputes the total score from all accepted guesses.
    pub fn recalculate_score(&self) -> u32 {
        self.accepted_guesses
            .iter()
            .map(|w| calculate_word_score(w, self.puzzle.is_pangram(w)))
            .sum()
    }

    /// Verifies session state invariants.
    pub fn check_invariants(&self) -> Result<(), String> {
        let expected_score = self.recalculate_score();
        if self.cached_score != expected_score {
            return Err(format!(
                "score mismatch: cached {}, computed {}",
                self.cached_score, expected_score
            ));
        }

        for word in &self.accepted_guesses {
            if self.solutions.binary_search(word).is_err() {
                return Err(format!("accepted guess '{word}' not in solutions"));
            }
        }

        let is_qb =
            !self.solutions.is_empty() && self.accepted_guesses.len() == self.solutions.len();

        if self.is_terminated && !self.is_quit && !is_qb {
            return Err("session marked terminated without quit or Queen Bee".into());
        }

        Ok(())
    }

    /// Computes the initial-letter by length hint grid.
    pub fn hint_grid(&self) -> HintGrid {
        HintGrid::from_solutions(&self.puzzle, &self.solutions)
    }

    /// Computes two-letter prefix counts.
    pub fn two_letter_counts(&self) -> TwoLetterCounts {
        TwoLetterCounts::from_solutions(&self.solutions)
    }

    /// Formats accepted guesses alphabetically in a 3-column layout.
    pub fn format_accepted_guesses_three_columns(&self) -> String {
        format_three_columns(self.accepted_guesses.iter().map(|s| s.as_str()))
    }
}

/// Formats a list of words in alphabetical order into three evenly-spaced columns.
pub fn format_three_columns<'a, I>(words: I) -> String
where
    I: IntoIterator<Item = &'a str>,
{
    let list: Vec<&'a str> = words.into_iter().collect();
    if list.is_empty() {
        return "No words guessed yet.\n".to_string();
    }

    let col_width = list.iter().map(|w| w.len()).max().unwrap_or(8).max(12) + 2;
    let total_words = list.len();
    let num_rows = total_words.div_ceil(3);

    let mut out = String::new();
    for row in 0..num_rows {
        for col in 0..3 {
            let idx = col * num_rows + row;
            if idx < total_words {
                let word = list[idx];
                out.push_str(&format!("{word:<width$}", width = col_width));
            }
        }
        out.push('\n');
    }
    out
}
