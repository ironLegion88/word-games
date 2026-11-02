//! Feedback algorithms evaluating guesses against the secret Wordle target.
//!
//! Two retained feedback algorithms are implemented:
//! 1. **WL-F1 (`evaluate_markers`)**: Positional scorer directly adapted from sibling
//!    `mastermind/src/scoring.rs::score_guess`. Uses two matching marker passes (`target_used`
//!    and `guess_used`) with left-to-right partial match resolution. Adapted from dynamic heap vectors
//!    to stack-allocated `[bool; 5]` arrays.
//! 2. **WL-F2 (`evaluate_frequency`)**: Production optimized two-pass scorer utilizing a fixed
//!    26-element remaining target frequency table. Eliminates the inner search loop ($O(L + 26)$).

use crate::word::{Word, WORD_LENGTH};

/// Positional feedback state for a single letter slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum LetterFeedback {
    /// Letter is not present in the remaining target word.
    #[default]
    Absent,
    /// Letter is present in the target word but at a different position.
    Present,
    /// Letter is in the exact correct position.
    Correct,
}

impl LetterFeedback {
    /// Single-character plain text tag representing the feedback state.
    #[inline]
    pub const fn tag(self) -> char {
        match self {
            LetterFeedback::Correct => 'C',
            LetterFeedback::Present => 'P',
            LetterFeedback::Absent => '.',
        }
    }

    /// Readable label for plain text rendering.
    #[inline]
    pub const fn label(self) -> &'static str {
        match self {
            LetterFeedback::Correct => "[CORRECT]",
            LetterFeedback::Present => "[PRESENT]",
            LetterFeedback::Absent => "[ABSENT ]",
        }
    }
}

/// A complete 5-letter feedback array for a submitted guess.
pub type Feedback = [LetterFeedback; WORD_LENGTH];

/// Feedback algorithm strategy selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FeedbackStrategy {
    /// Adapted Mastermind two-pass marker algorithm (WL-F1).
    Marker,
    /// Production 26-letter frequency array algorithm (WL-F2).
    #[default]
    Frequency,
}

/// Evaluates a guess against a target using the adapted Mastermind two-pass marker algorithm (WL-F1).
///
/// # Provenance & Adaptation
/// Directly ported from sibling repository `mastermind/src/scoring.rs::score_guess`:
/// - Pass 1: Mark all exact positional matches as `Correct`. Consume both target and guess occurrences.
/// - Pass 2: For unmatched guess positions from left to right, search for the first unmatched target occurrence.
///   If found, mark as `Present`, consume target and guess occurrences, and break the inner search loop.
/// - Storage adaptation: Replaces `Vec<bool>` heap allocations with fixed-size stack arrays `[bool; 5]`.
/// - Output adaptation: Emits the full positional `[LetterFeedback; 5]` array rather than aggregate counts.
pub fn evaluate_markers(target: &Word, guess: &Word) -> Feedback {
    let mut feedback = [LetterFeedback::Absent; WORD_LENGTH];
    let mut target_used = [false; WORD_LENGTH];
    let mut guess_used = [false; WORD_LENGTH];

    // Pass 1: Exact matches (Bulls -> Correct).
    for i in 0..WORD_LENGTH {
        if target[i] == guess[i] {
            feedback[i] = LetterFeedback::Correct;
            target_used[i] = true;
            guess_used[i] = true;
        }
    }

    // Pass 2: Positional partial matches left to right (Cows -> Present).
    for g in 0..WORD_LENGTH {
        if !guess_used[g] {
            for t in 0..WORD_LENGTH {
                if !target_used[t] && guess[g] == target[t] {
                    feedback[g] = LetterFeedback::Present;
                    target_used[t] = true;
                    guess_used[g] = true;
                    break;
                }
            }
        }
    }

    feedback
}

/// Evaluates a guess against a target using a 26-element remaining target frequency table (WL-F2).
///
/// # Algorithmic Details
/// - Pass 1: Identifies exact matches, setting `LetterFeedback::Correct`. For all non-matching target positions,
///   increments the remaining target frequency table for that letter index `0..26`.
/// - Pass 2: Iterates through guess positions left to right. For any position not already `Correct`,
///   if the remaining frequency for `guess[i]` is $> 0$, assigns `LetterFeedback::Present` and decrements
///   the available count. Otherwise remains `LetterFeedback::Absent`.
/// - Complexity: Exactly two linear scans over the 5 letter positions, with $O(1)$ table lookups.
pub fn evaluate_frequency(target: &Word, guess: &Word) -> Feedback {
    let mut feedback = [LetterFeedback::Absent; WORD_LENGTH];
    let mut remaining_freq = [0u8; 26];

    // Pass 1: Exact matches, count remaining letters in target.
    for i in 0..WORD_LENGTH {
        if target[i] == guess[i] {
            feedback[i] = LetterFeedback::Correct;
        } else {
            remaining_freq[target[i].index()] += 1;
        }
    }

    // Pass 2: Left-to-right assignment of available remaining target occurrences.
    for i in 0..WORD_LENGTH {
        if feedback[i] != LetterFeedback::Correct {
            let idx = guess[i].index();
            if remaining_freq[idx] > 0 {
                feedback[i] = LetterFeedback::Present;
                remaining_freq[idx] -= 1;
            }
        }
    }

    feedback
}

/// Evaluates a guess against a target using the specified feedback strategy.
#[inline]
pub fn evaluate(target: &Word, guess: &Word, strategy: FeedbackStrategy) -> Feedback {
    match strategy {
        FeedbackStrategy::Marker => evaluate_markers(target, guess),
        FeedbackStrategy::Frequency => evaluate_frequency(target, guess),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_all_correct() {
        let target = Word::parse("CRANE").unwrap();
        let guess = Word::parse("CRANE").unwrap();
        let expected = [LetterFeedback::Correct; 5];

        assert_eq!(evaluate_markers(&target, &guess), expected);
        assert_eq!(evaluate_frequency(&target, &guess), expected);
    }

    #[test]
    fn test_all_absent() {
        let target = Word::parse("CRANE").unwrap();
        let guess = Word::parse("TOILS").unwrap();
        let expected = [LetterFeedback::Absent; 5];

        assert_eq!(evaluate_markers(&target, &guess), expected);
        assert_eq!(evaluate_frequency(&target, &guess), expected);
    }

    #[test]
    fn test_duplicate_trap_apple_alley() {
        // Target APPLE, Guess ALLEY
        // Pos 0: A == A -> Correct
        // Pos 1: L != P. Target has one L at pos 3 -> Present
        // Pos 2: L != P. Target L is already consumed -> Absent
        // Pos 3: E != L. Target has one E at pos 4 -> Present
        // Pos 4: Y != E. Y not in APPLE -> Absent
        let target = Word::parse("APPLE").unwrap();
        let guess = Word::parse("ALLEY").unwrap();
        let expected = [
            LetterFeedback::Correct,
            LetterFeedback::Present,
            LetterFeedback::Absent,
            LetterFeedback::Present,
            LetterFeedback::Absent,
        ];

        assert_eq!(evaluate_markers(&target, &guess), expected);
        assert_eq!(evaluate_frequency(&target, &guess), expected);
    }

    #[test]
    fn test_duplicate_trap_level_eerie() {
        // Target LEVEL, Guess EERIE
        // Target letters: L:2, E:2, V:1
        // Guess: E, E, R, I, E
        // Pass 1:
        // Pos 1: E == E -> Correct. Consumes one E from target. (Remaining E in target: 1).
        // Pass 2:
        // Pos 0: E. Matches remaining target E -> Present. (Remaining E: 0).
        // Pos 2: R -> Absent.
        // Pos 3: I -> Absent.
        // Pos 4: E. No remaining E in target -> Absent.
        let target = Word::parse("LEVEL").unwrap();
        let guess = Word::parse("EERIE").unwrap();
        let expected = [
            LetterFeedback::Present,
            LetterFeedback::Correct,
            LetterFeedback::Absent,
            LetterFeedback::Absent,
            LetterFeedback::Absent,
        ];

        assert_eq!(evaluate_markers(&target, &guess), expected);
        assert_eq!(evaluate_frequency(&target, &guess), expected);
    }

    #[test]
    fn test_excess_guess_duplicates_after_exact() {
        // Target: WATER (W, A, T, E, R)
        // Guess:  EERIE (E, E, R, I, E)
        // Target has one E. Guess has three Es.
        // Guess pos 3 is I. Guess pos 2 is R. Target pos 4 is R.
        // Target has E at pos 3. Guess has E at pos 0, 1, 4. None at pos 3.
        // So pos 0 gets Present (consumes target E).
        // Pos 1 gets Absent. Pos 4 gets Absent.
        let target = Word::parse("WATER").unwrap();
        let guess = Word::parse("EERIE").unwrap();
        let expected = [
            LetterFeedback::Present,
            LetterFeedback::Absent,
            LetterFeedback::Present,
            LetterFeedback::Absent,
            LetterFeedback::Absent,
        ];

        assert_eq!(evaluate_markers(&target, &guess), expected);
        assert_eq!(evaluate_frequency(&target, &guess), expected);
    }
}
