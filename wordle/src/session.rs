//! Wordle session lifecycle, attempt tracking, state transitions, and replay management.

use rand::Rng;

use crate::error::WordleError;
use crate::feedback::{evaluate, Feedback, FeedbackStrategy, LetterFeedback};
use crate::keyboard::Keyboard;
use crate::lexicon::WordleLexicon;
use crate::word::{Word, WORD_LENGTH};

/// Maximum valid attempts allowed in a Wordle round.
pub const MAX_ATTEMPTS: usize = 6;

/// Evaluation result for a successfully submitted guess row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordleGuessEvaluation {
    /// The validated 5-letter word guessed.
    pub guess: Word,
    /// Positional feedback array for the 5 letter slots.
    pub feedback: Feedback,
    /// 1-based attempt index (1..=6).
    pub attempt_number: usize,
}

/// The terminal or active status of a Wordle game session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordleStatus {
    /// Round is currently active; guesses may be submitted.
    InProgress,
    /// The player successfully identified the secret word in `attempts` attempts.
    Won { attempts: usize },
    /// The player exhausted all 6 attempts without discovering the secret.
    Lost { attempts: usize, secret: Word },
    /// The player voluntarily forfeited/quit the round midway.
    Quit { attempts: usize, secret: Word },
}

/// The result returned after a guess is evaluated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordleSubmissionResult {
    /// Valid guess evaluated; round continues.
    Evaluated(WordleGuessEvaluation),
    /// Valid guess matched the secret word; player has won.
    Won(WordleGuessEvaluation),
    /// Valid guess did not match the secret and was the 6th attempt; player has lost.
    Lost(WordleGuessEvaluation, Word),
}

/// Wordle game session.
///
/// Encapsulates private round state, private secret target, private attempt counter,
/// private history, and the owned lexicon.
#[derive(Debug, Clone)]
pub struct WordleGame {
    lexicon: WordleLexicon,
    secret: Word,
    strategy: FeedbackStrategy,
    attempts_used: usize,
    history: Vec<WordleGuessEvaluation>,
    status: WordleStatus,
    keyboard: Keyboard,
}

impl WordleGame {
    /// Creates a new game session with a deterministic, pre-checked secret word.
    ///
    /// # Errors
    /// Returns `Err(WordleError::NotAnAnswer)` if `secret` is not in `lexicon.curated_answers()`.
    pub fn new(
        lexicon: WordleLexicon,
        secret: Word,
        strategy: FeedbackStrategy,
    ) -> Result<Self, WordleError> {
        if !lexicon.is_answer(&secret) {
            return Err(WordleError::NotAnAnswer(secret.as_string()));
        }

        Ok(Self {
            lexicon,
            secret,
            strategy,
            attempts_used: 0,
            history: Vec::with_capacity(MAX_ATTEMPTS),
            status: WordleStatus::InProgress,
            keyboard: Keyboard::new(),
        })
    }

    /// Creates a new game session selecting a secret randomly using an injected RNG.
    ///
    /// Uniformly samples from `lexicon.curated_answers()`.
    pub fn with_rng<R: Rng + ?Sized>(
        lexicon: WordleLexicon,
        strategy: FeedbackStrategy,
        rng: &mut R,
    ) -> Result<Self, WordleError> {
        let answers = lexicon.curated_answers();
        if answers.is_empty() {
            return Err(WordleError::EmptyAnswers);
        }

        let idx = rng.gen_range(0..answers.len());
        let secret = answers[idx];

        Ok(Self {
            lexicon,
            secret,
            strategy,
            attempts_used: 0,
            history: Vec::with_capacity(MAX_ATTEMPTS),
            status: WordleStatus::InProgress,
            keyboard: Keyboard::new(),
        })
    }

    /// Evaluates a submitted guess string against the secret word.
    ///
    /// # State Mutation & Validation Order:
    /// 1. Terminal Check: If `self.status != InProgress`, returns `Err(WordleError::GameFinished)`
    ///    with zero mutation to attempts or history.
    /// 2. Parsing & Syntax Check: If `guess_str` is invalid length or contains invalid characters,
    ///    returns typed `WordleError` with zero mutation to attempts or history.
    /// 3. Dictionary Membership Check: If the word is not in `lexicon.allowed_guesses()`,
    ///    returns `Err(WordleError::NotInDictionary)` with zero mutation to attempts or history.
    /// 4. Valid Guess Accepted:
    ///    - Increments `attempts_used`.
    ///    - Repeated valid guesses are permitted and consume a turn.
    ///    - Evaluates feedback using `self.strategy`.
    ///    - Updates keyboard letter discovery statuses monotonically.
    ///    - Appends row to history.
    ///    - Checks for win condition (all 5 letters `Correct`).
    ///    - **Final-Attempt Invariant**: A correct 6th guess is strictly a `Won` outcome,
    ///      evaluated before checking attempt exhaustion.
    ///    - If not won and `attempts_used >= MAX_ATTEMPTS`, transitions to `Lost`.
    pub fn submit_guess(&mut self, guess_str: &str) -> Result<WordleSubmissionResult, WordleError> {
        if self.status != WordleStatus::InProgress {
            return Err(WordleError::GameFinished);
        }

        let word = Word::parse(guess_str)?;

        if !self.lexicon.is_allowed(&word) {
            return Err(WordleError::NotInDictionary(word.as_string()));
        }

        self.attempts_used += 1;
        let feedback = evaluate(&self.secret, &word, self.strategy);

        for i in 0..WORD_LENGTH {
            self.keyboard.observe(word[i], feedback[i]);
        }

        let eval = WordleGuessEvaluation {
            guess: word,
            feedback,
            attempt_number: self.attempts_used,
        };
        self.history.push(eval.clone());

        if feedback == [LetterFeedback::Correct; WORD_LENGTH] {
            self.status = WordleStatus::Won {
                attempts: self.attempts_used,
            };
            return Ok(WordleSubmissionResult::Won(eval));
        }

        if self.attempts_used >= MAX_ATTEMPTS {
            self.status = WordleStatus::Lost {
                attempts: self.attempts_used,
                secret: self.secret,
            };
            return Ok(WordleSubmissionResult::Lost(eval, self.secret));
        }

        Ok(WordleSubmissionResult::Evaluated(eval))
    }

    /// Voluntarily quits the current round, transitioning to `WordleStatus::Quit`
    /// and revealing the secret word.
    ///
    /// # Errors
    /// Returns `Err(WordleError::GameFinished)` if the game was already in a terminal state.
    pub fn quit(&mut self) -> Result<Word, WordleError> {
        if self.status != WordleStatus::InProgress {
            return Err(WordleError::GameFinished);
        }

        self.status = WordleStatus::Quit {
            attempts: self.attempts_used,
            secret: self.secret,
        };

        Ok(self.secret)
    }

    /// Resets the game session for a replay round using a new secret sampled from an injected RNG.
    ///
    /// Reuses the existing owned `WordleLexicon` without requiring trie re-traversal or re-allocation.
    pub fn reset_with_rng<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), WordleError> {
        let answers = self.lexicon.curated_answers();
        if answers.is_empty() {
            return Err(WordleError::EmptyAnswers);
        }

        let idx = rng.gen_range(0..answers.len());
        self.secret = answers[idx];
        self.attempts_used = 0;
        self.history.clear();
        self.status = WordleStatus::InProgress;
        self.keyboard.reset();

        Ok(())
    }

    /// Resets the game session for a deterministic replay round with a specified secret.
    pub fn reset_with_secret(&mut self, secret: Word) -> Result<(), WordleError> {
        if !self.lexicon.is_answer(&secret) {
            return Err(WordleError::NotAnAnswer(secret.as_string()));
        }

        self.secret = secret;
        self.attempts_used = 0;
        self.history.clear();
        self.status = WordleStatus::InProgress;
        self.keyboard.reset();

        Ok(())
    }

    /// Number of valid, evaluated attempts used so far.
    #[inline]
    pub fn attempts_used(&self) -> usize {
        self.attempts_used
    }

    /// Number of valid attempts remaining before exhaustion.
    #[inline]
    pub fn attempts_remaining(&self) -> usize {
        MAX_ATTEMPTS.saturating_sub(self.attempts_used)
    }

    /// History of past evaluated guesses in this round.
    #[inline]
    pub fn history(&self) -> &[WordleGuessEvaluation] {
        &self.history
    }

    /// Current game status.
    #[inline]
    pub fn status(&self) -> &WordleStatus {
        &self.status
    }

    /// Letter keyboard status tracker.
    #[inline]
    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    /// Reference to the underlying owned lexicon.
    #[inline]
    pub fn lexicon(&self) -> &WordleLexicon {
        &self.lexicon
    }

    /// Feedback strategy being used.
    #[inline]
    pub fn strategy(&self) -> FeedbackStrategy {
        self.strategy
    }

    /// Revealed secret word, available ONLY if the round has finished (`Won`, `Lost`, or `Quit`).
    ///
    /// Returns `None` if the round is still `InProgress` to prevent information leakage.
    pub fn revealed_secret(&self) -> Option<Word> {
        match self.status {
            WordleStatus::InProgress => None,
            WordleStatus::Won { .. } => Some(self.secret),
            WordleStatus::Lost { secret, .. } => Some(secret),
            WordleStatus::Quit { secret, .. } => Some(secret),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lexicon() -> WordleLexicon {
        let allowed = vec![
            Word::parse("APPLE").unwrap(),
            Word::parse("CRANE").unwrap(),
            Word::parse("SLATE").unwrap(),
            Word::parse("WATER").unwrap(),
            Word::parse("ROUND").unwrap(),
            Word::parse("LIGHT").unwrap(),
        ];
        let answers = vec![Word::parse("CRANE").unwrap(), Word::parse("APPLE").unwrap()];
        WordleLexicon::new_validated(allowed, answers).unwrap()
    }

    #[test]
    fn test_initial_state() {
        let lex = sample_lexicon();
        let game =
            WordleGame::new(lex, Word::parse("CRANE").unwrap(), FeedbackStrategy::Marker).unwrap();

        assert_eq!(game.attempts_used(), 0);
        assert_eq!(game.attempts_remaining(), 6);
        assert_eq!(game.status(), &WordleStatus::InProgress);
        assert!(game.history().is_empty());
        assert_eq!(game.revealed_secret(), None);
    }

    #[test]
    fn test_invalid_input_does_not_mutate_state() {
        let lex = sample_lexicon();
        let mut game =
            WordleGame::new(lex, Word::parse("CRANE").unwrap(), FeedbackStrategy::Marker).unwrap();

        // 1. Empty input
        assert_eq!(game.submit_guess(""), Err(WordleError::EmptyInput));
        assert_eq!(game.attempts_used(), 0);
        assert_eq!(game.history().len(), 0);

        // 2. Invalid length
        assert_eq!(
            game.submit_guess("CAT"),
            Err(WordleError::InvalidLength {
                expected: 5,
                actual: 3
            })
        );
        assert_eq!(game.attempts_used(), 0);

        // 3. Invalid characters
        assert_eq!(
            game.submit_guess("CR4NE"),
            Err(WordleError::InvalidCharacter {
                character: '4',
                position: 2
            })
        );
        assert_eq!(game.attempts_used(), 0);

        // 4. Not in dictionary
        assert_eq!(
            game.submit_guess("ZZZZZ"),
            Err(WordleError::NotInDictionary("ZZZZZ".to_string()))
        );
        assert_eq!(game.attempts_used(), 0);
        assert_eq!(game.history().len(), 0);
        assert_eq!(game.status(), &WordleStatus::InProgress);
    }

    #[test]
    fn test_repeated_valid_guess_consumes_turn() {
        let lex = sample_lexicon();
        let mut game =
            WordleGame::new(lex, Word::parse("CRANE").unwrap(), FeedbackStrategy::Marker).unwrap();

        let res1 = game.submit_guess("SLATE").unwrap();
        assert_eq!(game.attempts_used(), 1);
        assert!(matches!(res1, WordleSubmissionResult::Evaluated(_)));

        // Repeating valid guess "SLATE"
        let res2 = game.submit_guess("SLATE").unwrap();
        assert_eq!(game.attempts_used(), 2);
        assert_eq!(game.attempts_remaining(), 4);
        assert!(matches!(res2, WordleSubmissionResult::Evaluated(_)));
        assert_eq!(game.history().len(), 2);
    }

    #[test]
    fn test_final_attempt_win_invariant() {
        let lex = sample_lexicon();
        let mut game =
            WordleGame::new(lex, Word::parse("CRANE").unwrap(), FeedbackStrategy::Marker).unwrap();

        // Use 5 non-winning valid attempts
        for _ in 0..5 {
            let res = game.submit_guess("SLATE").unwrap();
            assert!(matches!(res, WordleSubmissionResult::Evaluated(_)));
        }
        assert_eq!(game.attempts_used(), 5);
        assert_eq!(game.attempts_remaining(), 1);

        // 6th attempt is the correct secret
        let res6 = game.submit_guess("CRANE").unwrap();
        assert!(matches!(res6, WordleSubmissionResult::Won(_)));
        assert_eq!(game.attempts_used(), 6);
        assert_eq!(game.status(), &WordleStatus::Won { attempts: 6 });
        assert_eq!(game.revealed_secret(), Some(Word::parse("CRANE").unwrap()));
    }

    #[test]
    fn test_sixth_attempt_loss_transition() {
        let lex = sample_lexicon();
        let mut game =
            WordleGame::new(lex, Word::parse("CRANE").unwrap(), FeedbackStrategy::Marker).unwrap();

        for _ in 0..5 {
            game.submit_guess("SLATE").unwrap();
        }
        assert_eq!(game.attempts_used(), 5);

        // 6th attempt is incorrect
        let res6 = game.submit_guess("WATER").unwrap();
        assert!(matches!(res6, WordleSubmissionResult::Lost(_, _)));
        assert_eq!(game.attempts_used(), 6);
        assert_eq!(game.attempts_remaining(), 0);
        assert_eq!(
            game.status(),
            &WordleStatus::Lost {
                attempts: 6,
                secret: Word::parse("CRANE").unwrap()
            }
        );
        assert_eq!(game.revealed_secret(), Some(Word::parse("CRANE").unwrap()));
    }

    #[test]
    fn test_terminal_state_guard_rejects_subsequent_guesses() {
        let lex = sample_lexicon();
        let mut game =
            WordleGame::new(lex, Word::parse("CRANE").unwrap(), FeedbackStrategy::Marker).unwrap();

        // Win immediately
        game.submit_guess("CRANE").unwrap();
        assert_eq!(game.status(), &WordleStatus::Won { attempts: 1 });

        // Attempting a 7th or post-finish submission
        let post_err = game.submit_guess("WATER");
        assert_eq!(post_err, Err(WordleError::GameFinished));
        assert_eq!(game.attempts_used(), 1);
        assert_eq!(game.history().len(), 1);
    }

    #[test]
    fn test_voluntary_quit_reveals_secret() {
        let lex = sample_lexicon();
        let mut game =
            WordleGame::new(lex, Word::parse("CRANE").unwrap(), FeedbackStrategy::Marker).unwrap();

        game.submit_guess("SLATE").unwrap();
        assert_eq!(game.attempts_used(), 1);

        let secret = game.quit().unwrap();
        assert_eq!(secret, Word::parse("CRANE").unwrap());
        assert_eq!(
            game.status(),
            &WordleStatus::Quit {
                attempts: 1,
                secret
            }
        );

        // Subsequent guess rejected
        assert_eq!(game.submit_guess("WATER"), Err(WordleError::GameFinished));
    }

    #[test]
    fn test_reset_reuses_lexicon() {
        let lex = sample_lexicon();
        let mut game =
            WordleGame::new(lex, Word::parse("CRANE").unwrap(), FeedbackStrategy::Marker).unwrap();

        game.submit_guess("SLATE").unwrap();
        assert_eq!(game.attempts_used(), 1);

        game.reset_with_secret(Word::parse("APPLE").unwrap())
            .unwrap();
        assert_eq!(game.attempts_used(), 0);
        assert_eq!(game.attempts_remaining(), 6);
        assert_eq!(game.status(), &WordleStatus::InProgress);
        assert!(game.history().is_empty());
        assert_eq!(
            game.keyboard()
                .status(trie::letter::Letter::from_char('S').unwrap()),
            crate::keyboard::KeyStatus::Unknown
        );
    }
}
