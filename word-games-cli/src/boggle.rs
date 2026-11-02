//! Interactive terminal driver for Boggle.

use boggle::board::Board;
use boggle::session::{BoggleSession, GuessOutcome, QuitSummary};
use std::io::{BufRead, Write};

use crate::terminal::ColorMode;

/// Runs a single interactive session of Boggle reading from `input` and writing to `output`.
pub fn play_session<R: BufRead, W: Write>(
    mut session: BoggleSession,
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
        color.bold("                 BOGGLE                 ")
    )?;
    writeln!(
        output,
        "{}",
        color.bold("========================================")
    )?;
    writeln!(output)?;

    print_board_display(session.board(), output, color)?;
    writeln!(
        output,
        "Commands: /help (rules), /board (show grid), /guesses (found words), /quit (forfeit)"
    )?;
    writeln!(output)?;

    let mut line_buf = String::new();

    while !session.is_finished() {
        write!(
            output,
            "[Score: {} | Words: {}] Enter guess: ",
            color.yellow(&session.cached_score().to_string()),
            color.cyan(&session.accepted_guesses().len().to_string())
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
                "/board" | "/grid" => {
                    writeln!(output)?;
                    print_board_display(session.board(), output, color)?;
                }
                "/guesses" | "/words" => {
                    writeln!(output)?;
                    writeln!(output, "{}", color.bold("--- WORDS FOUND ---"))?;
                    let words: Vec<&str> = session
                        .accepted_guesses()
                        .iter()
                        .map(|s| s.as_str())
                        .collect();
                    print_words_formatted(&words, output)?;
                    writeln!(output)?;
                }
                "/quit" => {
                    let summary = session.quit();
                    print_quit_summary(&summary, output, color)?;
                    return Ok(true);
                }
                other => {
                    writeln!(
                        output,
                        "{}",
                        color.red(&format!(
                            "Unknown command '{other}'. Type /help for options."
                        ))
                    )?;
                }
            }
            continue;
        }

        // Guess input: could be a single word or comma/space-delimited batch
        let tokens: Vec<&str> = trimmed
            .split(|c: char| c == ',' || c.is_ascii_whitespace())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if tokens.is_empty() {
            continue;
        }

        // Batch pre-validation: check each token for non-alphabetic syntax before mutating
        let mut malformed_token = None;
        for &tok in &tokens {
            if let Some(ch) = tok.chars().find(|c| !c.is_ascii_alphabetic()) {
                malformed_token = Some((tok, ch));
                break;
            }
        }

        if let Some((tok, ch)) = malformed_token {
            writeln!(
                output,
                "{}",
                color.red(&format!(
                    "Syntax error in '{tok}': invalid character '{ch}'. Entire line rejected."
                ))
            )?;
            continue;
        }

        // Evaluate tokens sequentially
        for tok in tokens {
            if session.is_finished() {
                writeln!(
                    output,
                    "{}",
                    color.dim(&format!("Round ended; ignoring remaining token '{tok}'."))
                )?;
                break;
            }

            match session.submit_guess(tok) {
                GuessOutcome::Accepted {
                    word,
                    points,
                    is_complete,
                    ..
                } => {
                    writeln!(
                        output,
                        "{} (+{} pts)",
                        color.green(&format!("  ✔ {word}")),
                        points
                    )?;

                    if is_complete {
                        writeln!(output)?;
                        writeln!(
                            output,
                            "{}",
                            color.bold(
                                &color.yellow(
                                    "🎉 CONGRATULATIONS! You found ALL words on this board!"
                                )
                            )
                        )?;
                        let summary = session.quit();
                        print_quit_summary(&summary, output, color)?;
                        return Ok(true);
                    }
                }
                GuessOutcome::Duplicate { word } => {
                    writeln!(
                        output,
                        "{}",
                        color.yellow(&format!("  ℹ Already found '{word}'."))
                    )?;
                }
                GuessOutcome::TooShort {
                    word, min_length, ..
                } => {
                    writeln!(
                        output,
                        "{}",
                        color.red(&format!(
                            "  ✘ '{word}' is too short (minimum {min_length} letters)."
                        ))
                    )?;
                }
                GuessOutcome::InvalidWord { word, reason } => {
                    writeln!(
                        output,
                        "{}",
                        color.red(&format!("  ✘ '{word}' is invalid: {reason}."))
                    )?;
                }
                GuessOutcome::NotASolution { word } => {
                    writeln!(
                        output,
                        "{}",
                        color.red(&format!("  ✘ '{word}' is not a solution on this board."))
                    )?;
                }
                GuessOutcome::GameFinished => {
                    writeln!(output, "{}", color.dim("  ℹ Game is already finished."))?;
                    break;
                }
            }
        }
    }

    Ok(true)
}

fn print_board_display<W: Write>(
    board: &Board,
    output: &mut W,
    color: ColorMode,
) -> std::io::Result<()> {
    writeln!(
        output,
        "{}",
        color.bold(&format!(
            "--- BOARD ({}x{}) ---",
            board.size(),
            board.size()
        ))
    )?;
    writeln!(output, "{}", board.format_grid())?;
    writeln!(output)?;
    Ok(())
}

fn print_help<W: Write>(output: &mut W, color: ColorMode) -> std::io::Result<()> {
    writeln!(output)?;
    writeln!(output, "{}", color.bold("--- BOGGLE RULES & COMMANDS ---"))?;
    writeln!(output, "• Find words of at least 3 letters on the board.")?;
    writeln!(
        output,
        "• Each letter must be an adjacent neighbor (horizontal, vertical, diagonal) of the previous."
    )?;
    writeln!(
        output,
        "• You may NOT use the same board cell more than once in the same word."
    )?;
    writeln!(output, "• 'Qu' counts as 2 letters while occupying 1 cell.")?;
    writeln!(
        output,
        "• Scoring: 3-4 letters: 1 pt | 5: 2 pts | 6: 3 pts | 7: 5 pts | 8: 11 pts | 9-12: 2x | 13-14: 3x | >=15: 4x"
    )?;
    writeln!(
        output,
        "• You can enter guesses one per line or comma/space separated."
    )?;
    writeln!(output, "• Commands:")?;
    writeln!(output, "    /board    Redisplay the current board")?;
    writeln!(output, "    /guesses  List all accepted words found so far")?;
    writeln!(output, "    /help     Show this help message")?;
    writeln!(
        output,
        "    /quit     End the round and reveal missed words"
    )?;
    writeln!(output)?;
    Ok(())
}

fn print_quit_summary<W: Write>(
    summary: &QuitSummary,
    output: &mut W,
    color: ColorMode,
) -> std::io::Result<()> {
    writeln!(output)?;
    writeln!(output, "{}", color.bold("=== ROUND SUMMARY ==="))?;
    writeln!(
        output,
        "Final Score: {} / {} possible points",
        color.yellow(&summary.score.to_string()),
        summary.total_possible_score
    )?;
    writeln!(
        output,
        "Words Found: {} / {} total solutions",
        color.cyan(&summary.found_count.to_string()),
        summary.total_solutions
    )?;
    writeln!(output)?;

    if !summary.missed_words.is_empty() {
        writeln!(
            output,
            "{}",
            color.bold(&format!(
                "--- MISSED WORDS ({}) ---",
                summary.missed_words.len()
            ))
        )?;
        let missed_refs: Vec<&str> = summary.missed_words.iter().map(|s| s.as_str()).collect();
        print_words_formatted(&missed_refs, output)?;
        writeln!(output)?;
    }

    Ok(())
}

/// Formats a list of words across multiple columns for readable display.
fn print_words_formatted<W: Write>(words: &[&str], output: &mut W) -> std::io::Result<()> {
    if words.is_empty() {
        writeln!(output, "  (none)")?;
        return Ok(());
    }

    let col_width = 16;
    let num_cols = 4;

    for chunk in words.chunks(num_cols) {
        write!(output, "  ")?;
        for w in chunk {
            let pad = if w.len() < col_width {
                col_width - w.len()
            } else {
                1
            };
            write!(output, "{}{}", w, " ".repeat(pad))?;
        }
        writeln!(output)?;
    }
    Ok(())
}
