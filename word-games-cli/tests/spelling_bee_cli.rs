use spelling_bee::{SpellingBeePuzzle, SpellingBeeSession};
use std::io::Cursor;
use word_games_cli::spelling_bee::play_session;
use word_games_cli::terminal::ColorMode;

#[test]
fn test_cli_play_session_batch_and_quit() {
    let puzzle = SpellingBeePuzzle::from_raw("CAIMORT").unwrap();
    let solutions = vec![
        "ACAI".to_string(),
        "AROMATIC".to_string(),
        "ATOMIC".to_string(),
        "CITRIC".to_string(),
        "MICRO".to_string(),
    ];
    let session = SpellingBeeSession::new(puzzle, solutions);

    // Provide comma-separated guesses, followed by /hints, /guesses, /help, and /quit
    let input_script = "ACAI, AROMATIC\n/hints\n/guesses\n/help\n/quit\n";
    let mut reader = Cursor::new(input_script);
    let mut output = Vec::new();

    let res = play_session(session, &mut reader, &mut output, ColorMode::Never);
    assert!(res.is_ok());

    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("✔ ACAI"));
    assert!(out_str.contains("✔ AROMATIC"));
    assert!(out_str.contains("PANGRAM"));
    assert!(out_str.contains("--- HINT GRID ---"));
    assert!(out_str.contains("--- WORDS FOUND ---"));
    assert!(out_str.contains("Final Score: 16"));
}

#[test]
fn test_cli_play_session_batch_syntax_error_rejects_line() {
    let puzzle = SpellingBeePuzzle::from_raw("CAIMORT").unwrap();
    let solutions = vec!["ACAI".to_string(), "MICRO".to_string()];
    let session = SpellingBeeSession::new(puzzle, solutions);

    // "ACAI, MIC!RO" has an invalid character '!' in the second token
    let input_script = "ACAI, MIC!RO\n/quit\n";
    let mut reader = Cursor::new(input_script);
    let mut output = Vec::new();

    let res = play_session(session, &mut reader, &mut output, ColorMode::Never);
    assert!(res.is_ok());

    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("Syntax error"));
    // ACAI must not have been accepted before the syntax error
    assert!(!out_str.contains("✔ ACAI"));
}
