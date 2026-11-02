//! Lexicon management, allowed guess extraction, curated answers, and override validation.

use std::fs;
use std::path::Path;
use trie::Trie;

use crate::error::WordleError;
use crate::word::Word;

/// Default bundled curated answers included at compile-time as fallback/baseline.
pub const BUNDLED_CURATED_ANSWERS_SRC: &str =
    include_str!("../../assets/wordle_curated_answers.txt");

/// Report returned when an answers override file is successfully validated and loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnswersOverrideReport {
    /// Total non-empty lines parsed from the override file.
    pub total_lines: usize,
    /// Number of distinct valid answers retained.
    pub unique_answers: usize,
    /// Number of duplicate lines filtered out.
    pub duplicates_filtered: usize,
}

/// Owned Wordle lexicon containing allowed guesses and curated answers.
///
/// Guaranteed invariants:
/// 1. `allowed_guesses` is sorted in strictly ascending order and contains only unique 5-letter words.
/// 2. `curated_answers` is nonempty and is a strict subset of `allowed_guesses`.
/// 3. Reusable across replay resets without cloning or mutating the dictionary trie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordleLexicon {
    allowed_guesses: Vec<Word>,
    curated_answers: Vec<Word>,
}

impl WordleLexicon {
    /// Extracts all 5-letter words from a temporary `&Trie` and checks curated answers against them.
    ///
    /// # Invariants Enforced
    /// - `allowed_guesses` are sorted and deduplicated.
    /// - Every curated answer must exist in `allowed_guesses`.
    /// - Curated answers must be nonempty.
    pub fn from_trie(trie: &Trie, curated_answers_text: &str) -> Result<Self, WordleError> {
        let mut allowed = Vec::new();
        trie.visit_words(|w| {
            if w.len() == 5 {
                if let Ok(word) = Word::parse(w) {
                    allowed.push(word);
                }
            }
        });

        allowed.sort_unstable();
        allowed.dedup();

        if allowed.is_empty() {
            return Err(WordleError::NotInDictionary(
                "Trie contains no 5-letter words".to_string(),
            ));
        }

        let curated = Self::parse_answers_subset(curated_answers_text, &allowed)?;

        Ok(Self {
            allowed_guesses: allowed,
            curated_answers: curated,
        })
    }

    /// Directly constructs a `WordleLexicon` from pre-validated vectors.
    ///
    /// Used in tests and fixtures. Verifies the subset and non-empty invariants.
    pub fn new_validated(
        mut allowed_guesses: Vec<Word>,
        mut curated_answers: Vec<Word>,
    ) -> Result<Self, WordleError> {
        if allowed_guesses.is_empty() {
            return Err(WordleError::NotInDictionary(
                "Allowed guesses cannot be empty".to_string(),
            ));
        }
        if curated_answers.is_empty() {
            return Err(WordleError::EmptyAnswers);
        }

        allowed_guesses.sort_unstable();
        allowed_guesses.dedup();

        curated_answers.sort_unstable();
        curated_answers.dedup();

        // Check that every answer is in allowed_guesses
        for ans in &curated_answers {
            if allowed_guesses.binary_search(ans).is_err() {
                return Err(WordleError::NotAnAnswer(ans.as_string()));
            }
        }

        Ok(Self {
            allowed_guesses,
            curated_answers,
        })
    }

    /// Returns a slice of all allowed 5-letter guesses.
    #[inline]
    pub fn allowed_guesses(&self) -> &[Word] {
        &self.allowed_guesses
    }

    /// Returns a slice of the curated answer candidates.
    #[inline]
    pub fn curated_answers(&self) -> &[Word] {
        &self.curated_answers
    }

    /// Checks if a 5-letter word is in the allowed guess dictionary.
    #[inline]
    pub fn is_allowed(&self, word: &Word) -> bool {
        self.allowed_guesses.binary_search(word).is_ok()
    }

    /// Checks if a 5-letter word is in the curated answer subset.
    #[inline]
    pub fn is_answer(&self, word: &Word) -> bool {
        self.curated_answers.binary_search(word).is_ok()
    }

    /// Validates and parses an answers override string, producing an updated `WordleLexicon`
    /// and report.
    ///
    /// # All-or-Nothing Atomicity
    /// If ANY entry in the override is invalid (syntax, length, non-ASCII) or not present
    /// in the allowed guesses dictionary, the ENTIRE override is rejected without mutating
    /// or replacing the existing answers.
    pub fn with_answers_override(
        &self,
        override_text: &str,
    ) -> Result<(Self, AnswersOverrideReport), WordleError> {
        let (answers, report) =
            Self::validate_and_parse_override(override_text, &self.allowed_guesses)?;

        let new_lexicon = Self {
            allowed_guesses: self.allowed_guesses.clone(),
            curated_answers: answers,
        };

        Ok((new_lexicon, report))
    }

    /// Loads and validates an answers override file from a disk path.
    pub fn with_answers_file(
        &self,
        path: &Path,
    ) -> Result<(Self, AnswersOverrideReport), WordleError> {
        let content = fs::read_to_string(path).map_err(|e| WordleError::InvalidAnswerOverride {
            line_number: 0,
            line: path.display().to_string(),
            reason: format!("Failed to read file: {e}"),
        })?;

        self.with_answers_override(&content)
    }

    /// Helper parsing an answer list against allowed guesses, deduplicating while reporting counts.
    fn validate_and_parse_override(
        text: &str,
        allowed: &[Word],
    ) -> Result<(Vec<Word>, AnswersOverrideReport), WordleError> {
        let mut answers = Vec::new();
        let mut total_lines = 0;

        for (line_idx, raw_line) in text.lines().enumerate() {
            let line_number = line_idx + 1;
            let trimmed = raw_line.trim();
            if trimmed.is_empty() {
                continue;
            }

            total_lines += 1;

            let word = Word::parse(trimmed).map_err(|e| WordleError::InvalidAnswerOverride {
                line_number,
                line: trimmed.to_string(),
                reason: e.to_string(),
            })?;

            if allowed.binary_search(&word).is_err() {
                return Err(WordleError::InvalidAnswerOverride {
                    line_number,
                    line: trimmed.to_string(),
                    reason: format!("Word '{word}' is not in the allowed dictionary"),
                });
            }

            answers.push(word);
        }

        if answers.is_empty() {
            return Err(WordleError::EmptyAnswers);
        }

        answers.sort_unstable();
        let before_dedup = answers.len();
        answers.dedup();
        let duplicates_filtered = before_dedup - answers.len();

        let report = AnswersOverrideReport {
            total_lines,
            unique_answers: answers.len(),
            duplicates_filtered,
        };

        Ok((answers, report))
    }

    fn parse_answers_subset(text: &str, allowed: &[Word]) -> Result<Vec<Word>, WordleError> {
        let (answers, _) = Self::validate_and_parse_override(text, allowed)?;
        Ok(answers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_words() -> Vec<Word> {
        vec![
            Word::parse("APPLE").unwrap(),
            Word::parse("CRANE").unwrap(),
            Word::parse("SLATE").unwrap(),
            Word::parse("WATER").unwrap(),
        ]
    }

    #[test]
    fn test_new_validated_success() {
        let words = sample_words();
        let curated = vec![Word::parse("CRANE").unwrap(), Word::parse("APPLE").unwrap()];
        let lex = WordleLexicon::new_validated(words, curated).unwrap();
        assert_eq!(lex.allowed_guesses().len(), 4);
        assert_eq!(lex.curated_answers().len(), 2);
        assert!(lex.is_allowed(&Word::parse("SLATE").unwrap()));
        assert!(lex.is_answer(&Word::parse("CRANE").unwrap()));
        assert!(!lex.is_answer(&Word::parse("SLATE").unwrap()));
    }

    #[test]
    fn test_answer_not_in_allowed_rejected() {
        let words = sample_words();
        let curated = vec![Word::parse("ZEBRA").unwrap()];
        let res = WordleLexicon::new_validated(words, curated);
        assert_eq!(res, Err(WordleError::NotAnAnswer("ZEBRA".to_string())));
    }

    #[test]
    fn test_override_with_duplicates_and_whitespace() {
        let words = sample_words();
        let lex = WordleLexicon::new_validated(words, vec![Word::parse("CRANE").unwrap()]).unwrap();

        let override_src = "apple\n  CRANE  \napple\n\nWATER\n";
        let (new_lex, report) = lex.with_answers_override(override_src).unwrap();

        assert_eq!(report.total_lines, 4);
        assert_eq!(report.unique_answers, 3);
        assert_eq!(report.duplicates_filtered, 1);
        assert_eq!(new_lex.curated_answers().len(), 3);
    }

    #[test]
    fn test_override_all_or_nothing_atomicity() {
        let words = sample_words();
        let lex = WordleLexicon::new_validated(words, vec![Word::parse("CRANE").unwrap()]).unwrap();

        // Line 3 has non-dictionary word "ZEBRA"
        let override_src = "apple\nCRANE\nZEBRA\nWATER\n";
        let res = lex.with_answers_override(override_src);
        assert!(matches!(
            res,
            Err(WordleError::InvalidAnswerOverride { line_number: 3, .. })
        ));

        // Original answers unchanged
        assert_eq!(lex.curated_answers().len(), 1);
    }

    #[test]
    fn test_empty_override_rejected() {
        let words = sample_words();
        let lex = WordleLexicon::new_validated(words, vec![Word::parse("CRANE").unwrap()]).unwrap();

        assert_eq!(
            lex.with_answers_override(""),
            Err(WordleError::EmptyAnswers)
        );
        assert_eq!(
            lex.with_answers_override("   \n\n  "),
            Err(WordleError::EmptyAnswers)
        );
    }
}
