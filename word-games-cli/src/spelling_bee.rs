//! Interactive terminal driver for Spelling Bee.

use crate::terminal::ColorMode;
use spelling_bee::{
    format_three_columns, GuessOutcome, QuitSummary, SolverStrategy, SpellingBeeSession,
};
use std::io::{BufRead, Write};

/// Options passed from CLI for playing Spelling Bee.
#[derive(Clone, Debug)]
pub struct CliSpellingBeeOptions {
    pub color: ColorMode,
    pub solver: SolverStrategy,
}

/// Runs a single interactive session of Spelling Bee reading from `input` and writing to `output`.
pub fn play_session<R: BufRead, W: Write>(
    mut session: SpellingBeeSession,
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
        color.bold("             SPELLING BEE               ")
    )?;
    writeln!(
        output,
        "{}",
        color.bold("========================================")
    )?;
    writeln!(output)?;

    print_puzzle_header(&session, output, color)?;
    writeln!(
        output,
        "Commands: /help (rules), /hints (grid & counts), /guesses (found words), /quit (forfeit)"
    )?;
    writeln!(output)?;

    let mut line_buf = String::new();

    while !session.is_terminated() {
        write!(
            output,
            "[Score: {} | Words: {}] Enter guess: ",
            color.yellow(&session.current_score().to_string()),
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
                "/hints" => {
                    writeln!(output)?;
                    writeln!(output, "{}", color.bold("--- HINT GRID ---"))?;
                    write!(output, "{}", session.hint_grid().format_table())?;
                    writeln!(output)?;
                    writeln!(output, "{}", color.bold("--- TWO-LETTER LIST ---"))?;
                    write!(output, "{}", session.two_letter_counts().format_list())?;
                    writeln!(output)?;
                }
                "/guesses" => {
                    writeln!(output)?;
                    writeln!(output, "{}", color.bold("--- WORDS FOUND ---"))?;
                    write!(
                        output,
                        "{}",
                        session.format_accepted_guesses_three_columns()
                    )?;
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
            if session.is_terminated() {
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
                    is_pangram,
                    queen_bee,
                } => {
                    let pangram_str = if is_pangram {
                        format!(" {}", color.yellow("[PANGRAM! +7]"))
                    } else {
                        String::new()
                    };
                    writeln!(
                        output,
                        "{} (+{} pts){pangram_str}",
                        color.green(&format!("  ✔ {word}")),
                        points
                    )?;

                    if queen_bee {
                        writeln!(output)?;
                        writeln!(
                            output,
                            "{}",
                            color.yellow("************************************************")
                        )?;
                        writeln!(
                            output,
                            "{}",
                            color.bold("   QUEEN BEE! You found all solutions!          ")
                        )?;
                        writeln!(
                            output,
                            "   Final Score: {} points across {} words.",
                            color.yellow(&session.current_score().to_string()),
                            session.solutions().len()
                        )?;
                        writeln!(
                            output,
                            "{}",
                            color.yellow("************************************************")
                        )?;
                        writeln!(output)?;
                    }
                }
                GuessOutcome::AlreadyGuessed { word } => {
                    writeln!(
                        output,
                        "{}",
                        color.yellow(&format!("  ⚠ Already found: '{word}'"))
                    )?;
                }
                GuessOutcome::InvalidWord(err) => {
                    writeln!(output, "{}", color.red(&format!("  ✗ '{tok}': {err}")))?;
                }
                GuessOutcome::GameFinished => {
                    writeln!(output, "{}", color.dim("  Round has finished."))?;
                    break;
                }
            }
        }
    }

    Ok(true)
}

fn print_puzzle_header<W: Write>(
    session: &SpellingBeeSession,
    output: &mut W,
    color: ColorMode,
) -> std::io::Result<()> {
    let puzzle = session.puzzle();
    let center = puzzle.center().to_char();
    let outer_display: String = puzzle
        .letters()
        .iter()
        .skip(1)
        .map(|l| l.to_char().to_string())
        .collect::<Vec<_>>()
        .join(" ");

    writeln!(
        output,
        "Letters: [ {} ] {}",
        color.bold(&center.to_string()),
        outer_display
    )?;
    writeln!(
        output,
        "Center letter is {}. Words must be at least 4 letters.",
        color.bold(&center.to_string())
    )?;
    writeln!(output)?;
    Ok(())
}

fn print_help<W: Write>(output: &mut W, color: ColorMode) -> std::io::Result<()> {
    writeln!(output)?;
    writeln!(output, "{}", color.bold("--- RULES & HELP ---"))?;
    writeln!(
        output,
        "- Guess words that use only the 7 displayed letters."
    )?;
    writeln!(output, "- Every word MUST include the central letter.")?;
    writeln!(output, "- Words must be at least 4 letters long.")?;
    writeln!(output, "- Letters may be repeated multiple times.")?;
    writeln!(
        output,
        "- Scoring: 4-letter words = 1 pt; 5+ letters = 1 pt/letter; Pangram = +7 pts."
    )?;
    writeln!(output, "- Commands: /hints, /guesses, /help, /quit.")?;
    writeln!(output)?;
    Ok(())
}

fn print_quit_summary<W: Write>(
    summary: &QuitSummary,
    output: &mut W,
    color: ColorMode,
) -> std::io::Result<()> {
    writeln!(output)?;
    writeln!(
        output,
        "{}",
        color.bold("========================================")
    )?;
    writeln!(
        output,
        "Final Score: {} (found {} of {} words)",
        color.yellow(&summary.score.to_string()),
        summary.found_count,
        summary.total_solutions
    )?;
    writeln!(
        output,
        "{}",
        color.bold("========================================")
    )?;
    if !summary.missed_words.is_empty() {
        writeln!(
            output,
            "\n{}",
            color.bold(&format!("Missed words ({}):", summary.missed_words.len()))
        )?;
        write!(
            output,
            "{}",
            format_three_columns(summary.missed_words.iter().map(|s| s.as_str()))
        )?;
    }
    writeln!(output)?;
    Ok(())
}
