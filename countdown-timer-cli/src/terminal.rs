use std::io::{stdout, Stdout, Write};
use std::time::{Duration, Instant};

use countdown_timer::display_seconds_ceil;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{
    self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::QueueableCommand;

use crate::app::{map_key_event, App};
use crate::view::render_view;

/// RAII guard that restores the terminal state (cursor, alternate screen, raw mode)
/// upon destruction or error.
pub struct TerminalGuard {
    raw_mode_enabled: bool,
    alternate_screen_entered: bool,
    cursor_hidden: bool,
}

impl TerminalGuard {
    /// Attempts to configure the terminal into raw mode with alternate screen and hidden cursor.
    ///
    /// If any initialization step fails, all previously completed steps are rolled back immediately.
    ///
    /// # Errors
    ///
    /// Returns any [`std::io::Error`] encountered during terminal configuration.
    pub fn new() -> Result<Self, std::io::Error> {
        let mut guard = Self {
            raw_mode_enabled: false,
            alternate_screen_entered: false,
            cursor_hidden: false,
        };

        let mut out = stdout();

        enable_raw_mode()?;
        guard.raw_mode_enabled = true;

        if let Err(err) = out.queue(EnterAlternateScreen) {
            drop(guard);
            return Err(err);
        }
        guard.alternate_screen_entered = true;

        if let Err(err) = out.queue(Hide) {
            drop(guard);
            return Err(err);
        }
        guard.cursor_hidden = true;

        if let Err(err) = out.flush() {
            drop(guard);
            return Err(err);
        }

        Ok(guard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = stdout();
        if self.cursor_hidden {
            let _ = out.queue(Show);
            self.cursor_hidden = false;
        }
        if self.alternate_screen_entered {
            let _ = out.queue(LeaveAlternateScreen);
            self.alternate_screen_entered = false;
        }
        let _ = out.flush();
        if self.raw_mode_enabled {
            let _ = disable_raw_mode();
            self.raw_mode_enabled = false;
        }
    }
}

/// Runs the single-threaded Crossterm event loop until the application signals exit.
///
/// Ensures safe RAII terminal restoration on ordinary return or recoverable I/O error.
///
/// # Errors
///
/// Returns [`std::io::Error`] if terminal rendering or event polling fails.
pub fn run_event_loop(app: &mut App) -> Result<(), std::io::Error> {
    let guard = TerminalGuard::new()?;
    let mut out = stdout();

    let mut last_display_seconds = None;
    let mut last_status = None;
    let mut last_message = None;
    let mut last_dimensions = None;

    while !app.should_quit {
        let now = Instant::now();
        app.on_tick(now);

        let snap = app.timer.snapshot(now);
        let current_secs = display_seconds_ceil(snap.remaining);
        let (width, height) = terminal::size().unwrap_or((80, 24));

        let needs_draw = app.needs_redraw
            || last_display_seconds != Some(current_secs)
            || last_status != Some(snap.status)
            || last_message != app.message
            || last_dimensions != Some((width, height));

        if needs_draw {
            draw_frame(&mut out, &snap, app.message.as_deref(), width, height)?;
            last_display_seconds = Some(current_secs);
            last_status = Some(snap.status);
            last_message = app.message.clone();
            last_dimensions = Some((width, height));
            app.needs_redraw = false;
        }

        if app.should_quit {
            break;
        }

        // Bounded responsiveness wait (50ms target cap)
        if event::poll(Duration::from_millis(50))? {
            let ev = event::read()?;
            match ev {
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Press {
                        let key_now = Instant::now();
                        if let Some(cmd) = map_key_event(key_event) {
                            app.on_command(cmd, key_now);
                        }
                    }
                }
                Event::Resize(_w, _h) => {
                    app.on_resize();
                }
                _ => {}
            }
        }
    }

    drop(guard);
    Ok(())
}

fn draw_frame(
    out: &mut Stdout,
    snap: &countdown_timer::TimerSnapshot,
    message: Option<&str>,
    width: u16,
    height: u16,
) -> Result<(), std::io::Error> {
    let lines = render_view(*snap, message, width, height);
    out.queue(MoveTo(0, 0))?;
    out.queue(Clear(ClearType::All))?;

    for (i, line) in lines.iter().enumerate() {
        if i >= height as usize {
            break;
        }
        out.queue(MoveTo(0, i as u16))?;
        out.write_all(line.as_bytes())?;
    }

    out.flush()?;
    Ok(())
}
