//! Subprocess and interactive session integration tests for Wordle CLI.

use std::io::Cursor;
use std::path::Path;
use std::process::Command;
use word_games_cli::terminal::ColorMode;
use word_games_cli::wordle::play_session;
use wordle::feedback::FeedbackStrategy;
use wordle::{Word, WordleGame, WordleLexicon};

fn make_fixture_game(secret_str: &str) -> WordleGame {
    let allowed = vec![
        Word::parse("APPLE").unwrap(),
        Word::parse("CRANE").unwrap(),
        Word::parse("SLATE").unwrap(),
        Word::parse("WATER").unwrap(),
        Word::parse("ROUND").unwrap(),
        Word::parse("LIGHT").unwrap(),
    ];
    let curated = vec![
        Word::parse("CRANE").unwrap(),
        Word::parse("APPLE").unwrap(),
        Word::parse("WATER").unwrap(),
    ];
    let lexicon = WordleLexicon::new_validated(allowed, curated).unwrap();
    let secret = Word::parse(secret_str).unwrap();
    WordleGame::new(lexicon, secret, FeedbackStrategy::Frequency).unwrap()
}

fn find_dictionary_path() -> &'static str {
    let dict_paths = [
        "assets/dictionary.txt",
        "../assets/dictionary.txt",
        "word-games/assets/dictionary.txt",
    ];
    dict_paths
        .into_iter()
        .find(|p| Path::new(p).exists())
        .expect("dictionary exists")
}

#[test]
fn test_wordle_cli_play_session_commands_and_quit() {
    let game = make_fixture_game("CRANE");

    // Guess SLATE, run /keyboard, /history, /help, then /quit
    let input_script = "SLATE\n/keyboard\n/history\n/help\n/quit\n";
    let mut reader = Cursor::new(input_script);
    let mut output = Vec::new();

    let res = play_session(game, &mut reader, &mut output, ColorMode::Never);
    assert!(res.is_ok());
    assert!(res.unwrap()); // true indicates normal completed round

    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("WORDLE"));
    assert!(out_str.contains("[1/6]"));
    assert!(out_str.contains("[..C.C]")); // Pos 2 (A) and Pos 4 (E) are Correct
    assert!(out_str.contains("Keyboard Status:"));
    assert!(out_str.contains("--- PAST GUESSES ---"));
    assert!(out_str.contains("--- WORDLE RULES & INSTRUCTIONS ---"));
    assert!(out_str.contains("ROUND FORFEITED"));
    assert!(out_str.contains("The secret word was: CRANE"));
}

#[test]
fn test_wordle_cli_play_session_win_on_sixth_attempt() {
    let game = make_fixture_game("CRANE");

    // 5 non-winning valid guesses, then correct guess on 6th
    let input_script = "SLATE\nSLATE\nSLATE\nSLATE\nSLATE\nCRANE\n";
    let mut reader = Cursor::new(input_script);
    let mut output = Vec::new();

    let res = play_session(game, &mut reader, &mut output, ColorMode::Never);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("VICTORY!"));
    assert!(out_str.contains("Solved the puzzle in 6/6 attempts!"));
}

#[test]
fn test_wordle_cli_play_session_loss_on_sixth_attempt() {
    let game = make_fixture_game("CRANE");

    // 6 non-winning guesses
    let input_script = "SLATE\nSLATE\nSLATE\nSLATE\nSLATE\nWATER\n";
    let mut reader = Cursor::new(input_script);
    let mut output = Vec::new();

    let res = play_session(game, &mut reader, &mut output, ColorMode::Never);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("GAME OVER"));
    assert!(out_str.contains("Exhausted all 6 attempts without solving the code."));
    assert!(out_str.contains("The secret word was: CRANE"));
}

#[test]
fn test_wordle_cli_invalid_input_does_not_consume_attempt() {
    let game = make_fixture_game("CRANE");

    // Bad length, bad char, not in dict, then winning guess
    let input_script = "CAT\nCR4NE\nZZZZZ\nCRANE\n";
    let mut reader = Cursor::new(input_script);
    let mut output = Vec::new();

    let res = play_session(game, &mut reader, &mut output, ColorMode::Never);
    assert!(res.is_ok());

    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("Invalid guess: Invalid word length"));
    assert!(out_str.contains("Invalid guess: Invalid character"));
    assert!(out_str.contains("Invalid guess: Word 'ZZZZZ' is not in the allowed dictionary"));
    // Solved on attempt 1 because invalid inputs did not consume turns
    assert!(out_str.contains("Solved the puzzle in 1/6 attempts!"));
}

#[test]
fn test_wordle_cli_clean_eof() {
    let game = make_fixture_game("CRANE");

    // Immediate EOF
    let input_script = "";
    let mut reader = Cursor::new(input_script);
    let mut output = Vec::new();

    let res = play_session(game, &mut reader, &mut output, ColorMode::Never);
    assert!(res.is_ok());
    assert!(!res.unwrap()); // returns false indicating EOF exit

    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("EOF received. Exiting game."));
}

// ---------------------------------------------------------------------------
// Subprocess binary tests
// ---------------------------------------------------------------------------

#[test]
fn test_wordle_cli_subprocess_help() {
    let bin = env!("CARGO_BIN_EXE_word-games-cli");
    let output = Command::new(bin)
        .arg("wordle")
        .arg("--help")
        .output()
        .expect("executes word-games-cli wordle --help");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("WORDLE OPTIONS:"));
    assert!(stderr.contains("--answers <PATH>"));
    assert!(stderr.contains("--feedback <STRATEGY>"));
}

#[test]
fn test_wordle_cli_subprocess_play_and_quit() {
    use std::io::Write;
    use std::process::Stdio;

    let bin = env!("CARGO_BIN_EXE_word-games-cli");
    let dict = find_dictionary_path();

    let mut child = Command::new(bin)
        .arg("wordle")
        .arg("--dictionary")
        .arg(dict)
        .arg("--color")
        .arg("never")
        .arg("--seed")
        .arg("42")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn word-games-cli wordle");

    {
        let stdin = child.stdin.as_mut().expect("stdin");
        stdin.write_all(b"/quit\n2\n").expect("write input");
    }

    let output = child.wait_with_output().expect("wait on child");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("WORDLE"));
    assert!(stdout.contains("ROUND FORFEITED"));
    assert!(stdout.contains("The secret word was:"));
}

#[test]
fn test_wordle_cli_subprocess_invalid_answers_override() {
    let bin = env!("CARGO_BIN_EXE_word-games-cli");
    let dict = find_dictionary_path();

    let output = Command::new(bin)
        .arg("wordle")
        .arg("--dictionary")
        .arg(dict)
        .arg("--answers")
        .arg("nonexistent_answers_file.txt")
        .output()
        .expect("executes wordle with missing answers file");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Failed to read file"));
}
