//! Command line launcher for Word Games.

use ::spelling_bee as bee;
use std::env;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use trie::Trie;
use word_games_cli::replay::{prompt_replay, ReplayChoice};
use word_games_cli::spelling_bee::play_session;
use word_games_cli::terminal::ColorMode;

fn print_usage() {
    eprintln!(
        r#"word-games-cli — Interactive Word Games Suite

USAGE:
    cargo run -p word-games-cli -- <GAME> [OPTIONS]

AVAILABLE GAMES:
    spelling-bee    Play the Spelling Bee word puzzle game
    boggle          Play the Boggle word grid game
    wordle          Play the Wordle word deduction game

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

BOGGLE OPTIONS:
    --size <4|5|6>        Grid dimension (default: 4)
    --solver <STRATEGY>   Solver strategy: scan | trie (default: trie)
    --generator <GEN>     Board generator: uniform | weighted | dice (default: uniform)
    --board <CHARS>       Fixed board tiles (row-major string or spaces/newlines)

WORDLE OPTIONS:
    --answers <PATH>      Path to custom answers override list
    --feedback <STRATEGY> Feedback strategy: frequency | marker (default: frequency)

EXAMPLES:
    cargo run -p word-games-cli -- spelling-bee
    cargo run -p word-games-cli -- spelling-bee --difficulty easy --seed 42
    cargo run -p word-games-cli -- spelling-bee --letters CAIMORT --center C
    cargo run -p word-games-cli -- boggle
    cargo run -p word-games-cli -- boggle --size 5 --generator weighted
    cargo run -p word-games-cli -- boggle --generator dice --seed 42
    cargo run -p word-games-cli -- boggle --board "CATSDOGSQREWBIRD"
    cargo run -p word-games-cli -- wordle
    cargo run -p word-games-cli -- wordle --seed 42 --feedback marker
    cargo run -p word-games-cli -- wordle --answers assets/wordle_curated_answers.txt
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
        "boggle" => run_boggle(&args[1..])?,
        "wordle" => run_wordle(&args[1..])?,
        other => {
            eprintln!("Unknown game '{other}'. Available games: spelling-bee, boggle, wordle");
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
        let alt_word_games = Path::new("word-games").join(&dict_path);
        let alt_parent = Path::new("..").join(&dict_path);
        if alt_word_games.exists() {
            dict_path = alt_word_games;
        } else if alt_parent.exists() {
            dict_path = alt_parent;
        } else {
            eprintln!(
                "Error: dictionary file not found at '{}'",
                dict_path.display()
            );
            std::process::exit(1);
        }
    }

    let trie = Trie::from_file(&dict_path)?;

    use rand::SeedableRng;
    let mut rng = match seed {
        Some(s) => rand::rngs::StdRng::seed_from_u64(s),
        None => rand::rngs::StdRng::from_entropy(),
    };

    let index = if fixed_letters.is_none() {
        Some(bee::AnchorIndex::build(&trie)?)
    } else {
        None
    };

    let mut config = bee::GenerationConfig {
        center_override: center_char,
        solver_strategy: solver,
        ..Default::default()
    };
    if let Some(min) = min_words {
        config.min_words = min;
    }
    if let Some(max) = max_words {
        config.max_words = max;
    }
    if let Some(d) = difficulty {
        config = config.with_preset(d)?;
    }

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = io::stdout();

    loop {
        let (puzzle, solutions) = if let Some(ref letters_str) = fixed_letters {
            let center = center_char.ok_or("fixed --letters requires explicit --center <CHAR>")?;
            let p = bee::SpellingBeePuzzle::from_letters_and_center(letters_str, center)?;
            let sols = bee::solve(&p, &trie, solver);
            (p, sols)
        } else {
            let idx = index.as_ref().unwrap();
            bee::generate_puzzle(idx, &trie, &config, &mut rng)?
        };

        let session = bee::SpellingBeeSession::new(puzzle, solutions);
        let completed = play_session(session, &mut reader, &mut stdout, color)?;

        if !completed {
            break;
        }

        match prompt_replay(&mut reader, &mut stdout, color)? {
            ReplayChoice::SameSettings => continue,
            ReplayChoice::Exit => break,
        }
    }

    Ok(())
}

fn run_boggle(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut dict_path = PathBuf::from("assets/dictionary.txt");
    let mut seed: Option<u64> = None;
    let mut color = ColorMode::Auto;
    let mut size: usize = 4;
    let mut solver = boggle::SolverStrategy::Trie;
    let mut generator = boggle::BoardGenerator::Uniform;
    let mut fixed_board: Option<String> = None;

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
                    color = match args[i].to_ascii_lowercase().as_str() {
                        "always" => ColorMode::Always,
                        "never" => ColorMode::Never,
                        "auto" => ColorMode::Auto,
                        other => {
                            eprintln!(
                                "Invalid color mode '{other}'. Options: auto | always | never"
                            );
                            std::process::exit(1);
                        }
                    };
                }
            }
            "--size" => {
                i += 1;
                if i < args.len() {
                    size = args[i].parse()?;
                    if !(boggle::MIN_BOARD_SIZE..=boggle::MAX_BOARD_SIZE).contains(&size) {
                        eprintln!("Invalid board size {size}. Supported sizes: 4, 5, 6");
                        std::process::exit(1);
                    }
                }
            }
            "--solver" => {
                i += 1;
                if i < args.len() {
                    solver = boggle::SolverStrategy::from_str_name(&args[i]).ok_or_else(|| {
                        format!(
                            "Unknown solver strategy '{}'. Options: scan | trie",
                            args[i]
                        )
                    })?;
                }
            }
            "--generator" => {
                i += 1;
                if i < args.len() {
                    generator =
                        boggle::BoardGenerator::from_str_name(&args[i]).ok_or_else(|| {
                            format!(
                                "Unknown generator '{}'. Options: uniform | weighted | dice",
                                args[i]
                            )
                        })?;
                }
            }
            "--board" | "--fixed-board" => {
                i += 1;
                if i < args.len() {
                    fixed_board = Some(args[i].clone());
                }
            }
            other => {
                eprintln!("Unknown boggle option '{other}'. Type --help for usage.");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    if !dict_path.exists() {
        let alt_word_games = Path::new("word-games").join(&dict_path);
        let alt_parent = Path::new("..").join(&dict_path);
        if alt_word_games.exists() {
            dict_path = alt_word_games;
        } else if alt_parent.exists() {
            dict_path = alt_parent;
        } else {
            eprintln!(
                "Error: dictionary file not found at '{}'",
                dict_path.display()
            );
            std::process::exit(1);
        }
    }

    let trie = Trie::from_file(&dict_path)?;

    use rand::SeedableRng;
    let mut rng = match seed {
        Some(s) => rand::rngs::StdRng::seed_from_u64(s),
        None => rand::rngs::StdRng::from_entropy(),
    };

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = io::stdout();

    loop {
        let (board, solutions) = if let Some(ref board_str) = fixed_board {
            let b = boggle::Board::from_str_tiles(size, board_str)?;
            let sols = boggle::solve(&b, &trie, solver)?;
            (b, sols)
        } else {
            boggle::generate_playable_board(
                size,
                generator,
                &trie,
                solver,
                &mut rng,
                boggle::DEFAULT_RETRY_BUDGET,
            )?
        };

        let session = boggle::BoggleSession::new(board, solutions, solver)?;
        let completed =
            word_games_cli::boggle::play_session(session, &mut reader, &mut stdout, color)?;

        if !completed {
            break;
        }

        match prompt_replay(&mut reader, &mut stdout, color)? {
            ReplayChoice::SameSettings => continue,
            ReplayChoice::Exit => break,
        }
    }

    Ok(())
}

fn run_wordle(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut dict_path = PathBuf::from("assets/dictionary.txt");
    let mut answers_path: Option<PathBuf> = None;
    let mut seed: Option<u64> = None;
    let mut color = ColorMode::Auto;
    let mut feedback_strategy = wordle::FeedbackStrategy::Frequency;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dictionary" => {
                i += 1;
                if i < args.len() {
                    dict_path = PathBuf::from(&args[i]);
                }
            }
            "--answers" => {
                i += 1;
                if i < args.len() {
                    answers_path = Some(PathBuf::from(&args[i]));
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
            "--feedback" => {
                i += 1;
                if i < args.len() {
                    match args[i].to_ascii_lowercase().as_str() {
                        "frequency" | "freq" => {
                            feedback_strategy = wordle::FeedbackStrategy::Frequency
                        }
                        "marker" | "markers" => {
                            feedback_strategy = wordle::FeedbackStrategy::Marker
                        }
                        other => {
                            return Err(format!(
                                "unknown feedback strategy '{other}'. Options: frequency, marker"
                            )
                            .into());
                        }
                    }
                }
            }
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            other => {
                return Err(format!("unknown option '{other}' for wordle").into());
            }
        }
        i += 1;
    }

    let trie = Trie::from_file(&dict_path)?;
    let mut lexicon =
        wordle::WordleLexicon::from_trie(&trie, wordle::lexicon::BUNDLED_CURATED_ANSWERS_SRC)?;

    if let Some(ref ans_path) = answers_path {
        let (new_lex, report) = lexicon.with_answers_file(ans_path)?;
        if report.duplicates_filtered > 0 {
            eprintln!(
                "Note: Filtered {} duplicate lines from answers override file ({} unique answers loaded).",
                report.duplicates_filtered, report.unique_answers
            );
        }
        lexicon = new_lex;
    }

    use rand::SeedableRng;
    let mut rng = match seed {
        Some(s) => rand::rngs::StdRng::seed_from_u64(s),
        None => rand::rngs::StdRng::from_entropy(),
    };

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = io::stdout();

    loop {
        let game = wordle::WordleGame::with_rng(lexicon.clone(), feedback_strategy, &mut rng)?;
        let completed =
            word_games_cli::wordle::play_session(game, &mut reader, &mut stdout, color)?;

        if !completed {
            break;
        }

        match prompt_replay(&mut reader, &mut stdout, color)? {
            ReplayChoice::SameSettings => continue,
            ReplayChoice::Exit => break,
        }
    }

    Ok(())
}
