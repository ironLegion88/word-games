//! Monotonic letter-status keyboard tracking.

use trie::letter::{Letter, ALPHABET_SIZE};

use crate::feedback::LetterFeedback;

/// Monotonic status of an individual letter on the keyboard.
///
/// Precedence order: `Correct > Present > Absent > Unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum KeyStatus {
    /// Letter has not yet been guessed.
    Unknown = 0,
    /// Letter was guessed and confirmed absent from all positions.
    Absent = 1,
    /// Letter was guessed and confirmed present elsewhere in the target word.
    Present = 2,
    /// Letter was guessed in an exact position.
    Correct = 3,
}

impl KeyStatus {
    /// Plain text single-character tag for this status.
    #[inline]
    pub const fn tag(self) -> char {
        match self {
            KeyStatus::Unknown => ' ',
            KeyStatus::Absent => '.',
            KeyStatus::Present => 'P',
            KeyStatus::Correct => 'C',
        }
    }
}

/// Keyboard tracking the monotonic discovery status of all 26 uppercase English letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Keyboard {
    statuses: [KeyStatus; ALPHABET_SIZE],
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Keyboard {
    /// Creates a fresh keyboard with all 26 letters marked as `Unknown`.
    pub const fn new() -> Self {
        Self {
            statuses: [KeyStatus::Unknown; ALPHABET_SIZE],
        }
    }

    /// Resets all letters to `Unknown`.
    pub fn reset(&mut self) {
        self.statuses = [KeyStatus::Unknown; ALPHABET_SIZE];
    }

    /// Returns the current discovery status of a letter.
    #[inline]
    pub fn status(&self, letter: Letter) -> KeyStatus {
        self.statuses[letter.index()]
    }

    /// Updates the status of a letter with new feedback, enforcing monotonicity.
    ///
    /// Monotonicity guarantee:
    /// `Correct (3) > Present (2) > Absent (1) > Unknown (0)`.
    /// A weaker evidence observation (e.g. an Absent duplicate mark) will NEVER
    /// overwrite or downgrade an already established Present or Correct status.
    pub fn observe(&mut self, letter: Letter, mark: LetterFeedback) {
        let new_status = match mark {
            LetterFeedback::Correct => KeyStatus::Correct,
            LetterFeedback::Present => KeyStatus::Present,
            LetterFeedback::Absent => KeyStatus::Absent,
        };

        let current = self.statuses[letter.index()];
        if new_status > current {
            self.statuses[letter.index()] = new_status;
        }
    }

    /// Standard QWERTY keyboard letter layout across 3 rows.
    pub const QWERTY_ROWS: [&'static str; 3] = ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];

    /// Formats the keyboard into 3 lines of plain text with letter tags.
    pub fn format_plain(&self) -> [String; 3] {
        let mut lines = [String::new(), String::new(), String::new()];
        for (r, &row) in Self::QWERTY_ROWS.iter().enumerate() {
            let mut line = String::new();
            if r == 1 {
                line.push(' ');
            } else if r == 2 {
                line.push_str("   ");
            }
            for ch in row.chars() {
                let letter = Letter::from_char(ch).unwrap();
                let status = self.status(letter);
                match status {
                    KeyStatus::Unknown => line.push_str(&format!("{ch}  ")),
                    KeyStatus::Absent => line.push_str(&format!("{ch}. ")),
                    KeyStatus::Present => line.push_str(&format!("{ch}? ")),
                    KeyStatus::Correct => line.push_str(&format!("{ch}! ")),
                }
            }
            lines[r] = line.trim_end().to_string();
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monotonicity_absent_does_not_downgrade_correct() {
        let mut kb = Keyboard::new();
        let letter_a = Letter::from_char('A').unwrap();

        assert_eq!(kb.status(letter_a), KeyStatus::Unknown);

        // Mark Correct
        kb.observe(letter_a, LetterFeedback::Correct);
        assert_eq!(kb.status(letter_a), KeyStatus::Correct);

        // Mark Absent (e.g. duplicate guess)
        kb.observe(letter_a, LetterFeedback::Absent);
        assert_eq!(
            kb.status(letter_a),
            KeyStatus::Correct,
            "Absent must not downgrade Correct"
        );

        // Mark Present
        kb.observe(letter_a, LetterFeedback::Present);
        assert_eq!(
            kb.status(letter_a),
            KeyStatus::Correct,
            "Present must not downgrade Correct"
        );
    }

    #[test]
    fn test_monotonicity_upgrade_sequence() {
        let mut kb = Keyboard::new();
        let letter_e = Letter::from_char('E').unwrap();

        assert_eq!(kb.status(letter_e), KeyStatus::Unknown);

        kb.observe(letter_e, LetterFeedback::Absent);
        assert_eq!(kb.status(letter_e), KeyStatus::Absent);

        kb.observe(letter_e, LetterFeedback::Present);
        assert_eq!(kb.status(letter_e), KeyStatus::Present);

        kb.observe(letter_e, LetterFeedback::Correct);
        assert_eq!(kb.status(letter_e), KeyStatus::Correct);
    }
}
