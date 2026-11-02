use spelling_bee::{SpellingBeePuzzle, SpellingBeeSession};
use std::io::Cursor;
use std::process::Command;
use word_games_cli::replay::{prompt_replay, ReplayChoice};
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

#[test]
fn test_replay_menu_choices() {
    // Option 1
    let mut r1 = Cursor::new("1\n");
    let mut out1 = Vec::new();
    let c1 = prompt_replay(&mut r1, &mut out1, ColorMode::Never).unwrap();
    assert_eq!(c1, ReplayChoice::SameSettings);

    // Option 2
    let mut r2 = Cursor::new("2\n");
    let mut out2 = Vec::new();
    let c2 = prompt_replay(&mut r2, &mut out2, ColorMode::Never).unwrap();
    assert_eq!(c2, ReplayChoice::Exit);

    // EOF exits cleanly
    let mut r_eof = Cursor::new("");
    let mut out_eof = Vec::new();
    let c_eof = prompt_replay(&mut r_eof, &mut out_eof, ColorMode::Never).unwrap();
    assert_eq!(c_eof, ReplayChoice::Exit);
}

#[test]
fn test_cli_binary_subprocess_help() {
    let bin_path = env!("CARGO_BIN_EXE_word-games-cli");
    let output = Command::new(bin_path)
        .arg("--help")
        .output()
        .expect("run word-games-cli --help");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("word-games-cli — Interactive Word Games Suite"));
    assert!(stderr.contains("spelling-bee"));
}

#[test]
fn test_cli_binary_subprocess_quit() {
    let bin_path = env!("CARGO_BIN_EXE_word-games-cli");
    let dict_paths = [
        "assets/dictionary.txt",
        "../assets/dictionary.txt",
        "word-games/assets/dictionary.txt",
    ];
    let dict_path = dict_paths
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .expect("dictionary exists");

    let mut child = Command::new(bin_path)
        .args([
            "spelling-bee",
            "--dictionary",
            dict_path,
            "--letters",
            "CAIMORT",
            "--center",
            "C",
            "--color",
            "never",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn word-games-cli");

    {
        use std::io::Write;
        let mut stdin = child.stdin.take().expect("get stdin");
        stdin
            .write_all(b"AROMATIC\n/quit\n2\n")
            .expect("write stdin");
    }

    let output = child.wait_with_output().expect("wait child");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("SPELLING BEE"));
    assert!(stdout.contains("✔ AROMATIC"));
    assert!(stdout.contains("PANGRAM"));
    assert!(stdout.contains("Final Score:"));
}
