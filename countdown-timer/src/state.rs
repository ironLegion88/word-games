use std::time::{Duration, Instant};

/// Status of the countdown timer reported to consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerStatus {
    /// Timer has been initialized and is ready to start.
    Ready,
    /// Timer is currently running and counting down.
    Running,
    /// Timer is paused; countdown is frozen.
    Paused,
    /// Timer has reached its deadline and completed.
    Finished,
}

/// A pure, immutable snapshot of the timer's state at a specific observation point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerSnapshot {
    /// Initial duration configured when the timer was created.
    pub initial: Duration,
    /// Exact remaining duration calculated at the observed instant.
    pub remaining: Duration,
    /// Current logical status at the observed instant.
    pub status: TimerStatus,
}

/// Lifecycle event emitted by the timer upon completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerEvent {
    /// The timer has expired (reached zero remaining time).
    Expired,
}

/// Private internal state machine representation.
#[derive(Debug)]
pub(crate) enum TimerState {
    Ready,
    Running {
        started_at: Instant,
        remaining_at_start: Duration,
    },
    Paused {
        remaining: Duration,
    },
    Finished,
}

impl TimerState {
    pub(crate) fn status(&self) -> TimerStatus {
        match self {
            Self::Ready => TimerStatus::Ready,
            Self::Running { .. } => TimerStatus::Running,
            Self::Paused { .. } => TimerStatus::Paused,
            Self::Finished => TimerStatus::Finished,
        }
    }
}
