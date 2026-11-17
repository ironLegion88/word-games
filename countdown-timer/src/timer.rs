use std::time::{Duration, Instant};

use crate::error::{TimerAction, TimerError};
use crate::state::{TimerEvent, TimerSnapshot, TimerState, TimerStatus};

/// A reusable, deterministic, clock-correct countdown timer.
///
/// Time tracking is based on caller-supplied [`Instant`] observations, ensuring that
/// testing is entirely deterministic without sleeping or thread scheduling delays.
/// All countdown arithmetic is calculated from monotonic durations rather than decremented
/// tick counters.
#[derive(Debug)]
pub struct CountdownTimer {
    initial: Duration,
    state: TimerState,
}

impl CountdownTimer {
    /// Creates a new countdown timer with the specified initial duration.
    ///
    /// The timer begins in the [`TimerStatus::Ready`] state.
    ///
    /// # Errors
    ///
    /// Returns [`TimerError::ZeroDuration`] if `initial` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use countdown_timer::{CountdownTimer, TimerError};
    ///
    /// let timer = CountdownTimer::new(Duration::from_secs(60)).unwrap();
    /// assert_eq!(timer.initial_duration(), Duration::from_secs(60));
    ///
    /// assert!(matches!(CountdownTimer::new(Duration::ZERO), Err(TimerError::ZeroDuration)));
    /// ```
    pub fn new(initial: Duration) -> Result<Self, TimerError> {
        if initial.is_zero() {
            return Err(TimerError::ZeroDuration);
        }
        Ok(Self {
            initial,
            state: TimerState::Ready,
        })
    }

    /// Returns the initial duration configured when this timer was created.
    #[must_use]
    pub fn initial_duration(&self) -> Duration {
        self.initial
    }

    /// Takes a pure, non-mutating snapshot of the timer state at the given instant.
    ///
    /// If the timer is running and `now` has reached or passed the deadline, this snapshot
    /// reports [`TimerStatus::Finished`] with zero remaining time. Calling `snapshot` does
    /// **not** mutate internal state and does **not** consume the pending [`TimerEvent::Expired`] event.
    #[must_use]
    pub fn snapshot(&self, now: Instant) -> TimerSnapshot {
        match &self.state {
            TimerState::Ready => TimerSnapshot {
                initial: self.initial,
                remaining: self.initial,
                status: TimerStatus::Ready,
            },
            TimerState::Paused { remaining } => TimerSnapshot {
                initial: self.initial,
                remaining: *remaining,
                status: TimerStatus::Paused,
            },
            TimerState::Finished => TimerSnapshot {
                initial: self.initial,
                remaining: Duration::ZERO,
                status: TimerStatus::Finished,
            },
            TimerState::Running {
                started_at,
                remaining_at_start,
            } => {
                let elapsed = now.saturating_duration_since(*started_at);
                let remaining = remaining_at_start.saturating_sub(elapsed);
                if remaining.is_zero() {
                    TimerSnapshot {
                        initial: self.initial,
                        remaining: Duration::ZERO,
                        status: TimerStatus::Finished,
                    }
                } else {
                    TimerSnapshot {
                        initial: self.initial,
                        remaining,
                        status: TimerStatus::Running,
                    }
                }
            }
        }
    }

    /// Checks if a running timer has reached its deadline, committing the transition
    /// to [`TimerStatus::Finished`] and emitting [`TimerEvent::Expired`] exactly once.
    ///
    /// Returns `None` if the timer is not running, has not reached its deadline, or has
    /// already emitted the expiration event.
    pub fn poll(&mut self, now: Instant) -> Option<TimerEvent> {
        if let TimerState::Running {
            started_at,
            remaining_at_start,
        } = self.state
        {
            let elapsed = now.saturating_duration_since(started_at);
            let remaining = remaining_at_start.saturating_sub(elapsed);
            if remaining.is_zero() {
                self.state = TimerState::Finished;
                return Some(TimerEvent::Expired);
            }
        }
        None
    }

    /// Starts the countdown from [`TimerStatus::Ready`].
    ///
    /// # Errors
    ///
    /// Returns [`TimerError::InvalidTransition`] if the timer is not currently in the `Ready` state.
    pub fn start(&mut self, now: Instant) -> Result<(), TimerError> {
        match self.state {
            TimerState::Ready => {
                self.state = TimerState::Running {
                    started_at: now,
                    remaining_at_start: self.initial,
                };
                Ok(())
            }
            _ => Err(TimerError::InvalidTransition {
                action: TimerAction::Start,
                status: self.state.status(),
            }),
        }
    }

    /// Pauses a running timer before its deadline.
    ///
    /// # Errors
    ///
    /// - Returns [`TimerError::DeadlineReached`] if the timer is running but `now` has reached or
    ///   exceeded its deadline. In this case, internal state is not modified and the pending
    ///   expiration event is preserved for subsequent `poll` calls.
    /// - Returns [`TimerError::InvalidTransition`] if the timer is not running.
    pub fn pause(&mut self, now: Instant) -> Result<(), TimerError> {
        match self.state {
            TimerState::Running {
                started_at,
                remaining_at_start,
            } => {
                let elapsed = now.saturating_duration_since(started_at);
                let remaining = remaining_at_start.saturating_sub(elapsed);
                if remaining.is_zero() {
                    Err(TimerError::DeadlineReached)
                } else {
                    self.state = TimerState::Paused { remaining };
                    Ok(())
                }
            }
            _ => Err(TimerError::InvalidTransition {
                action: TimerAction::Pause,
                status: self.state.status(),
            }),
        }
    }

    /// Resumes a paused timer with its remaining duration budget starting from `now`.
    ///
    /// # Errors
    ///
    /// Returns [`TimerError::InvalidTransition`] if the timer is not in the `Paused` state.
    pub fn resume(&mut self, now: Instant) -> Result<(), TimerError> {
        match self.state {
            TimerState::Paused { remaining } => {
                self.state = TimerState::Running {
                    started_at: now,
                    remaining_at_start: remaining,
                };
                Ok(())
            }
            _ => Err(TimerError::InvalidTransition {
                action: TimerAction::Resume,
                status: self.state.status(),
            }),
        }
    }

    /// Resets the timer to the [`TimerStatus::Ready`] state with its original initial duration.
    ///
    /// Any active running interval, paused state, or pending/unpolled expiration event is cancelled.
    pub fn reset(&mut self) {
        self.state = TimerState::Ready;
    }
}
