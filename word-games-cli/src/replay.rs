//! Replay menu parsing and prompt handling.
//!
//! Adapted from Mastermind `src/game.rs` and `src/ui.rs` (MM-11, MM-29).

use crate::terminal::ColorMode;
use std::io::{self, BufRead, Write};

/// User's choice after a game round finishes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReplayChoice {
    /// Play another puzzle with the same configuration.
    SameSettings,
    /// Exit the game cleanly.
    Exit,
}

impl ReplayChoice {
    /// Parses a user input string into a `ReplayChoice`.
    pub fn from_input(input: &str) -> Option<Self> {
        let trimmed = input.trim().to_ascii_lowercase();
        match trimmed.as_str() {
            "1" | "y" | "yes" | "again" | "same" => Some(Self::SameSettings),
            "2" | "n" | "no" | "q" | "quit" | "exit" => Some(Self::Exit),
            _ => None,
        }
    }
}

/// Prompts the user with a replay menu. Returns `ReplayChoice::Exit` on EOF.
pub fn prompt_replay<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    color: ColorMode,
) -> io::Result<ReplayChoice> {
    let mut line = String::new();

    loop {
        writeln!(output)?;
        writeln!(output, "{}", color.bold("What would you like to do next?"))?;
        writeln!(output, "  [1] Play another round with same settings")?;
        writeln!(output, "  [2] Exit")?;
        write!(output, "Choice [1-2]: ")?;
        output.flush()?;

        line.clear();
        let bytes_read = input.read_line(&mut line)?;
        if bytes_read == 0 {
            // EOF reached
            writeln!(output)?;
            return Ok(ReplayChoice::Exit);
        }

        if let Some(choice) = ReplayChoice::from_input(&line) {
            return Ok(choice);
        }

        writeln!(
            output,
            "{}",
            color.red("Invalid choice. Please enter 1 to play again or 2 to exit.")
        )?;
    }
}
