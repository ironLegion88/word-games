//! Scoring rules and checked arithmetic for Boggle words.
//!
//! Scoring schedule:
//! - 3–4 letters: 1 point
//! - 5 letters: 2 points
//! - 6 letters: 3 points
//! - 7 letters: 5 points
//! - 8 letters: 11 points
//! - 9–12 letters: 2 × length points
//! - 13–14 letters: 3 × length points
//! - ≥15 letters: 4 × length points for every length (custom dictionary support)
//!
//! Checked arithmetic is used throughout to avoid overflow.

use crate::error::ScoringError;

/// Calculates the score for a single word according to the extended Boggle schedule.
///
/// Returns `Ok(0)` for words shorter than 3 letters.
/// Uses checked multiplication and returns `Err(ScoringError::ArithmeticOverflow)`
/// if arithmetic limits are exceeded.
pub fn word_score(word: &str) -> Result<u32, ScoringError> {
    let len = word.len();
    if len < 3 {
        return Ok(0);
    }

    match len {
        3 | 4 => Ok(1),
        5 => Ok(2),
        6 => Ok(3),
        7 => Ok(5),
        8 => Ok(11),
        9..=12 => (len as u32)
            .checked_mul(2)
            .ok_or(ScoringError::ArithmeticOverflow),
        13..=14 => (len as u32)
            .checked_mul(3)
            .ok_or(ScoringError::ArithmeticOverflow),
        _ => (len as u32)
            .checked_mul(4)
            .ok_or(ScoringError::ArithmeticOverflow),
    }
}

/// Computes the total score for an iterator of words using checked arithmetic.
pub fn calculate_total_score<'a, I>(words: I) -> Result<u32, ScoringError>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut total = 0u32;
    for w in words {
        let pts = word_score(w)?;
        total = total
            .checked_add(pts)
            .ok_or(ScoringError::ArithmeticOverflow)?;
    }
    Ok(total)
}
