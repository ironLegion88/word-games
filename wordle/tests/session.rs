use rand::rngs::StdRng;
use rand::SeedableRng;
use wordle::feedback::FeedbackStrategy;
use wordle::{
    Word, WordleError, WordleGame, WordleLexicon, WordleStatus, WordleSubmissionResult,
    MAX_ATTEMPTS,
};

fn sample_lexicon() -> WordleLexicon {
    let allowed = vec![
        Word::parse("APPLE").unwrap(),
        Word::parse("CRANE").unwrap(),
        Word::parse("SLATE").unwrap(),
        Word::parse("WATER").unwrap(),
        Word::parse("ROUND").unwrap(),
        Word::parse("LIGHT").unwrap(),
    ];
    let curated = vec![Word::parse("CRANE").unwrap(), Word::parse("APPLE").unwrap()];
    WordleLexicon::new_validated(allowed, curated).unwrap()
}

#[test]
fn test_game_session_initial_state() {
    let lex = sample_lexicon();
    let game = WordleGame::new(
        lex,
        Word::parse("CRANE").unwrap(),
        FeedbackStrategy::Frequency,
    )
    .unwrap();

    assert_eq!(game.attempts_used(), 0);
    assert_eq!(game.attempts_remaining(), MAX_ATTEMPTS);
    assert_eq!(game.status(), &WordleStatus::InProgress);
    assert!(game.history().is_empty());
    assert_eq!(game.revealed_secret(), None);
}

#[test]
fn test_invalid_guess_does_not_consume_turn() {
    let lex = sample_lexicon();
    let mut game = WordleGame::new(
        lex,
        Word::parse("CRANE").unwrap(),
        FeedbackStrategy::Frequency,
    )
    .unwrap();

    // 1. Bad length
    let _ = game.submit_guess("CAT");
    assert_eq!(game.attempts_used(), 0);

    // 2. Bad char
    let _ = game.submit_guess("CR4NE");
    assert_eq!(game.attempts_used(), 0);

    // 3. Not in dictionary
    let _ = game.submit_guess("ZZZZZ");
    assert_eq!(game.attempts_used(), 0);

    // 4. Whitespace only
    let _ = game.submit_guess("   ");
    assert_eq!(game.attempts_used(), 0);

    assert_eq!(game.attempts_remaining(), 6);
    assert!(game.history().is_empty());
    assert_eq!(game.status(), &WordleStatus::InProgress);
}

#[test]
fn test_repeated_valid_guess_consumes_turn() {
    let lex = sample_lexicon();
    let mut game = WordleGame::new(
        lex,
        Word::parse("CRANE").unwrap(),
        FeedbackStrategy::Frequency,
    )
    .unwrap();

    let res1 = game.submit_guess("SLATE").unwrap();
    assert!(matches!(res1, WordleSubmissionResult::Evaluated(_)));
    assert_eq!(game.attempts_used(), 1);

    // Repeated submission of SLATE
    let res2 = game.submit_guess("SLATE").unwrap();
    assert!(matches!(res2, WordleSubmissionResult::Evaluated(_)));
    assert_eq!(game.attempts_used(), 2);
    assert_eq!(game.attempts_remaining(), 4);
    assert_eq!(game.history().len(), 2);
}

#[test]
fn test_final_sixth_attempt_win_evaluated_first() {
    let lex = sample_lexicon();
    let mut game = WordleGame::new(
        lex,
        Word::parse("CRANE").unwrap(),
        FeedbackStrategy::Frequency,
    )
    .unwrap();

    for _ in 0..5 {
        game.submit_guess("SLATE").unwrap();
    }
    assert_eq!(game.attempts_used(), 5);

    // 6th attempt is the winning answer
    let res6 = game.submit_guess("CRANE").unwrap();
    assert!(matches!(res6, WordleSubmissionResult::Won(_)));
    assert_eq!(game.attempts_used(), 6);
    assert_eq!(game.status(), &WordleStatus::Won { attempts: 6 });
    assert_eq!(game.revealed_secret(), Some(Word::parse("CRANE").unwrap()));
}

#[test]
fn test_sixth_attempt_loss_transitions_to_lost() {
    let lex = sample_lexicon();
    let mut game = WordleGame::new(
        lex,
        Word::parse("CRANE").unwrap(),
        FeedbackStrategy::Frequency,
    )
    .unwrap();

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
fn test_terminal_state_guard_blocks_seventh_submission() {
    let lex = sample_lexicon();
    let mut game = WordleGame::new(
        lex,
        Word::parse("CRANE").unwrap(),
        FeedbackStrategy::Frequency,
    )
    .unwrap();

    // Win on attempt 1
    game.submit_guess("CRANE").unwrap();
    assert_eq!(game.status(), &WordleStatus::Won { attempts: 1 });

    // 2nd attempt after finish
    let res = game.submit_guess("WATER");
    assert_eq!(res, Err(WordleError::GameFinished));
    assert_eq!(game.attempts_used(), 1);
    assert_eq!(game.history().len(), 1);
}

#[test]
fn test_voluntary_quit_reveals_secret_and_blocks_further_play() {
    let lex = sample_lexicon();
    let mut game = WordleGame::new(
        lex,
        Word::parse("CRANE").unwrap(),
        FeedbackStrategy::Frequency,
    )
    .unwrap();

    game.submit_guess("SLATE").unwrap();
    let secret = game.quit().unwrap();
    assert_eq!(secret, Word::parse("CRANE").unwrap());
    assert_eq!(
        game.status(),
        &WordleStatus::Quit {
            attempts: 1,
            secret
        }
    );

    // Post-quit submission fails
    assert_eq!(game.submit_guess("WATER"), Err(WordleError::GameFinished));
    // Redundant quit fails
    assert_eq!(game.quit(), Err(WordleError::GameFinished));
}

#[test]
fn test_seeded_rng_selection_and_replay_reset() {
    let lex = sample_lexicon();
    let mut rng = StdRng::seed_from_u64(0x5EED_2026);

    let mut game =
        WordleGame::with_rng(lex.clone(), FeedbackStrategy::Frequency, &mut rng).unwrap();

    let initial_status = game.status().clone();
    assert_eq!(initial_status, WordleStatus::InProgress);

    game.submit_guess("SLATE").unwrap();
    assert_eq!(game.attempts_used(), 1);

    // Reset with new RNG selection
    game.reset_with_rng(&mut rng).unwrap();
    assert_eq!(game.attempts_used(), 0);
    assert_eq!(game.attempts_remaining(), 6);
    assert_eq!(game.status(), &WordleStatus::InProgress);
    assert!(game.history().is_empty());
}
