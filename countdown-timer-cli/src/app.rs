use std::time::{Duration, Instant};

use countdown_timer::{CountdownTimer, TimerError, TimerEvent, TimerStatus};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Commands derived from user input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppCommand {
    /// Toggle timer between Start, Pause, and Resume.
    Toggle,
    /// Reset timer to initial duration and clear messages.
    Reset,
    /// Exit the application cleanly.
    Quit,
}

/// Maps a crossterm key event to an application command.
///
/// Only reacts to [`KeyEventKind::Press`] to avoid duplicate actions from held key repeats
/// or key release events.
#[must_use]
pub fn map_key_event(event: KeyEvent) -> Option<AppCommand> {
    if event.kind != KeyEventKind::Press {
        return None;
    }

    if event.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(event.code, KeyCode::Char('c') | KeyCode::Char('C'))
    {
        return Some(AppCommand::Quit);
    }

    match event.code {
        KeyCode::Char(' ') => Some(AppCommand::Toggle),
        KeyCode::Char('r' | 'R') => Some(AppCommand::Reset),
        KeyCode::Char('q' | 'Q') | KeyCode::Esc => Some(AppCommand::Quit),
        _ => None,
    }
}

/// Standalone timer application controller.
///
/// Owns the countdown timer, manages derived UI messages, and coordinates
/// time progression and user interactions.
pub struct App {
    /// Owned countdown timer engine.
    pub timer: CountdownTimer,
    /// Optional transient UI message (e.g. "Time's up!").
    pub message: Option<String>,
    /// Flag indicating whether the application should terminate.
    pub should_quit: bool,
    /// Flag indicating whether the view requires redrawing.
    pub needs_redraw: bool,
}

impl App {
    /// Constructs a new application instance with the given countdown duration.
    ///
    /// # Errors
    ///
    /// Returns [`TimerError::ZeroDuration`] if `duration` is zero.
    pub fn new(duration: Duration) -> Result<Self, TimerError> {
        let timer = CountdownTimer::new(duration)?;
        Ok(Self {
            timer,
            message: None,
            should_quit: false,
            needs_redraw: true,
        })
    }

    /// Advances the timer by the given instant and commits one-time expiration if the deadline has arrived.
    pub fn on_tick(&mut self, now: Instant) {
        if let Some(TimerEvent::Expired) = self.timer.poll(now) {
            self.message = Some("Time's up!".to_string());
            self.needs_redraw = true;
        }
    }

    /// Handles an application command, ensuring expiration is checked and committed
    /// before applying the command.
    pub fn on_command(&mut self, cmd: AppCommand, now: Instant) {
        // Critical requirement: poll before acting on any user command at the given instant
        if let Some(TimerEvent::Expired) = self.timer.poll(now) {
            self.message = Some("Time's up!".to_string());
            self.needs_redraw = true;
        }

        match cmd {
            AppCommand::Quit => {
                self.should_quit = true;
            }
            AppCommand::Reset => {
                self.timer.reset();
                self.message = None;
                self.needs_redraw = true;
            }
            AppCommand::Toggle => {
                let snap = self.timer.snapshot(now);
                match snap.status {
                    TimerStatus::Ready => {
                        let _ = self.timer.start(now);
                        self.needs_redraw = true;
                    }
                    TimerStatus::Running => {
                        match self.timer.pause(now) {
                            Ok(()) => {
                                self.needs_redraw = true;
                            }
                            Err(TimerError::DeadlineReached) => {
                                // Deadline reached during pause attempt
                                let _ = self.timer.poll(now);
                                self.message = Some("Time's up!".to_string());
                                self.needs_redraw = true;
                            }
                            Err(_) => {}
                        }
                    }
                    TimerStatus::Paused => {
                        let _ = self.timer.resume(now);
                        self.needs_redraw = true;
                    }
                    TimerStatus::Finished => {
                        self.message = Some("Time's up! Press 'r' to reset.".to_string());
                        self.needs_redraw = true;
                    }
                }
            }
        }
    }

    /// Marks that the view requires redrawing due to a terminal resize or external signal.
    pub fn on_resize(&mut self) {
        self.needs_redraw = true;
    }
}
