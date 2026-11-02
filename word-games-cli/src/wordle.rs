//! Interactive terminal driver for Wordle.

use std::io::{BufRead, Write};
use trie::letter::Letter;
use wordle::feedback::LetterFeedback;
use wordle::keyboard::{KeyStatus, Keyboard};
use wordle::session::{WordleGame, WordleGuessEvaluation, WordleSubmissionResult, MAX_ATTEMPTS};
use wordle::word::WORD_LENGTH;

use crate::terminal::ColorMode;

/// Runs a single interactive session of Wordle reading from `input` and writing to `output`.
///
/// Returns `Ok(true)` if the round reached a normal completion (win, loss, or quit) and can prompt
/// for replay, or `Ok(false)` on clean EOF.
pub fn play_session<R: BufRead, W: Write>(
    mut game: WordleGame,
    input: &mut R,
    output: &mut W,
    color: ColorMode,
) -> std::io::Result<bool> {
    writeln!(output)?;
    writeln!(
        output,
        "{}",
        color.bold("========================================")
    )?;
    writeln!(
        output,
        "{}",
        color.bold("                 WORDLE                 ")
    )?;
    writeln!(
        output,
        "{}",
        color.bold("========================================")
    )?;
    writeln!(
        output,
        "Guess the 5-letter secret word in 6 valid attempts."
    )?;
    writeln!(
        output,
        "Feedback: {} = Correct, {} = Present, {} = Absent",
        color.green("C / Green"),
        color.yellow("P / Yellow"),
        color.dim(". / Gray")
    )?;
    writeln!(
        output,
        "Commands: /help (instructions), /keyboard (letter status), /history (past guesses), /quit (forfeit)"
    )?;
    writeln!(output)?;

    print_keyboard(game.keyboard(), output, color)?;
    writeln!(output)?;

    let mut line_buf = String::new();

    while game.attempts_remaining() > 0 {
        write!(
            output,
            "[Attempt {}/{}] Enter 5-letter guess: ",
            color.cyan(&(game.attempts_used() + 1).to_string()),
            MAX_ATTEMPTS
        )?;
        output.flush()?;

        line_buf.clear();
        let bytes_read = input.read_line(&mut line_buf)?;
        if bytes_read == 0 {
            // EOF reached
            writeln!(output, "\n{}", color.dim("EOF received. Exiting game."))?;
            return Ok(false);
        }

        let trimmed = line_buf.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check for slash commands
        if trimmed.starts_with('/') {
            match trimmed.to_ascii_lowercase().as_str() {
                "/help" => {
                    print_help(output, color)?;
                }
                "/keyboard" | "/keys" => {
                    writeln!(output)?;
                    print_keyboard(game.keyboard(), output, color)?;
                    writeln!(output)?;
                }
                "/history" | "/guesses" => {
                    writeln!(output)?;
                    writeln!(output, "{}", color.bold("--- PAST GUESSES ---"))?;
                    if game.history().is_empty() {
                        writeln!(output, "  (No guesses submitted yet)")?;
                    } else {
                        for eval in game.history() {
                            print_evaluation_row(eval, output, color)?;
                        }
                    }
                    writeln!(output)?;
                }
                "/quit" => {
                    let secret = game.quit().expect("in-progress quit succeeds");
                    writeln!(output)?;
                    writeln!(
                        output,
                        "{}",
                        color.bold("========================================")
                    )?;
                    writeln!(
                        output,
                        "{}",
                        color.red("             ROUND FORFEITED            ")
                    )?;
                    writeln!(
                        output,
                        "{}",
                        color.bold("========================================")
                    )?;
                    writeln!(output, "You quit after {} attempts.", game.attempts_used())?;
                    writeln!(
                        output,
                        "The secret word was: {}",
                        color.bold(&color.yellow(&secret.as_string()))
                    )?;
                    return Ok(true);
                }
                other => {
                    writeln!(
                        output,
                        "{}",
                        color.red(&format!(
                            "Unknown command '{other}'. Commands: /help, /keyboard, /history, /quit"
                        ))
                    )?;
                }
            }
            continue;
        }

        // Normal word guess submission
        match game.submit_guess(trimmed) {
            Ok(WordleSubmissionResult::Evaluated(eval)) => {
                print_evaluation_row(&eval, output, color)?;
                writeln!(output)?;
                print_keyboard(game.keyboard(), output, color)?;
                writeln!(output)?;
            }
            Ok(WordleSubmissionResult::Won(eval)) => {
                print_evaluation_row(&eval, output, color)?;
                writeln!(output)?;
                writeln!(
                    output,
                    "{}",
                    color.bold("========================================")
                )?;
                writeln!(
                    output,
                    "{}",
                    color.bold(&color.green("                 VICTORY!               "))
                )?;
                writeln!(
                    output,
                    "{}",
                    color.bold("========================================")
                )?;
                writeln!(
                    output,
                    "Magnificent! Solved the puzzle in {}/{} attempts!",
                    color.bold(&color.green(&eval.attempt_number.to_string())),
                    MAX_ATTEMPTS
                )?;
                return Ok(true);
            }
            Ok(WordleSubmissionResult::Lost(eval, secret)) => {
                print_evaluation_row(&eval, output, color)?;
                writeln!(output)?;
                writeln!(
                    output,
                    "{}",
                    color.bold("========================================")
                )?;
                writeln!(
                    output,
                    "{}",
                    color.bold(&color.red("                GAME OVER               "))
                )?;
                writeln!(
                    output,
                    "{}",
                    color.bold("========================================")
                )?;
                writeln!(
                    output,
                    "Exhausted all {} attempts without solving the code.",
                    MAX_ATTEMPTS
                )?;
                writeln!(
                    output,
                    "The secret word was: {}",
                    color.bold(&color.yellow(&secret.as_string()))
                )?;
                return Ok(true);
            }
            Err(e) => {
                writeln!(output, "{}", color.red(&format!("Invalid guess: {e}")))?;
            }
        }
    }

    Ok(true)
}

fn print_evaluation_row<W: Write>(
    eval: &WordleGuessEvaluation,
    output: &mut W,
    color: ColorMode,
) -> std::io::Result<()> {
    write!(output, "  [{}/{}]  ", eval.attempt_number, MAX_ATTEMPTS)?;
    for i in 0..WORD_LENGTH {
        let ch = eval.guess[i].to_char();
        let mark = eval.feedback[i];
        if color.is_active() {
            match mark {
                LetterFeedback::Correct => {
                    write!(output, "{}", color.bold(&color.green(&format!(" {ch} "))))?
                }
                LetterFeedback::Present => {
                    write!(output, "{}", color.bold(&color.yellow(&format!(" {ch} "))))?
                }
                LetterFeedback::Absent => write!(output, "{}", color.dim(&format!(" {ch} ")))?,
            }
        } else {
            write!(output, "{}{}", ch, mark.tag())?;
            if i + 1 < WORD_LENGTH {
                write!(output, " ")?;
            }
        }
    }

    // Append bracketed tags for plain text readability and clarity
    let tags: String = eval.feedback.iter().map(|m| m.tag()).collect();
    writeln!(output, "   [{tags}]")?;

    Ok(())
}

fn print_keyboard<W: Write>(
    keyboard: &Keyboard,
    output: &mut W,
    color: ColorMode,
) -> std::io::Result<()> {
    writeln!(output, "{}", color.bold("Keyboard Status:"))?;
    for (r, &row) in Keyboard::QWERTY_ROWS.iter().enumerate() {
        write!(output, "  ")?;
        if r == 1 {
            write!(output, " ")?;
        } else if r == 2 {
            write!(output, "   ")?;
        }

        for ch in row.chars() {
            let letter = Letter::from_char(ch).unwrap();
            let status = keyboard.status(letter);
            if color.is_active() {
                match status {
                    KeyStatus::Unknown => write!(output, "{} ", color.dim(&ch.to_string()))?,
                    KeyStatus::Absent => {
                        write!(output, "{} ", color.style("90", &format!("{ch}.")))?
                    }
                    KeyStatus::Present => {
                        write!(output, "{} ", color.bold(&color.yellow(&format!("{ch}?"))))?
                    }
                    KeyStatus::Correct => {
                        write!(output, "{} ", color.bold(&color.green(&format!("{ch}!"))))?
                    }
                }
            } else {
                match status {
                    KeyStatus::Unknown => write!(output, "{ch}  ")?,
                    KeyStatus::Absent => write!(output, "{ch}. ")?,
                    KeyStatus::Present => write!(output, "{ch}? ")?,
                    KeyStatus::Correct => write!(output, "{ch}! ")?,
                }
            }
        }
        writeln!(output)?;
    }
    Ok(())
}

fn print_help<W: Write>(output: &mut W, color: ColorMode) -> std::io::Result<()> {
    writeln!(output)?;
    writeln!(
        output,
        "{}",
        color.bold("--- WORDLE RULES & INSTRUCTIONS ---")
    )?;
    writeln!(
        output,
        "1. Guess the hidden 5-letter English word in 6 attempts."
    )?;
    writeln!(
        output,
        "2. Each guess must be a valid 5-letter word found in the dictionary."
    )?;
    writeln!(
        output,
        "3. Malformed words or non-dictionary entries do NOT consume an attempt."
    )?;
    writeln!(
        output,
        "4. Repeated valid guesses ARE allowed and DO consume an attempt."
    )?;
    writeln!(output, "5. Positional feedback:")?;
    writeln!(
        output,
        "   - {} [C]: Letter is in the exact correct slot.",
        color.green("Correct")
    )?;
    writeln!(
        output,
        "   - {} [P]: Letter is present in the word, but in a different slot.",
        color.yellow("Present")
    )?;
    writeln!(
        output,
        "   - {} [.]: Letter does not appear in any remaining slot.",
        color.dim("Absent ")
    )?;
    writeln!(
        output,
        "6. Slash commands: /help, /keyboard, /history, /quit"
    )?;
    writeln!(output)?;
    Ok(())
}
