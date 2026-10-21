mod commands;
mod repl;

use std::env;
use std::io::{self, BufReader};
use std::process::exit;
use trie::Trie;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut dict_path = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dictionary" | "-d" => {
                if i + 1 < args.len() {
                    dict_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --dictionary requires a <path> argument.");
                    exit(1);
                }
            }
            "--help" | "-h" => {
                println!("trie-cli: Word Trie interactive dictionary");
                println!("Usage: trie-cli [--dictionary <path>]");
                return;
            }
            other => {
                eprintln!("Error: unrecognized argument '{other}'.");
                eprintln!("Usage: trie-cli [--dictionary <path>]");
                exit(1);
            }
        }
    }

    let mut trie = if let Some(path) = dict_path {
        println!("Loading initial dictionary from '{path}'...");
        match Trie::from_file(&path) {
            Ok(t) => {
                println!("Successfully loaded {} words.", t.len());
                t
            }
            Err(e) => {
                eprintln!("Failed to load initial dictionary from '{path}': {e}");
                exit(1);
            }
        }
    } else {
        Trie::new()
    };

    let stdin = io::stdin();
    let stdout = io::stdout();
    let reader = BufReader::new(stdin.lock());
    let writer = stdout.lock();

    if let Err(e) = repl::run_repl(&mut trie, reader, writer) {
        eprintln!("I/O Error in REPL: {e}");
        exit(1);
    }
}
