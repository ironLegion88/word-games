use std::io::{BufRead, Write};
use trie::{normalize_word, Trie};

use crate::commands::{parse_command, Command, CommandError};

/// Runs the interactive REPL loop reading from `reader` and writing output to `writer`.
pub fn run_repl<R: BufRead, W: Write>(
    trie: &mut Trie,
    mut reader: R,
    mut writer: W,
) -> std::io::Result<()> {
    writeln!(writer, "Word Trie Interactive CLI")?;
    writeln!(
        writer,
        "Type 'help' for available commands, or 'quit' to exit.\n"
    )?;

    let mut line = String::new();

    loop {
        write!(writer, "trie> ")?;
        writer.flush()?;

        line.clear();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            // EOF reached
            writeln!(writer)?;
            break;
        }

        match parse_command(&line) {
            Ok(Command::Help) => print_help(&mut writer)?,
            Ok(Command::New) => {
                *trie = Trie::new();
                writeln!(writer, "Created a new empty dictionary.")?;
            }
            Ok(Command::Load(path)) => match Trie::from_file(&path) {
                Ok(new_trie) => {
                    let count = new_trie.len();
                    *trie = new_trie;
                    writeln!(
                        writer,
                        "Loaded {count} words from '{path}' into a fresh dictionary."
                    )?;
                }
                Err(err) => writeln!(writer, "Error loading dictionary from '{path}': {err}")?,
            },
            Ok(Command::AddFile(path)) => match trie.extend_from_file(&path) {
                Ok(report) => {
                    writeln!(
                        writer,
                        "Added {} new words ({} already present, {} total tokens) from '{path}'. Total words: {}.",
                        report.inserted,
                        report.already_present,
                        report.tokens_seen,
                        trie.len()
                    )?;
                }
                Err(err) => writeln!(writer, "Error adding words from '{path}': {err}")?,
            },
            Ok(Command::Insert(word)) => match trie.insert(&word) {
                Ok(true) => {
                    writeln!(
                        writer,
                        "Inserted \"{}\" (total words: {}).",
                        word.to_ascii_uppercase(),
                        trie.len()
                    )?;
                }
                Ok(false) => {
                    writeln!(
                        writer,
                        "Word \"{}\" already exists in the dictionary.",
                        word.to_ascii_uppercase()
                    )?;
                }
                Err(err) => writeln!(writer, "Error inserting word: {err}")?,
            },
            Ok(Command::Contains(word)) => match trie.try_contains(&word) {
                Ok(true) => {
                    writeln!(
                        writer,
                        "true: \"{}\" is in the dictionary.",
                        word.to_ascii_uppercase()
                    )?;
                }
                Ok(false) => {
                    writeln!(
                        writer,
                        "false: \"{}\" is NOT in the dictionary.",
                        word.to_ascii_uppercase()
                    )?;
                }
                Err(err) => {
                    writeln!(writer, "false (invalid input: {err})")?;
                }
            },
            Ok(Command::Remove(word)) => match trie.remove(&word) {
                Ok(true) => {
                    writeln!(
                        writer,
                        "Removed \"{}\" (remaining words: {}).",
                        word.to_ascii_uppercase(),
                        trie.len()
                    )?;
                }
                Ok(false) => {
                    writeln!(
                        writer,
                        "Word \"{}\" was not found in the dictionary.",
                        word.to_ascii_uppercase()
                    )?;
                }
                Err(err) => writeln!(writer, "Error removing word: {err}")?,
            },
            Ok(Command::RemoveFile(path)) => match trie.remove_from_file(&path) {
                Ok(report) => {
                    writeln!(
                        writer,
                        "Removed {} words ({} not found, {} requested) from '{path}'. Remaining words: {}.",
                        report.removed,
                        report.not_found,
                        report.requested,
                        trie.len()
                    )?;
                }
                Err(err) => writeln!(writer, "Error removing words from '{path}': {err}")?,
            },
            Ok(Command::Present(path)) => match trie.filter_present_from_file(&path) {
                Ok(words) => {
                    writeln!(writer, "Words present in trie ({}):", words.len())?;
                    for w in words {
                        writeln!(writer, "{w}")?;
                    }
                }
                Err(err) => writeln!(writer, "Error filtering present words from '{path}': {err}")?,
            },
            Ok(Command::Absent(path)) => match trie.filter_absent_from_file(&path) {
                Ok(words) => {
                    writeln!(writer, "Words absent from trie ({}):", words.len())?;
                    for w in words {
                        writeln!(writer, "{w}")?;
                    }
                }
                Err(err) => writeln!(writer, "Error filtering absent words from '{path}': {err}")?,
            },
            Ok(Command::Normalize(word)) => match normalize_word(&word) {
                Ok(upper) => writeln!(writer, "Normalized: {upper}")?,
                Err(err) => writeln!(writer, "Error normalizing word: {err}")?,
            },
            Ok(Command::Stats) => {
                let s = trie.stats();
                writeln!(writer, "Dictionary Statistics:")?;
                writeln!(writer, "  Words:            {}", s.words)?;
                writeln!(writer, "  Live nodes:       {}", s.live_nodes)?;
                writeln!(writer, "  Arena slots:      {}", s.total_slots)?;
                writeln!(writer, "  Free slots:       {}", s.free_slots)?;
                writeln!(
                    writer,
                    "  Allocated memory: {} bytes (nominal arena nodes)",
                    s.total_slots * std::mem::size_of::<trie::node::Node>()
                )?;
            }
            Ok(Command::Quit) => {
                writeln!(writer, "Goodbye!")?;
                break;
            }
            Err(CommandError::EmptyLine) => {}
            Err(err) => {
                writeln!(writer, "Error: {err}")?;
            }
        }
    }

    Ok(())
}

fn print_help<W: Write>(writer: &mut W) -> std::io::Result<()> {
    writeln!(writer, "Available Commands:")?;
    writeln!(writer, "  help                  Show this list of commands")?;
    writeln!(
        writer,
        "  new                   Reset to a new empty dictionary"
    )?;
    writeln!(
        writer,
        "  load <path>           Replace dictionary with words from file"
    )?;
    writeln!(
        writer,
        "  add-file <path>       Add words from file to existing dictionary"
    )?;
    writeln!(
        writer,
        "  insert <word>         Insert a single word into the dictionary"
    )?;
    writeln!(
        writer,
        "  contains <word>       Check if a word is in the dictionary (returns boolean)"
    )?;
    writeln!(
        writer,
        "  remove <word>         Remove a single word from the dictionary"
    )?;
    writeln!(
        writer,
        "  remove-file <path>    Remove a list of words from file"
    )?;
    writeln!(
        writer,
        "  present <path>        Output all words in file that are present in trie"
    )?;
    writeln!(
        writer,
        "  absent <path>         Output all words in file that are absent from trie"
    )?;
    writeln!(
        writer,
        "  normalize <word>      Convert a word to uppercase ASCII"
    )?;
    writeln!(
        writer,
        "  stats                 Display dictionary node and arena statistics"
    )?;
    writeln!(writer, "  quit / exit           Exit the program")?;
    Ok(())
}
