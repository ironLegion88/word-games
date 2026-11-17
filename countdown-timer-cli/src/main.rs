use std::io::IsTerminal;
use std::process::ExitCode;
use std::time::Duration;

use countdown_timer_cli::{help_text, parse_args, prompt_duration, run_event_loop, App};

fn main() -> ExitCode {
    let parsed = match parse_args(std::env::args_os()) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("error: {}", err.message);
            return ExitCode::from(err.code);
        }
    };

    if parsed.help {
        print!("{}", help_text());
        return ExitCode::SUCCESS;
    }

    let stdin_is_terminal = std::io::stdin().is_terminal();
    let stdout_is_terminal = std::io::stdout().is_terminal();

    let duration = if let Some(secs) = parsed.seconds {
        if !stdin_is_terminal || !stdout_is_terminal {
            eprintln!("error: interactive terminal required for countdown timer");
            return ExitCode::from(1);
        }
        Duration::from_secs(secs)
    } else {
        if !stdin_is_terminal || !stdout_is_terminal {
            eprintln!("error: duration required via --seconds in non-interactive environment");
            return ExitCode::from(2);
        }
        let mut stdin = std::io::stdin().lock();
        let mut stdout = std::io::stdout();
        match prompt_duration(&mut stdin, &mut stdout) {
            Ok(Some(d)) => d,
            Ok(None) => return ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: failed to read duration: {e}");
                return ExitCode::from(1);
            }
        }
    };

    let mut app = match App::new(duration) {
        Ok(app) => app,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::from(2);
        }
    };

    if let Err(err) = run_event_loop(&mut app) {
        eprintln!("error: {err}");
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}
