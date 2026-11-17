use std::fmt;

use crate::state::TimerStatus;

/// Actions that attempt to transition the timer state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerAction {
    /// Attempt to start the timer from Ready.
    Start,
    /// Attempt to pause the timer from Running.
    Pause,
    /// Attempt to resume the timer from Paused.
    Resume,
}

impl fmt::Display for TimerAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start => write!(f, "start"),
            Self::Pause => write!(f, "pause"),
            Self::Resume => write!(f, "resume"),
        }
    }
}

/// Errors returned by timer operations when invalid inputs or transitions occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerError {
    /// Timer creation failed because the supplied duration was zero.
    ZeroDuration,
    /// The requested action cannot be performed in the timer's current state.
    InvalidTransition {
        /// The transition action attempted.
        action: TimerAction,
        /// The current status of the timer when the transition was attempted.
        status: TimerStatus,
    },
    /// The timer has reached or passed its deadline; a pause operation was rejected.
    DeadlineReached,
}

impl fmt::Display for TimerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDuration => write!(f, "timer duration must be strictly positive"),
            Self::InvalidTransition { action, status } => {
                write!(f, "cannot {} timer in {:?} state", action, status)
            }
            Self::DeadlineReached => write!(f, "timer has already reached its deadline"),
        }
    }
}

impl std::error::Error for TimerError {}
