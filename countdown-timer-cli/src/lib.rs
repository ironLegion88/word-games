//! Interactive standalone terminal countdown timer application library.

pub mod app;
pub mod args;
pub mod terminal;
pub mod view;

pub use app::{map_key_event, App, AppCommand};
pub use args::{help_text, parse_args, parse_seconds_str, prompt_duration, ArgError, ParsedArgs};
pub use terminal::{run_event_loop, TerminalGuard};
pub use view::render_view;
