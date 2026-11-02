//! Interactive Boggle game session and state management.
//!
//! Enforces:
//! - Setup temporarily borrows `&Trie` to generate the complete sorted solutions snapshot,
//!   then owns the snapshot with no persistent trie lifetime or borrow.
//! - Maintained invariant: `cached_score == Σ score(accepted_guesses)`.
//! - Evaluated guesses cannot be retracted.
//! - Duplicate, invalid, or off-board guesses score zero points.
//! - Honest `NotASolution` feedback when a word is absent from the snapshot.
//! - Immutability after game completion or `/quit`.

use std::collections::BTreeSet;
use trie::Trie;

use crate::board::Board;
use crate::error::{BoggleError, SessionError};
use crate::path::MIN_WORD_LENGTH;
use crate::scoring::{calculate_total_score, word_score};
use crate::solver::{solve, SolverStrategy};

/// Result of evaluating a player's guess in an active session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuessOutcome {
    /// Valid new solution word accepted.
    Accepted {
        word: String,
        points: u32,
        total_score: u32,
        is_complete: bool,
    },
    /// Word was already guessed earlier in this session.
    Duplicate { word: String },
    /// Word is shorter than the minimum required length (3).
    TooShort {
        word: String,
        min_length: usize,
        actual_length: usize,
    },
    /// Word contains invalid characters (non-ASCII).
    InvalidWord { word: String, reason: String },
    /// Word is not among the solutions for this board.
    NotASolution { word: String },
    /// Submission was rejected because the session has finished or was quit.
    GameFinished,
}

/// Summary returned when a session terminates via quit or completion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuitSummary {
    /// Player's final achieved score.
    pub score: u32,
    /// Total possible score achievable on this board.
    pub total_possible_score: u32,
    /// Number of distinct valid words found by the player.
    pub found_count: usize,
    /// Total number of valid words on this board.
    pub total_solutions: usize,
    /// Sorted list of valid words found by the player.
    pub found_words: Vec<String>,
    /// Sorted list of valid words missed by the player.
    pub missed_words: Vec<String>,
}

/// An owned Boggle game session.
#[derive(Clone, Debug)]
pub struct BoggleSession {
    board: Board,
    solutions: Vec<String>,
    accepted_guesses: BTreeSet<String>,
    cached_score: u32,
    is_finished: bool,
    solver_strategy: SolverStrategy,
}

impl BoggleSession {
    /// Creates a new `BoggleSession` from a board and precomputed sorted solution set.
    ///
    /// Validates that `solutions` is strictly sorted and contains no duplicates.
    pub fn new(
        board: Board,
        solutions: Vec<String>,
        solver_strategy: SolverStrategy,
    ) -> Result<Self, SessionError> {
        // Validate strictly ascending order and no duplicates
        for window in solutions.windows(2) {
            if window[0] == window[1] {
                return Err(SessionError::DuplicateWordsInSolutions);
            }
            if window[0] > window[1] {
                return Err(SessionError::UnsortedSolutions);
            }
        }

        let session = Self {
            board,
            solutions,
            accepted_guesses: BTreeSet::new(),
            cached_score: 0,
            is_finished: false,
            solver_strategy,
        };

        session.assert_invariants();
        Ok(session)
    }

    /// Sets up a new `BoggleSession` by temporarily borrowing `&Trie` to generate
    /// the solution snapshot using the chosen solver strategy.
    ///
    /// The resulting session owns its board and solution snapshot; it does not
    /// retain any borrow or reference to the trie.
    pub fn from_trie(
        board: Board,
        trie: &Trie,
        strategy: SolverStrategy,
    ) -> Result<Self, BoggleError> {
        let solutions = solve(&board, trie, strategy)?;
        let session = Self::new(board, solutions, strategy)?;
        Ok(session)
    }

    /// Returns a reference to the board.
    #[inline]
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Returns a slice of all precomputed solution words.
    #[inline]
    pub fn solutions(&self) -> &[String] {
        &self.solutions
    }

    /// Returns the number of total solutions for this board.
    #[inline]
    pub fn solution_count(&self) -> usize {
        self.solutions.len()
    }

    /// Returns the set of currently accepted unique guesses.
    #[inline]
    pub fn accepted_guesses(&self) -> &BTreeSet<String> {
        &self.accepted_guesses
    }

    /// Returns the cached running score.
    #[inline]
    pub fn cached_score(&self) -> u32 {
        self.cached_score
    }

    /// Returns `true` if the session has concluded.
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.is_finished
    }

    /// Returns the solver strategy used during session construction.
    #[inline]
    pub fn solver_strategy(&self) -> SolverStrategy {
        self.solver_strategy
    }

    /// Recalculates the total score from all accepted guesses from scratch.
    pub fn recalculate_score(&self) -> u32 {
        calculate_total_score(self.accepted_guesses.iter().map(|s| s.as_str())).unwrap_or(0)
    }

    /// Asserts internal state invariants:
    /// 1. `cached_score == Σ word_score(accepted_guesses)`.
    /// 2. All accepted guesses belong to the solution snapshot.
    pub fn assert_invariants(&self) {
        let expected = self.recalculate_score();
        assert_eq!(
            self.cached_score, expected,
            "invariant violation: cached_score ({}) != recalculate_score ({})",
            self.cached_score, expected
        );
        for guess in &self.accepted_guesses {
            assert!(
                self.solutions.binary_search(guess).is_ok(),
                "invariant violation: accepted guess '{guess}' not in solutions"
            );
        }
    }

    /// Submits a guess word to the session.
    ///
    /// Validates the guess, updates score and accepted guesses if valid and new,
    /// and maintains state invariants.
    pub fn submit_guess(&mut self, input: &str) -> GuessOutcome {
        if self.is_finished {
            return GuessOutcome::GameFinished;
        }

        let normalized = match trie::normalize_word(input) {
            Ok(norm) => norm.into_owned(),
            Err(e) => {
                return GuessOutcome::InvalidWord {
                    word: input.to_string(),
                    reason: e.to_string(),
                }
            }
        };

        if normalized.len() < MIN_WORD_LENGTH {
            return GuessOutcome::TooShort {
                word: normalized.clone(),
                min_length: MIN_WORD_LENGTH,
                actual_length: normalized.len(),
            };
        }

        if self.accepted_guesses.contains(&normalized) {
            return GuessOutcome::Duplicate { word: normalized };
        }

        if self.solutions.binary_search(&normalized).is_err() {
            return GuessOutcome::NotASolution { word: normalized };
        }

        let points = word_score(&normalized).unwrap_or(0);
        self.cached_score = self
            .cached_score
            .checked_add(points)
            .expect("score overflow");
        self.accepted_guesses.insert(normalized.clone());

        let is_complete =
            !self.solutions.is_empty() && self.accepted_guesses.len() == self.solutions.len();
        if is_complete {
            self.is_finished = true;
        }

        self.assert_invariants();

        GuessOutcome::Accepted {
            word: normalized,
            points,
            total_score: self.cached_score,
            is_complete,
        }
    }

    /// Gracefully terminates the session and returns a summary of results.
    ///
    /// After calling `quit`, further submissions will return `GuessOutcome::GameFinished`.
    pub fn quit(&mut self) -> QuitSummary {
        self.is_finished = true;
        self.assert_invariants();

        let total_possible_score =
            calculate_total_score(self.solutions.iter().map(|s| s.as_str())).unwrap_or(0);

        let mut missed = Vec::new();
        for s in &self.solutions {
            if !self.accepted_guesses.contains(s) {
                missed.push(s.clone());
            }
        }

        let found = self.accepted_guesses.iter().cloned().collect();

        QuitSummary {
            score: self.cached_score,
            total_possible_score,
            found_count: self.accepted_guesses.len(),
            total_solutions: self.solutions.len(),
            found_words: found,
            missed_words: missed,
        }
    }
}
