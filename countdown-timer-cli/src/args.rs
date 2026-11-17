use std::ffi::OsString;
use std::io::{BufRead, Write};
use std::time::Duration;

/// Minimum allowed countdown duration in seconds.
pub const MIN_SECONDS: u64 = 1;
/// Maximum allowed countdown duration in seconds (24 hours).
pub const MAX_SECONDS: u64 = 86400;

/// Error encountered while parsing command-line arguments or duration input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgError {
    /// Diagnostic error message.
    pub message: String,
    /// Process exit code associated with this error (1 for environment, 2 for usage/input).
    pub code: u8,
}

impl ArgError {
    /// Creates a usage error (exit code 2).
    pub fn usage(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            code: 2,
        }
    }

    /// Creates an environment/terminal error (exit code 1).
    pub fn env_error(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            code: 1,
        }
    }
}

impl std::fmt::Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ArgError {}

/// Result of parsing command-line arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArgs {
    /// True if `--help` or `-h` was requested.
    pub help: bool,
    /// Optional explicitly specified countdown duration in seconds.
    pub seconds: Option<u64>,
}

/// Returns the formatted command-line usage and help documentation.
#[must_use]
pub fn help_text() -> &'static str {
    "Countdown Timer CLI — Standalone interactive terminal countdown timer\n\n\
USAGE:\n    \
countdown-timer-cli [OPTIONS]\n\n\
OPTIONS:\n    \
-s, --seconds <SECONDS>    Set countdown duration in seconds (1..=86400)\n    \
-h, --help                 Print this help information and exit\n\n\
INTERACTIVE CONTROLS:\n    \
Space                      Start / Pause / Resume\n    \
r, R                       Reset timer to initial duration\n    \
q, Q, Esc, Ctrl+C          Quit\n"
}

/// Validates and parses a string slice containing a decimal seconds duration.
///
/// Leading zeros are accepted (e.g. `01` parses as 1). Signs, fractions, whitespace,
/// and values outside `1..=86400` are rejected.
///
/// # Errors
///
/// Returns an explanatory error string if parsing or range validation fails.
pub fn parse_seconds_str(s: &str) -> Result<u64, String> {
    if s.is_empty() {
        return Err("duration cannot be empty".to_string());
    }

    if !s.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!(
            "invalid duration '{s}': must contain only decimal digits (1..={MAX_SECONDS})"
        ));
    }

    let val = s
        .parse::<u64>()
        .map_err(|_| format!("duration '{s}' overflows maximum supported value"))?;

    if !(MIN_SECONDS..=MAX_SECONDS).contains(&val) {
        return Err(format!(
            "duration {val} out of allowed range: must be between {MIN_SECONDS} and {MAX_SECONDS} seconds"
        ));
    }

    Ok(val)
}

/// Parses command-line arguments from any iterator of `OsString`-convertible items.
///
/// # Errors
///
/// Returns [`ArgError`] with exit code 2 if unknown flags, missing values, duplicate flags,
/// or invalid durations are encountered.
pub fn parse_args<I, T>(args: I) -> Result<ParsedArgs, ArgError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let mut iter = args.into_iter();
    // Skip program name if present
    let _prog_name = iter.next();

    let mut help = false;
    let mut seconds: Option<u64> = None;

    while let Some(item) = iter.next() {
        let os_str: OsString = item.into();
        let arg = os_str.to_str().ok_or_else(|| ArgError {
            message: "invalid non-UTF-8 command-line argument".to_string(),
            code: 2,
        })?;

        match arg {
            "-h" | "--help" => {
                help = true;
            }
            "-s" | "--seconds" => {
                if seconds.is_some() {
                    return Err(ArgError::usage("duplicate --seconds flag specified"));
                }
                let next_item = iter
                    .next()
                    .ok_or_else(|| ArgError::usage("missing value for --seconds flag"))?;
                let val_os: OsString = next_item.into();
                let val_str = val_os.to_str().ok_or_else(|| ArgError {
                    message: "invalid non-UTF-8 value for --seconds".to_string(),
                    code: 2,
                })?;
                let secs = parse_seconds_str(val_str).map_err(ArgError::usage)?;
                seconds = Some(secs);
            }
            other => {
                if let Some(val_str) = other.strip_prefix("--seconds=") {
                    if seconds.is_some() {
                        return Err(ArgError::usage("duplicate --seconds flag specified"));
                    }
                    let secs = parse_seconds_str(val_str).map_err(ArgError::usage)?;
                    seconds = Some(secs);
                } else if other.starts_with('-') {
                    return Err(ArgError::usage(format!("unknown option '{other}'")));
                } else {
                    return Err(ArgError::usage(format!("unexpected argument '{other}'")));
                }
            }
        }
    }

    Ok(ParsedArgs { help, seconds })
}

/// Prompts the user interactively in line mode for a duration between 1 and 86400 seconds.
///
/// Returns `Ok(Some(Duration))` on valid input, `Ok(None)` on clean EOF, or `Err` on I/O failure.
pub fn prompt_duration<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<Option<Duration>, std::io::Error> {
    let mut line = String::new();
    loop {
        write!(
            writer,
            "Enter countdown duration in seconds ({MIN_SECONDS}..={MAX_SECONDS}): "
        )?;
        writer.flush()?;

        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            // Clean EOF
            return Ok(None);
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match parse_seconds_str(trimmed) {
            Ok(secs) => return Ok(Some(Duration::from_secs(secs))),
            Err(err) => {
                writeln!(writer, "Invalid input: {err}. Please try again.")?;
                writer.flush()?;
            }
        }
    }
}
