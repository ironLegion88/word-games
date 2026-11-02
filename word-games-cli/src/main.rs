//! Command line launcher for Word Games.

use ::spelling_bee as bee;
use std::env;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use trie::Trie;
use word_games_cli::spelling_bee::play_session;
use word_games_cli::terminal::ColorMode;

fn print_usage() {
    eprintln!(
        r#"word-games-cli — Interactive Word Games Suite

USAGE:
    cargo run -p word-games-cli -- <GAME> [OPTIONS]

AVAILABLE GAMES:
    spelling-bee    Play the Spelling Bee word puzzle game

COMMON OPTIONS:
    --dictionary <PATH>   Path to dictionary file (default: assets/dictionary.txt)
    --seed <U64>          Deterministic random seed for puzzle generation
    --color <MODE>        Color output mode: auto | always | never (default: auto)
    -h, --help            Print help information

SPELLING BEE OPTIONS:
    --solver <STRATEGY>   Solver strategy: filter | trie (default: trie)
    --difficulty <PRESET> Difficulty preset: easy | medium | hard
    --min-words <N>       Minimum solution words required
    --max-words <N>       Maximum solution words allowed
    --center <CHAR>       Force a specific central letter
    --letters <CHARS>     Set fixed 7 puzzle letters (requires 7 distinct letters)

EXAMPLES:
    cargo run -p word-games-cli -- spelling-bee
    cargo run -p word-games-cli -- spelling-bee --difficulty easy --seed 42
    cargo run -p word-games-cli -- spelling-bee --letters CAIMORT --center C
"#
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return Ok(());
    }

    let game_name = &args[0];
    if game_name.starts_with('-') {
        print_usage();
        return Ok(());
    }

    match game_name.to_ascii_lowercase().as_str() {
        "spelling-bee" | "bee" => run_spelling_bee(&args[1..])?,
        other => {
            eprintln!("Unknown game '{other}'. Available games: spelling-bee");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn run_spelling_bee(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut dict_path = PathBuf::from("assets/dictionary.txt");
    let mut seed: Option<u64> = None;
    let mut color = ColorMode::Auto;
    let mut solver = bee::SolverStrategy::TriePruned;
    let mut difficulty: Option<bee::DifficultyPreset> = None;
    let mut min_words: Option<usize> = None;
    let mut max_words: Option<usize> = None;
    let mut center_char: Option<char> = None;
    let mut fixed_letters: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dictionary" => {
                i += 1;
                if i < args.len() {
                    dict_path = PathBuf::from(&args[i]);
                }
            }
            "--seed" => {
                i += 1;
                if i < args.len() {
                    seed = Some(args[i].parse()?);
                }
            }
            "--color" => {
                i += 1;
                if i < args.len() {
                    color = ColorMode::from_str_name(&args[i])
                        .ok_or_else(|| format!("invalid color mode '{}'", args[i]))?;
                }
            }
            "--solver" => {
                i += 1;
                if i < args.len() {
                    solver = bee::SolverStrategy::from_str_name(&args[i])
                        .ok_or_else(|| format!("invalid solver strategy '{}'", args[i]))?;
                }
            }
            "--difficulty" => {
                i += 1;
                if i < args.len() {
                    difficulty = Some(
                        bee::DifficultyPreset::from_str_name(&args[i])
                            .ok_or_else(|| format!("invalid difficulty '{}'", args[i]))?,
                    );
                }
            }
            "--min-words" => {
                i += 1;
                if i < args.len() {
                    min_words = Some(args[i].parse()?);
                }
            }
            "--max-words" => {
                i += 1;
                if i < args.len() {
                    max_words = Some(args[i].parse()?);
                }
            }
            "--center" => {
                i += 1;
                if i < args.len() {
                    center_char = args[i].chars().next();
                }
            }
            "--letters" => {
                i += 1;
                if i < args.len() {
                    fixed_letters = Some(args[i].clone());
                }
            }
            other => {
                eprintln!("Unknown option: '{other}'");
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    if !dict_path.exists() {
        // Try fallback to word-games/assets/dictionary.txt if invoked from workspace parent
        let alt = Path::new("word-games").join(&dict_path);
        if alt.exists() {
            dict_path = alt;
        } else {
            eprintln!(
                "Error: dictionary file not found at '{}'",
                dict_path.display()
            );
            std::process::exit(1);
        }
    }

    let trie = Trie::from_file(&dict_path)?;

    // Build or generate puzzle
    let (puzzle, solutions) = if let Some(letters_str) = fixed_letters {
        let center = center_char.ok_or("fixed --letters requires explicit --center <CHAR>")?;
        let p = bee::SpellingBeePuzzle::from_letters_and_center(&letters_str, center)?;
        let sols = bee::solve(&p, &trie, solver);
        (p, sols)
    } else {
        let index = bee::AnchorIndex::build(&trie)?;
        let mut config = bee::GenerationConfig {
            center_override: center_char,
            solver_strategy: solver,
            ..Default::default()
        };
        if let Some(d) = difficulty {
            config = config.with_preset(d);
        }
        if let Some(min) = min_words {
            config.min_words = min;
        }
        if let Some(max) = max_words {
            config.max_words = max;
        }

        use rand::SeedableRng;
        let mut rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };

        bee::generate_puzzle(&index, &trie, &config, &mut rng)?
    };

    let session = bee::SpellingBeeSession::new(puzzle, solutions);
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = io::stdout();

    play_session(session, &mut reader, &mut stdout, color)?;

    Ok(())
}
