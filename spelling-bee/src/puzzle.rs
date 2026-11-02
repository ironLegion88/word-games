//! Immutable puzzle model for Spelling Bee.

use crate::error::{GuessError, PuzzleError};
use std::fmt;
use trie::Letter;

/// Minimum allowed word length in Spelling Bee.
pub const MIN_WORD_LENGTH: usize = 4;
/// Number of distinct letters in a Spelling Bee puzzle.
pub const PUZZLE_LETTER_COUNT: usize = 7;

/// An immutable, validated Spelling Bee puzzle.
///
/// Contains exactly seven distinct ASCII letters, one of which is designated as
/// the central letter. The letters are normalized to uppercase.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpellingBeePuzzle {
    /// Designated central letter.
    center: Letter,
    /// The other six distinct outer letters (kept sorted for canonical representation).
    outer_letters: [Letter; 6],
    /// 26-bit bitmask where bit `i` is set if letter `i` is in the 7 allowed letters.
    allowed_mask: u32,
    /// 26-bit bitmask with only the center letter bit set.
    center_mask: u32,
}

impl SpellingBeePuzzle {
    /// Creates a puzzle from a central letter and six outer letters.
    ///
    /// Validates that all seven letters are ASCII alphabetic and mutually distinct.
    pub fn new(center_char: char, outer_chars: &[char]) -> Result<Self, PuzzleError> {
        let center = Letter::from_char(center_char)
            .ok_or(PuzzleError::InvalidCharacter { ch: center_char })?;

        if outer_chars.len() != 6 {
            let mut all = String::with_capacity(7);
            all.push(center_char);
            all.extend(outer_chars.iter());
            return Err(PuzzleError::InvalidDistinctCount {
                expected: PUZZLE_LETTER_COUNT,
                actual: outer_chars.len() + 1,
                letters: all,
            });
        }

        let mut outers = [Letter::from_char('A').unwrap(); 6];
        let mut seen_mask = 1u32 << center.index();

        for (i, &ch) in outer_chars.iter().enumerate() {
            let letter = Letter::from_char(ch).ok_or(PuzzleError::InvalidCharacter { ch })?;
            let bit = 1u32 << letter.index();
            if (seen_mask & bit) != 0 {
                let mut all = String::with_capacity(7);
                all.push(center.to_char());
                for &c in outer_chars {
                    all.push(c.to_ascii_uppercase());
                }
                return Err(PuzzleError::InvalidDistinctCount {
                    expected: PUZZLE_LETTER_COUNT,
                    actual: seen_mask.count_ones() as usize,
                    letters: all,
                });
            }
            seen_mask |= bit;
            outers[i] = letter;
        }

        outers.sort_unstable_by_key(|l| l.to_char());

        Ok(Self {
            center,
            outer_letters: outers,
            allowed_mask: seen_mask,
            center_mask: 1u32 << center.index(),
        })
    }

    /// Creates a puzzle from a 7-letter string where the first character is designated as the center.
    pub fn from_raw(letters_center_first: &str) -> Result<Self, PuzzleError> {
        if letters_center_first.is_empty() {
            return Err(PuzzleError::EmptyInput);
        }

        let chars: Vec<char> = letters_center_first.chars().collect();
        if chars.len() != PUZZLE_LETTER_COUNT {
            return Err(PuzzleError::InvalidDistinctCount {
                expected: PUZZLE_LETTER_COUNT,
                actual: chars.len(),
                letters: letters_center_first.to_string(),
            });
        }

        Self::new(chars[0], &chars[1..])
    }

    /// Creates a puzzle from a string of 7 letters and an explicit center letter.
    pub fn from_letters_and_center(
        letters_str: &str,
        center_char: char,
    ) -> Result<Self, PuzzleError> {
        let center = Letter::from_char(center_char)
            .ok_or(PuzzleError::InvalidCharacter { ch: center_char })?;

        let chars: Vec<char> = letters_str.chars().collect();
        let mut outer_chars = Vec::with_capacity(6);
        let mut found_center = false;

        for &c in &chars {
            let letter = Letter::from_char(c).ok_or(PuzzleError::InvalidCharacter { ch: c })?;
            if letter == center {
                if found_center {
                    return Err(PuzzleError::InvalidDistinctCount {
                        expected: PUZZLE_LETTER_COUNT,
                        actual: chars.len().saturating_sub(1),
                        letters: letters_str.to_string(),
                    });
                }
                found_center = true;
            } else {
                outer_chars.push(c);
            }
        }

        if !found_center {
            return Err(PuzzleError::CenterNotInLetters {
                center: center_char.to_ascii_uppercase(),
                letters: letters_str.to_ascii_uppercase(),
            });
        }

        Self::new(center.to_char(), &outer_chars)
    }

    /// Returns the central letter.
    #[inline]
    pub fn center(&self) -> Letter {
        self.center
    }

    /// Returns all seven letters with the center letter first, followed by outer letters in alphabetical order.
    pub fn letters(&self) -> [Letter; PUZZLE_LETTER_COUNT] {
        [
            self.center,
            self.outer_letters[0],
            self.outer_letters[1],
            self.outer_letters[2],
            self.outer_letters[3],
            self.outer_letters[4],
            self.outer_letters[5],
        ]
    }

    /// Returns the 26-bit bitmask of the seven allowed letters.
    #[inline]
    pub fn allowed_mask(&self) -> u32 {
        self.allowed_mask
    }

    /// Returns the 26-bit bitmask of the center letter.
    #[inline]
    pub fn center_mask(&self) -> u32 {
        self.center_mask
    }

    /// Displays the letters separated by spaces with the center letter first (e.g. "C A I M O R T").
    pub fn display_letters(&self) -> String {
        let mut s = String::with_capacity(13);
        s.push(self.center.to_char());
        for letter in self.outer_letters {
            s.push(' ');
            s.push(letter.to_char());
        }
        s
    }

    /// Returns `true` if the word contains all seven distinct letters of the puzzle.
    #[inline]
    pub fn is_pangram(&self, word: &str) -> bool {
        let mut mask = 0u32;
        for b in word.bytes() {
            if b.is_ascii_alphabetic() {
                let idx = (b.to_ascii_uppercase() - b'A') as u32;
                mask |= 1 << idx;
            }
        }
        (mask & self.allowed_mask) == self.allowed_mask
    }

    /// Validates guess syntax and letter rules against the puzzle.
    ///
    /// Checks:
    /// 1. Only ASCII letters A-Z
    /// 2. Word length >= 4
    /// 3. Contains center letter
    /// 4. Contains only letters from the 7 allowed letters
    pub fn validate_candidate(&self, word: &str) -> Result<(), GuessError> {
        let mut len = 0;
        let mut mask = 0u32;

        for b in word.bytes() {
            if !b.is_ascii_alphabetic() {
                return Err(GuessError::MalformedInput { ch: b as char });
            }
            let idx = (b.to_ascii_uppercase() - b'A') as u32;
            let bit = 1 << idx;
            if (self.allowed_mask & bit) == 0 {
                return Err(GuessError::InvalidLetter {
                    letter: (b.to_ascii_uppercase()) as char,
                });
            }
            mask |= bit;
            len += 1;
        }

        if len < MIN_WORD_LENGTH {
            return Err(GuessError::TooShort {
                min_length: MIN_WORD_LENGTH,
                actual_length: len,
            });
        }

        if (mask & self.center_mask) == 0 {
            return Err(GuessError::MissingCenter {
                center: self.center.to_char(),
            });
        }

        Ok(())
    }
}

impl fmt::Debug for SpellingBeePuzzle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SpellingBeePuzzle(center: {}, letters: [{}])",
            self.center.to_char(),
            self.display_letters()
        )
    }
}

impl fmt::Display for SpellingBeePuzzle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_letters())
    }
}
