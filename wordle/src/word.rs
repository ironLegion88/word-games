//! Validated five-letter ASCII word representation.

use std::fmt;
use std::ops::Index;
use trie::letter::Letter;

use crate::error::WordleError;

/// Wordle requires exactly 5 letters.
pub const WORD_LENGTH: usize = 5;

/// A validated five-letter uppercase ASCII word.
///
/// Guaranteed to contain exactly 5 uppercase ASCII letters (`'A'..='Z'`).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Word([Letter; WORD_LENGTH]);

impl Word {
    /// Attempts to parse and validate an arbitrary string slice into a 5-letter `Word`.
    ///
    /// Accepts uppercase or lowercase ASCII letters. Rejects non-ASCII, punctuation,
    /// spaces, digits, and lengths other than 5.
    pub fn parse(input: &str) -> Result<Self, WordleError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(WordleError::EmptyInput);
        }

        let char_count = trimmed.chars().count();
        if char_count != WORD_LENGTH {
            return Err(WordleError::InvalidLength {
                expected: WORD_LENGTH,
                actual: char_count,
            });
        }

        let mut letters = [Letter::from_ascii_byte(b'A').unwrap(); WORD_LENGTH];
        for (i, ch) in trimmed.chars().enumerate() {
            match Letter::from_char(ch) {
                Some(letter) => letters[i] = letter,
                None => {
                    return Err(WordleError::InvalidCharacter {
                        character: ch,
                        position: i,
                    });
                }
            }
        }

        Ok(Self(letters))
    }

    /// Creates a `Word` directly from an array of 5 validated `Letter` values.
    #[inline]
    pub const fn from_letters(letters: [Letter; WORD_LENGTH]) -> Self {
        Self(letters)
    }

    /// Returns a reference to the underlying letter array.
    #[inline]
    pub const fn letters(&self) -> &[Letter; WORD_LENGTH] {
        &self.0
    }

    /// Formats the word as an uppercase 5-character string.
    pub fn as_string(&self) -> String {
        let mut s = String::with_capacity(WORD_LENGTH);
        for &l in &self.0 {
            s.push(l.to_char());
        }
        s
    }

    /// Returns the uppercase ASCII bytes of the word.
    #[inline]
    pub const fn to_bytes(&self) -> [u8; WORD_LENGTH] {
        [
            self.0[0].to_ascii_byte(),
            self.0[1].to_ascii_byte(),
            self.0[2].to_ascii_byte(),
            self.0[3].to_ascii_byte(),
            self.0[4].to_ascii_byte(),
        ]
    }
}

impl Index<usize> for Word {
    type Output = Letter;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &l in &self.0 {
            write!(f, "{}", l.to_char())?;
        }
        Ok(())
    }
}

impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Word(\"{}\")", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_word_parsing() {
        let w = Word::parse("crane").unwrap();
        assert_eq!(w.as_string(), "CRANE");
        assert_eq!(w.to_bytes(), *b"CRANE");
        assert_eq!(w[0], Letter::from_char('C').unwrap());
        assert_eq!(w[4], Letter::from_char('E').unwrap());
    }

    #[test]
    fn test_empty_input_rejected() {
        assert_eq!(Word::parse(""), Err(WordleError::EmptyInput));
        assert_eq!(Word::parse("   "), Err(WordleError::EmptyInput));
    }

    #[test]
    fn test_invalid_length_rejected() {
        assert_eq!(
            Word::parse("CAT"),
            Err(WordleError::InvalidLength {
                expected: 5,
                actual: 3
            })
        );
        assert_eq!(
            Word::parse("PLANET"),
            Err(WordleError::InvalidLength {
                expected: 5,
                actual: 6
            })
        );
    }

    #[test]
    fn test_invalid_characters_rejected() {
        assert_eq!(
            Word::parse("CR4NE"),
            Err(WordleError::InvalidCharacter {
                character: '4',
                position: 2
            })
        );
        assert_eq!(
            Word::parse("CR!NE"),
            Err(WordleError::InvalidCharacter {
                character: '!',
                position: 2
            })
        );
    }
}
