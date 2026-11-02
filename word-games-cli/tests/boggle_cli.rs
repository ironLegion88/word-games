//! Subprocess and interactive session integration tests for Boggle CLI.

use boggle::board::Board;
use boggle::session::BoggleSession;
use boggle::solver::SolverStrategy;
use std::io::Cursor;
use std::process::Command;
use word_games_cli::boggle::play_session;
use word_games_cli::terminal::ColorMode;

fn make_fixture_session() -> BoggleSession {
    let board = Board::from_str_tiles(
        4,
        "
        C A T S
        D O G S
        Qu R E W
        B I R D
    ",
    )
    .unwrap();
    let solutions = vec![
        "BIRD".to_string(),
        "CATS".to_string(),
        "COED".to_string(),
        "DOGS".to_string(),
    ];
    BoggleSession::new(board, solutions, SolverStrategy::Trie).unwrap()
}

#[test]
fn test_boggle_cli_play_session_batch_and_quit() {
    let session = make_fixture_session();

    // Comma-separated guesses, /board, /guesses, /help, then /quit
    let input_script = "CATS, DOGS\n/board\n/guesses\n/help\n/quit\n";
    let mut reader = Cursor::new(input_script);
    let mut output = Vec::new();

    let res = play_session(session, &mut reader, &mut output, ColorMode::Never);
    assert!(res.is_ok());

    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("✔ CATS"));
    assert!(out_str.contains("✔ DOGS"));
    assert!(out_str.contains("--- BOARD (4x4) ---"));
    assert!(out_str.contains("--- WORDS FOUND ---"));
    assert!(out_str.contains("--- BOGGLE RULES & COMMANDS ---"));
    assert!(out_str.contains("=== ROUND SUMMARY ==="));
    assert!(out_str.contains("Final Score: 2"));
    assert!(out_str.contains("Words Found: 2 / 4"));
    assert!(out_str.contains("--- MISSED WORDS (2) ---"));
    assert!(out_str.contains("BIRD"));
    assert!(out_str.contains("COED"));
}

#[test]
fn test_boggle_cli_batch_syntax_error_atomicity() {
    let session = make_fixture_session();

    // "CATS, DO!GS" has '!' in second token
    let input_script = "CATS, DO!GS\n/quit\n";
    let mut reader = Cursor::new(input_script);
    let mut output = Vec::new();

    let res = play_session(session, &mut reader, &mut output, ColorMode::Never);
    assert!(res.is_ok());

    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("Syntax error"));
    // CATS must not have been accepted before the syntax error was detected
    assert!(!out_str.contains("✔ CATS"));
}

#[test]
fn test_boggle_cli_clean_eof() {
    let session = make_fixture_session();

    // Immediate EOF
    let input_script = "";
    let mut reader = Cursor::new(input_script);
    let mut output = Vec::new();

    let res = play_session(session, &mut reader, &mut output, ColorMode::Never);
    assert!(res.is_ok());
    assert!(!res.unwrap()); // returns false indicating EOF/exit

    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("EOF received"));
}

#[test]
fn test_boggle_cli_all_words_accolade() {
    let session = make_fixture_session();

    // Guess all 4 words in batch
    let input_script = "BIRD, CATS, COED, DOGS\n";
    let mut reader = Cursor::new(input_script);
    let mut output = Vec::new();

    let res = play_session(session, &mut reader, &mut output, ColorMode::Never);
    assert!(res.is_ok());

    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("CONGRATULATIONS"));
    assert!(out_str.contains("Words Found: 4 / 4"));
}

#[test]
fn test_boggle_cli_subprocess_help() {
    let bin_path = env!("CARGO_BIN_EXE_word-games-cli");
    let output = Command::new(bin_path)
        .arg("--help")
        .output()
        .expect("run word-games-cli --help");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("boggle"));
    assert!(stderr.contains("--size <4|5|6>"));
    assert!(stderr.contains("--generator <GEN>"));
}

#[test]
fn test_boggle_cli_subprocess_fixed_board_and_quit() {
    let bin_path = env!("CARGO_BIN_EXE_word-games-cli");
    let mut child = Command::new(bin_path)
        .arg("boggle")
        .arg("--board")
        .arg("CATSDOGSQREWBIRD")
        .arg("--color")
        .arg("never")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn word-games-cli boggle");

    use std::io::Write;
    {
        let stdin = child.stdin.as_mut().expect("stdin pipe");
        stdin.write_all(b"CATS\n/quit\n2\n").expect("write input");
    }

    let output = child.wait_with_output().expect("wait on child");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✔ CATS"));
    assert!(stdout.contains("=== ROUND SUMMARY ==="));
}
