use std::process::{Command, Stdio};

fn binary_path() -> &'static str {
    env!("CARGO_BIN_EXE_countdown-timer-cli")
}

#[test]
fn test_c01_help_flag() {
    let output = Command::new(binary_path())
        .arg("--help")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("USAGE:"));
    assert!(stdout.contains("--seconds"));
    assert!(stdout.contains("Space"));
}

#[test]
fn test_c02_short_help_flag() {
    let output = Command::new(binary_path())
        .arg("-h")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("USAGE:"));
}

#[test]
fn test_c03_unknown_flag() {
    let output = Command::new(binary_path())
        .arg("--unknown")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown option"));
}

#[test]
fn test_c04_missing_seconds_value() {
    let output = Command::new(binary_path())
        .arg("--seconds")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing value"));
}

#[test]
fn test_c05_zero_seconds() {
    let output = Command::new(binary_path())
        .args(["--seconds", "0"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("out of allowed range"));
}

#[test]
fn test_c06_overflow_86401_seconds() {
    let output = Command::new(binary_path())
        .args(["--seconds", "86401"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("out of allowed range"));
}

#[test]
fn test_c07_negative_seconds() {
    let output = Command::new(binary_path())
        .args(["--seconds", "-1"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("decimal digits") || stderr.contains("unknown option"));
}

#[test]
fn test_c08_fractional_seconds() {
    let output = Command::new(binary_path())
        .args(["--seconds", "0.5"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("decimal digits"));
}

#[test]
fn test_c09_non_digit_seconds() {
    let output = Command::new(binary_path())
        .args(["--seconds", "abc"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("decimal digits"));
}

#[test]
fn test_c10_huge_integer_overflow() {
    let output = Command::new(binary_path())
        .args(["--seconds", "184467440737095516160"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("overflows"));
}

#[test]
fn test_c11_leading_zero_seconds_piped() {
    let output = Command::new(binary_path())
        .args(["--seconds", "01"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    // Accepted as duration 1, then exits with code 1 due to non-TTY
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("interactive terminal required"));
}

#[test]
fn test_c12_max_86400_seconds_piped() {
    let output = Command::new(binary_path())
        .args(["--seconds", "86400"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("interactive terminal required"));
}

#[test]
fn test_c13_non_tty_no_ansi_escapes() {
    let output = Command::new(binary_path())
        .args(["--seconds", "1"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(1));
    // Verify stdout contains NO ANSI escape sequences
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.contains(&b'\x1b'));
}

#[test]
fn test_c14_no_flags_piped_non_tty() {
    let output = Command::new(binary_path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("duration required via --seconds in non-interactive environment"));
}

#[test]
fn test_c15_extra_positional_arguments() {
    let output = Command::new(binary_path())
        .args(["--seconds", "30", "junk"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unexpected argument 'junk'"));
}

#[test]
fn test_c16_repeated_seconds_flag() {
    let output = Command::new(binary_path())
        .args(["--seconds", "30", "--seconds", "60"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute binary");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("duplicate --seconds flag"));
}

#[test]
fn test_c17_invalid_unicode_parsing() {
    #[cfg(unix)]
    {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let invalid_os_str = OsStr::from_bytes(&[b'a', 0xFF, b'b']);
        let result = countdown_timer_cli::parse_args([
            std::ffi::OsString::from("prog"),
            invalid_os_str.to_os_string(),
        ]);
        assert_eq!(result.unwrap_err().code, 2);
    }
}
