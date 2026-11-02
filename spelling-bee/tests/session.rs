use spelling_bee::{GuessError, GuessOutcome, SpellingBeePuzzle, SpellingBeeSession};

fn build_test_session() -> SpellingBeeSession {
    let puzzle = SpellingBeePuzzle::from_raw("CAIMORT").unwrap();
    let solutions = vec![
        "ACAI".to_string(),
        "AROMATIC".to_string(), // pangram
        "ATOMIC".to_string(),
        "CITRIC".to_string(),
        "MICRO".to_string(),
    ];
    SpellingBeeSession::new(puzzle, solutions)
}

#[test]
fn test_session_basic_guesses_and_scoring() {
    let mut session = build_test_session();
    assert_eq!(session.current_score(), 0);
    assert_eq!(session.accepted_guesses().len(), 0);
    assert!(!session.is_terminated());

    // Guess 4-letter word: 1 point
    let out1 = session.submit_guess("ACAI");
    assert_eq!(
        out1,
        GuessOutcome::Accepted {
            word: "ACAI".into(),
            points: 1,
            is_pangram: false,
            queen_bee: false,
        }
    );
    assert_eq!(session.current_score(), 1);
    assert!(session.check_invariants().is_ok());

    // Duplicate guess rejected
    let out_dup = session.submit_guess("acai");
    assert_eq!(
        out_dup,
        GuessOutcome::AlreadyGuessed {
            word: "ACAI".into(),
        }
    );
    assert_eq!(session.current_score(), 1);
    assert!(session.check_invariants().is_ok());

    // Guess pangram: 8 letters + 7 bonus = 15 points
    let out_pan = session.submit_guess("aromatic");
    assert_eq!(
        out_pan,
        GuessOutcome::Accepted {
            word: "AROMATIC".into(),
            points: 15,
            is_pangram: true,
            queen_bee: false,
        }
    );
    assert_eq!(session.current_score(), 16);
    assert!(session.check_invariants().is_ok());

    // Invalid guess: too short
    let out_short = session.submit_guess("cat");
    assert!(matches!(
        out_short,
        GuessOutcome::InvalidWord(GuessError::TooShort { .. })
    ));
    assert_eq!(session.current_score(), 16);

    // Invalid guess: missing center
    let out_nocenter = session.submit_guess("atom");
    assert!(matches!(
        out_nocenter,
        GuessOutcome::InvalidWord(GuessError::MissingCenter { .. })
    ));
    assert_eq!(session.current_score(), 16);

    // Invalid guess: not in dictionary/solutions
    let out_not_in = session.submit_guess("comic");
    assert_eq!(
        out_not_in,
        GuessOutcome::InvalidWord(GuessError::NotInDictionary)
    );
    assert_eq!(session.current_score(), 16);
}

#[test]
fn test_session_queen_bee_and_immutability() {
    let mut session = build_test_session();

    assert!(!session.is_queen_bee());
    session.submit_guess("ACAI");
    session.submit_guess("AROMATIC");
    session.submit_guess("ATOMIC");
    session.submit_guess("CITRIC");

    assert!(!session.is_terminated());
    assert!(!session.is_queen_bee());

    // Final word triggers Queen Bee
    let out_final = session.submit_guess("MICRO");
    match out_final {
        GuessOutcome::Accepted { queen_bee, .. } => assert!(queen_bee),
        _ => panic!("expected accepted with queen bee"),
    }

    assert!(session.is_queen_bee());
    assert!(session.is_terminated());
    assert!(session.check_invariants().is_ok());

    let final_score = session.current_score();

    // Subsequent submissions are GameFinished and mutate nothing
    let out_after = session.submit_guess("ACAI");
    assert_eq!(out_after, GuessOutcome::GameFinished);
    assert_eq!(session.current_score(), final_score);
    assert_eq!(session.accepted_guesses().len(), 5);
}

#[test]
fn test_session_early_quit_summary() {
    let mut session = build_test_session();
    session.submit_guess("ACAI");
    session.submit_guess("MICRO");

    let summary = session.quit();
    assert_eq!(summary.score, 6); // 1 + 5
    assert_eq!(summary.found_count, 2);
    assert_eq!(summary.total_solutions, 5);
    assert_eq!(
        summary.missed_words,
        vec![
            "AROMATIC".to_string(),
            "ATOMIC".to_string(),
            "CITRIC".to_string()
        ]
    );

    assert!(session.is_terminated());
    // Post quit submission is GameFinished
    assert_eq!(session.submit_guess("CITRIC"), GuessOutcome::GameFinished);
}

#[test]
fn test_hint_grid_and_counts_totals() {
    let session = build_test_session();
    let grid = session.hint_grid();

    // Verify row sums match grand total
    let row_sum: usize = grid.row_totals.values().sum();
    assert_eq!(row_sum, grid.grand_total_words);

    // Verify col sums match grand total
    let col_sum: usize = grid.col_totals.values().sum();
    assert_eq!(col_sum, grid.grand_total_words);

    assert_eq!(grid.grand_total_words, 5);
    assert_eq!(grid.total_pangrams, 1);

    let two_letters = session.two_letter_counts();
    let prefix_sum: usize = two_letters.counts.values().sum();
    assert_eq!(prefix_sum, 5);
}

#[test]
fn test_randomized_session_invariants() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let seeds = [0xC0FFEE_u64, 0x5EED_2026_u64, 0xBADC0DE_u64];
    let candidate_pool = [
        "ACAI", "AROMATIC", "ATOMIC", "CITRIC", "MICRO", // valid
        "CAT", "DOG", "ATOM", "ZEBRA", "COMA", "ROCOCO", // invalid or not in solutions
        "acai", "aromatic", "", "C", "A", "M", // mixed case / malformed
    ];

    for &seed in &seeds {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut session = build_test_session();

        for _ in 0..100 {
            if session.is_terminated() {
                break;
            }
            let choice = rng.gen_range(0..candidate_pool.len());
            let word = candidate_pool[choice];
            let _ = session.submit_guess(word);
            session
                .check_invariants()
                .expect("invariant failed during fuzzing");
        }
    }
}
