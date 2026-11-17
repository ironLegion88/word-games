//! # countdown-timer
//!
//! A deterministic, clock-correct, `std`-only countdown timer library for Rust.
//!
//! ## Key Invariants and Design
//!
//! - **Clock-correct**: Time calculations use [`std::time::Duration`] and caller-injected
//!   [`std::time::Instant`] timestamps. The engine never decrements a tick counter or sleeps.
//! - **Pure snapshots**: [`CountdownTimer::snapshot`] is completely read-only (`&self`). If a
//!   running timer has reached its deadline, the snapshot reports [`TimerStatus::Finished`] with
//!   zero remaining time without mutating internal state or consuming the expiration event.
//! - **One-time expiration event**: Calling [`CountdownTimer::poll`] commits the transition from
//!   running to finished and returns `Some(TimerEvent::Expired)` exactly once per run cycle.
//! - **Safe transitions**: Attempting to pause a running timer that has already reached its deadline
//!   returns `Err(TimerError::DeadlineReached)` without swallowing the pending expiration.
//! - **Zero drift**: Pausing and resuming preserves exact fractional duration budgets across
//!   arbitrary intervals.
//!
//! ## Basic Example
//!
//! ```
//! use std::time::{Duration, Instant};
//! use countdown_timer::{CountdownTimer, TimerEvent, TimerStatus};
//!
//! let t0 = Instant::now();
//! let mut timer = CountdownTimer::new(Duration::from_secs(10)).unwrap();
//! assert_eq!(timer.initial_duration(), Duration::from_secs(10));
//!
//! // Start the timer
//! timer.start(t0).unwrap();
//!
//! // Inspect snapshot midway
//! let mid = t0 + Duration::from_secs(4);
//! let snap = timer.snapshot(mid);
//! assert_eq!(snap.status, TimerStatus::Running);
//! assert_eq!(snap.remaining, Duration::from_secs(6));
//!
//! // Poll before deadline returns None
//! assert_eq!(timer.poll(mid), None);
//!
//! // At deadline, snapshot reports Finished and poll emits Expired once
//! let end = t0 + Duration::from_secs(10);
//! assert_eq!(timer.snapshot(end).status, TimerStatus::Finished);
//! assert_eq!(timer.poll(end), Some(TimerEvent::Expired));
//! assert_eq!(timer.poll(end), None);
//! ```

pub mod error;
pub mod format;
pub mod state;
pub mod timer;

pub use error::{TimerAction, TimerError};
pub use format::{display_seconds_ceil, format_countdown};
pub use state::{TimerEvent, TimerSnapshot, TimerStatus};
pub use timer::CountdownTimer;
