/// Supported REPL commands for `trie-cli`.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    New,
    Load(String),
    AddFile(String),
    Insert(String),
    Contains(String),
    Remove(String),
    RemoveFile(String),
    Present(String),
    Absent(String),
    Normalize(String),
    Stats,
    Quit,
}

/// Errors that can occur when parsing a command line in the REPL.
#[derive(Debug, PartialEq, Eq)]
pub enum CommandError {
    EmptyLine,
    UnknownCommand(String),
    MissingArgument {
        command: &'static str,
        expected: &'static str,
    },
    TooManyArguments {
        command: &'static str,
        unexpected: String,
    },
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyLine => write!(f, "empty command line"),
            Self::UnknownCommand(cmd) => {
                write!(
                    f,
                    "unknown command '{cmd}'. Type 'help' for available commands."
                )
            }
            Self::MissingArgument { command, expected } => {
                write!(f, "missing argument for '{command}': expected <{expected}>")
            }
            Self::TooManyArguments {
                command,
                unexpected,
            } => {
                write!(
                    f,
                    "too many arguments for '{command}': unexpected '{unexpected}'"
                )
            }
        }
    }
}

impl std::error::Error for CommandError {}

/// Parses a raw user input line into a `Command`.
pub fn parse_command(line: &str) -> Result<Command, CommandError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err(CommandError::EmptyLine);
    }

    let mut parts = trimmed.splitn(2, |c: char| c.is_ascii_whitespace());
    let cmd_name = parts.next().unwrap_or("").to_ascii_lowercase();
    let rest = parts.next().map(|s| s.trim()).unwrap_or("");

    match cmd_name.as_str() {
        "help" => {
            if !rest.is_empty() {
                return Err(CommandError::TooManyArguments {
                    command: "help",
                    unexpected: rest.to_string(),
                });
            }
            Ok(Command::Help)
        }
        "new" => {
            if !rest.is_empty() {
                return Err(CommandError::TooManyArguments {
                    command: "new",
                    unexpected: rest.to_string(),
                });
            }
            Ok(Command::New)
        }
        "load" => {
            let path = parse_path_arg("load", rest)?;
            Ok(Command::Load(path))
        }
        "add-file" | "add_file" => {
            let path = parse_path_arg("add-file", rest)?;
            Ok(Command::AddFile(path))
        }
        "insert" => {
            let word = parse_single_word_arg("insert", rest)?;
            Ok(Command::Insert(word))
        }
        "contains" => {
            let word = parse_single_word_arg("contains", rest)?;
            Ok(Command::Contains(word))
        }
        "remove" => {
            let word = parse_single_word_arg("remove", rest)?;
            Ok(Command::Remove(word))
        }
        "remove-file" | "remove_file" => {
            let path = parse_path_arg("remove-file", rest)?;
            Ok(Command::RemoveFile(path))
        }
        "present" => {
            let path = parse_path_arg("present", rest)?;
            Ok(Command::Present(path))
        }
        "absent" => {
            let path = parse_path_arg("absent", rest)?;
            Ok(Command::Absent(path))
        }
        "normalize" => {
            let word = parse_single_word_arg("normalize", rest)?;
            Ok(Command::Normalize(word))
        }
        "stats" => {
            if !rest.is_empty() {
                return Err(CommandError::TooManyArguments {
                    command: "stats",
                    unexpected: rest.to_string(),
                });
            }
            Ok(Command::Stats)
        }
        "quit" | "exit" => {
            if !rest.is_empty() {
                return Err(CommandError::TooManyArguments {
                    command: "quit",
                    unexpected: rest.to_string(),
                });
            }
            Ok(Command::Quit)
        }
        _ => Err(CommandError::UnknownCommand(cmd_name)),
    }
}

/// Helper to parse a path argument, stripping surrounding quotes if present.
fn parse_path_arg(cmd: &'static str, rest: &str) -> Result<String, CommandError> {
    if rest.is_empty() {
        return Err(CommandError::MissingArgument {
            command: cmd,
            expected: "path",
        });
    }

    let unquoted = if (rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2)
        || (rest.starts_with('\'') && rest.ends_with('\'') && rest.len() >= 2)
    {
        &rest[1..rest.len() - 1]
    } else {
        rest
    };

    if unquoted.is_empty() {
        return Err(CommandError::MissingArgument {
            command: cmd,
            expected: "path",
        });
    }

    Ok(unquoted.to_string())
}

/// Helper to parse a single word argument.
fn parse_single_word_arg(cmd: &'static str, rest: &str) -> Result<String, CommandError> {
    if rest.is_empty() {
        return Err(CommandError::MissingArgument {
            command: cmd,
            expected: "word",
        });
    }

    let mut words = rest.split_ascii_whitespace();
    let word = words.next().unwrap();
    if let Some(extra) = words.next() {
        return Err(CommandError::TooManyArguments {
            command: cmd,
            unexpected: extra.to_string(),
        });
    }

    Ok(word.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_commands() {
        assert_eq!(parse_command("help").unwrap(), Command::Help);
        assert_eq!(parse_command("HELP").unwrap(), Command::Help);
        assert_eq!(parse_command("new").unwrap(), Command::New);
        assert_eq!(parse_command("stats").unwrap(), Command::Stats);
        assert_eq!(parse_command("quit").unwrap(), Command::Quit);
        assert_eq!(parse_command("exit").unwrap(), Command::Quit);

        assert_eq!(
            parse_command("insert apple").unwrap(),
            Command::Insert("apple".into())
        );
        assert_eq!(
            parse_command("contains cat").unwrap(),
            Command::Contains("cat".into())
        );
        assert_eq!(
            parse_command("remove DOG").unwrap(),
            Command::Remove("DOG".into())
        );
        assert_eq!(
            parse_command("normalize bIrD").unwrap(),
            Command::Normalize("bIrD".into())
        );

        assert_eq!(
            parse_command("load dict.txt").unwrap(),
            Command::Load("dict.txt".into())
        );
        assert_eq!(
            parse_command("load \"path with spaces/dict.txt\"").unwrap(),
            Command::Load("path with spaces/dict.txt".into())
        );
        assert_eq!(
            parse_command("add-file list.txt").unwrap(),
            Command::AddFile("list.txt".into())
        );
        assert_eq!(
            parse_command("remove-file del.txt").unwrap(),
            Command::RemoveFile("del.txt".into())
        );
        assert_eq!(
            parse_command("present check.txt").unwrap(),
            Command::Present("check.txt".into())
        );
        assert_eq!(
            parse_command("absent check.txt").unwrap(),
            Command::Absent("check.txt".into())
        );
    }

    #[test]
    fn parse_errors() {
        assert_eq!(parse_command(""), Err(CommandError::EmptyLine));
        assert_eq!(parse_command("   "), Err(CommandError::EmptyLine));
        assert_eq!(
            parse_command("foobar"),
            Err(CommandError::UnknownCommand("foobar".into()))
        );
        assert_eq!(
            parse_command("insert"),
            Err(CommandError::MissingArgument {
                command: "insert",
                expected: "word"
            })
        );
        assert_eq!(
            parse_command("load"),
            Err(CommandError::MissingArgument {
                command: "load",
                expected: "path"
            })
        );
        assert_eq!(
            parse_command("insert foo bar"),
            Err(CommandError::TooManyArguments {
                command: "insert",
                unexpected: "bar".into()
            })
        );
    }
}
