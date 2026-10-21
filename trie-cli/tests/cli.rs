use std::io::Write;
use std::process::{Command, Stdio};

fn run_cli_session(input: &str, args: &[&str]) -> (String, String, bool) {
    let bin_path = env!("CARGO_BIN_EXE_trie-cli");
    let mut child = Command::new(bin_path)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn trie-cli");

    {
        let stdin = child.stdin.as_mut().expect("failed to open stdin");
        stdin
            .write_all(input.as_bytes())
            .expect("failed to write to stdin");
    }

    let output = child.wait_with_output().expect("failed to wait on child");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

#[test]
fn cli_help_flag() {
    let (stdout, _, success) = run_cli_session("", &["--help"]);
    assert!(success);
    assert!(stdout.contains("Usage: trie-cli"));
}

#[test]
fn cli_nonexistent_dictionary_startup_fails() {
    let (_, stderr, success) = run_cli_session("", &["--dictionary", "nonexistent_file_xyz.txt"]);
    assert!(!success);
    assert!(stderr.contains("Failed to load initial dictionary"));
}

#[test]
fn cli_interactive_basic_operations() {
    let script = "\
help
insert CAT
contains cat
contains dog
stats
normalize bIrD
remove cat
contains cat
quit
";
    let (stdout, _, success) = run_cli_session(script, &[]);
    assert!(success);
    assert!(stdout.contains("Available Commands:"));
    assert!(stdout.contains("Inserted \"CAT\""));
    assert!(stdout.contains("true: \"CAT\" is in the dictionary."));
    assert!(stdout.contains("false: \"DOG\" is NOT in the dictionary."));
    assert!(stdout.contains("Dictionary Statistics:"));
    assert!(stdout.contains("Normalized: BIRD"));
    assert!(stdout.contains("Removed \"CAT\""));
    assert!(stdout.contains("false: \"CAT\" is NOT in the dictionary."));
    assert!(stdout.contains("Goodbye!"));
}

#[test]
fn cli_file_operations_and_filtering() {
    let temp_dir = std::env::temp_dir();
    let dict_file = temp_dir.join("test_cli_dict.txt");
    let test_file = temp_dir.join("test_cli_words.txt");

    std::fs::write(&dict_file, "APPLE\nBANANA\nCHERRY\n").unwrap();
    std::fs::write(&test_file, "apple, DOG, banana, CAT, cherry, FOX").unwrap();

    let dict_str = dict_file.to_str().unwrap().replace('\\', "/");
    let test_str = test_file.to_str().unwrap().replace('\\', "/");

    let script = format!(
        "\
load \"{dict_str}\"
contains apple
contains banana
present \"{test_str}\"
absent \"{test_str}\"
new
stats
quit
"
    );

    let (stdout, _, success) = run_cli_session(&script, &[]);
    assert!(success);
    assert!(stdout.contains("Loaded 3 words"));
    assert!(stdout.contains("true: \"APPLE\" is in the dictionary."));
    assert!(stdout.contains("true: \"BANANA\" is in the dictionary."));
    assert!(stdout.contains("Words present in trie (3):"));
    assert!(stdout.contains("Words absent from trie (3):"));
    assert!(stdout.contains("Created a new empty dictionary."));
    assert!(stdout.contains("Words:            0"));

    let _ = std::fs::remove_file(dict_file);
    let _ = std::fs::remove_file(test_file);
}

#[test]
fn cli_handles_eof_gracefully() {
    // Send empty input to simulate immediate EOF (Ctrl-D)
    let (stdout, _, success) = run_cli_session("", &[]);
    assert!(success);
    assert!(stdout.contains("Word Trie Interactive CLI"));
}

#[test]
fn cli_friendly_errors_on_malformed_input() {
    let script = "\
unknown_command
insert
contains
insert bad-word
insert word with space
quit
";
    let (stdout, _, success) = run_cli_session(script, &[]);
    assert!(success);
    assert!(stdout.contains("unknown command 'unknown_command'"));
    assert!(stdout.contains("missing argument for 'insert'"));
    assert!(stdout.contains("missing argument for 'contains'"));
    assert!(stdout.contains("invalid character '-'"));
    assert!(stdout.contains("too many arguments for 'insert'"));
}
