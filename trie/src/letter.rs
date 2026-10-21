use std::borrow::Cow;

use crate::error::WordError;

/// Number of letters in the standard English alphabet (A-Z).
pub const ALPHABET_SIZE: usize = 26;

/// A validated uppercase ASCII letter in the range `'A'..='Z'`.
///
/// Stored internally as a 0-indexed byte in `0..=25` where `0` corresponds to `'A'`
/// and `25` corresponds to `'Z'`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Letter(u8);

impl Letter {
    /// Attempts to create a `Letter` from an ASCII byte (accepts uppercase `b'A'..=b'Z'` or lowercase `b'a'..=b'z'`).
    /// Returns `None` if the byte is outside the ASCII alphabetic range.
    #[inline]
    pub const fn from_ascii_byte(byte: u8) -> Option<Self> {
        match byte {
            b'A'..=b'Z' => Some(Self(byte - b'A')),
            b'a'..=b'z' => Some(Self(byte - b'a')),
            _ => None,
        }
    }

    /// Attempts to create a `Letter` from a `char`.
    /// Returns `None` if the character is not an ASCII alphabetic character.
    #[inline]
    pub fn from_char(c: char) -> Option<Self> {
        if c.is_ascii() {
            Self::from_ascii_byte(c as u8)
        } else {
            None
        }
    }

    /// Creates a `Letter` from an index `0..26`.
    /// Returns `None` if `index >= ALPHABET_SIZE`.
    #[inline]
    pub const fn from_index(index: usize) -> Option<Self> {
        if index < ALPHABET_SIZE {
            Some(Self(index as u8))
        } else {
            None
        }
    }

    /// Returns the 0-based index of this letter (`0` for `'A'`, `25` for `'Z'`).
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// Returns the uppercase ASCII `char` representation of this letter (`'A'..='Z'`).
    #[inline]
    pub const fn to_char(self) -> char {
        (b'A' + self.0) as char
    }

    /// Returns the uppercase ASCII byte (`b'A'..=b'Z'`).
    #[inline]
    pub const fn to_ascii_byte(self) -> u8 {
        b'A' + self.0
    }
}

/// Validates and normalizes an input word to uppercase ASCII.
///
/// If the input string is already entirely uppercase ASCII, returns `Cow::Borrowed(word)`
/// without allocating heap memory. If it contains lowercase letters, converts them to uppercase
/// and returns `Cow::Owned(upper_string)`.
///
/// # Errors
/// Returns `Err(WordError::EmptyWord)` if the string is empty.
/// Returns `Err(WordError::InvalidCharacter)` on the first non-ASCII-alphabetic character
/// (e.g. digits, punctuation, hyphens, spaces, or non-ASCII characters).
pub fn normalize_word(word: &str) -> Result<Cow<'_, str>, WordError> {
    if word.is_empty() {
        return Err(WordError::EmptyWord);
    }

    let mut needs_allocation = false;
    for (i, &b) in word.as_bytes().iter().enumerate() {
        match b {
            b'A'..=b'Z' => {}
            b'a'..=b'z' => {
                needs_allocation = true;
            }
            _ => {
                let ch = word[i..].chars().next().unwrap_or(b as char);
                return Err(WordError::InvalidCharacter {
                    character: ch,
                    position: i,
                });
            }
        }
    }

    if needs_allocation {
        let mut owned = String::with_capacity(word.len());
        for &b in word.as_bytes() {
            owned.push(b.to_ascii_uppercase() as char);
        }
        Ok(Cow::Owned(owned))
    } else {
        Ok(Cow::Borrowed(word))
    }
}

/// Validates that a string is a valid word and returns an iterator over its `Letter` values.
///
/// Returns `Err(WordError)` if the string is empty or contains non-ASCII-alphabetic characters.
pub fn validate_and_parse_letters(word: &str) -> Result<impl Iterator<Item = Letter> + '_, WordError> {
    if word.is_empty() {
        return Err(WordError::EmptyWord);
    }

    for (i, &b) in word.as_bytes().iter().enumerate() {
        if Letter::from_ascii_byte(b).is_none() {
            let ch = word[i..].chars().next().unwrap_or(b as char);
            return Err(WordError::InvalidCharacter {
                character: ch,
                position: i,
            });
        }
    }

    Ok(word.as_bytes().iter().map(|&b| Letter::from_ascii_byte(b).expect("validated")))
}

/// Validates a prefix string (which is permitted to be empty) and returns an iterator of `Letter` values.
///
/// Returns `Err(WordError)` if any character is not an ASCII alphabetic character.
pub fn validate_and_parse_prefix(prefix: &str) -> Result<impl Iterator<Item = Letter> + '_, WordError> {
    for (i, &b) in prefix.as_bytes().iter().enumerate() {
        if Letter::from_ascii_byte(b).is_none() {
            let ch = prefix[i..].chars().next().unwrap_or(b as char);
            return Err(WordError::InvalidCharacter {
                character: ch,
                position: i,
            });
        }
    }

    Ok(prefix.as_bytes().iter().map(|&b| Letter::from_ascii_byte(b).expect("validated")))
}
