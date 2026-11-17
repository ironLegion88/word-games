use std::time::{Duration, Instant};

use countdown_timer::{
    CountdownTimer, TimerAction, TimerError, TimerEvent, TimerSnapshot, TimerStatus,
};

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_range(&mut self, min: u64, max: u64) -> u64 {
        min + (self.next_u64() % (max - min + 1))
    }
}

/// Independent reference model that maintains cumulative running intervals
/// without using the production struct's budget calculation.
struct ReferenceTimer {
    initial: Duration,
    status: TimerStatus,
    accumulated_running: Duration,
    current_interval_start: Option<Duration>,
    expired_emitted: bool,
    total_expired: usize,
}

impl ReferenceTimer {
    fn new(initial: Duration) -> Self {
        Self {
            initial,
            status: TimerStatus::Ready,
            accumulated_running: Duration::ZERO,
            current_interval_start: None,
            expired_emitted: false,
            total_expired: 0,
        }
    }

    fn snapshot(&self, current_offset: Duration) -> TimerSnapshot {
        match self.status {
            TimerStatus::Ready => TimerSnapshot {
                initial: self.initial,
                remaining: self.initial,
                status: TimerStatus::Ready,
            },
            TimerStatus::Paused => {
                let remaining = self.initial.saturating_sub(self.accumulated_running);
                TimerSnapshot {
                    initial: self.initial,
                    remaining,
                    status: TimerStatus::Paused,
                }
            }
            TimerStatus::Finished => TimerSnapshot {
                initial: self.initial,
                remaining: Duration::ZERO,
                status: TimerStatus::Finished,
            },
            TimerStatus::Running => {
                let interval_start = self.current_interval_start.expect("running has start");
                let elapsed_in_interval = current_offset.saturating_sub(interval_start);
                let total_elapsed = self.accumulated_running.saturating_add(elapsed_in_interval);
                if total_elapsed >= self.initial {
                    TimerSnapshot {
                        initial: self.initial,
                        remaining: Duration::ZERO,
                        status: TimerStatus::Finished,
                    }
                } else {
                    TimerSnapshot {
                        initial: self.initial,
                        remaining: self.initial.saturating_sub(total_elapsed),
                        status: TimerStatus::Running,
                    }
                }
            }
        }
    }

    fn poll(&mut self, current_offset: Duration) -> Option<TimerEvent> {
        if self.status == TimerStatus::Running {
            let interval_start = self.current_interval_start.expect("running has start");
            let elapsed_in_interval = current_offset.saturating_sub(interval_start);
            let total_elapsed = self.accumulated_running.saturating_add(elapsed_in_interval);
            if total_elapsed >= self.initial {
                self.accumulated_running = self.initial;
                self.current_interval_start = None;
                self.status = TimerStatus::Finished;
                if !self.expired_emitted {
                    self.expired_emitted = true;
                    self.total_expired += 1;
                    return Some(TimerEvent::Expired);
                }
            }
        }
        None
    }

    fn start(&mut self, current_offset: Duration) -> Result<(), TimerError> {
        if self.status == TimerStatus::Ready {
            self.status = TimerStatus::Running;
            self.current_interval_start = Some(current_offset);
            Ok(())
        } else {
            Err(TimerError::InvalidTransition {
                action: TimerAction::Start,
                status: self.status,
            })
        }
    }

    fn pause(&mut self, current_offset: Duration) -> Result<(), TimerError> {
        if self.status == TimerStatus::Running {
            let interval_start = self.current_interval_start.expect("running has start");
            let elapsed_in_interval = current_offset.saturating_sub(interval_start);
            let total_elapsed = self.accumulated_running.saturating_add(elapsed_in_interval);
            if total_elapsed >= self.initial {
                Err(TimerError::DeadlineReached)
            } else {
                self.accumulated_running = total_elapsed;
                self.current_interval_start = None;
                self.status = TimerStatus::Paused;
                Ok(())
            }
        } else {
            Err(TimerError::InvalidTransition {
                action: TimerAction::Pause,
                status: self.status,
            })
        }
    }

    fn resume(&mut self, current_offset: Duration) -> Result<(), TimerError> {
        if self.status == TimerStatus::Paused {
            self.status = TimerStatus::Running;
            self.current_interval_start = Some(current_offset);
            Ok(())
        } else {
            Err(TimerError::InvalidTransition {
                action: TimerAction::Resume,
                status: self.status,
            })
        }
    }

    fn reset(&mut self) {
        self.status = TimerStatus::Ready;
        self.accumulated_running = Duration::ZERO;
        self.current_interval_start = None;
        self.expired_emitted = false;
    }
}

fn run_trajectory(seed: u64, iterations: usize) {
    let mut rng = SplitMix64::new(seed);
    let t0 = Instant::now();
    let initial_secs = rng.next_range(1, 120);
    let initial = Duration::from_secs(initial_secs);

    let mut timer = CountdownTimer::new(initial).unwrap();
    let mut ref_timer = ReferenceTimer::new(initial);
    let mut current_offset = Duration::ZERO;
    let mut actual_expired_count = 0;

    for step in 0..iterations {
        // Synthetically advance time nondecreasingly (0 to 500ms or occasional large jump)
        let delta = match rng.next_range(0, 10) {
            0 => Duration::ZERO,
            1..=8 => Duration::from_millis(rng.next_range(1, 250)),
            9 => Duration::from_nanos(rng.next_range(1, 999_999)),
            _ => Duration::from_secs(rng.next_range(1, 60)),
        };
        current_offset += delta;
        let now = t0 + current_offset;

        // Pick action: 0: snapshot, 1: poll, 2: start, 3: pause, 4: resume, 5: reset
        let action = rng.next_range(0, 5);
        match action {
            0 => {
                // Snapshot comparison
                let s_actual = timer.snapshot(now);
                let s_ref = ref_timer.snapshot(current_offset);
                assert_eq!(
                    s_actual, s_ref,
                    "Snapshot mismatch at step {step}, seed {seed:#x}"
                );
            }
            1 => {
                // Poll comparison
                let e_actual = timer.poll(now);
                let e_ref = ref_timer.poll(current_offset);
                assert_eq!(
                    e_actual, e_ref,
                    "Poll mismatch at step {step}, seed {seed:#x}"
                );
                if e_actual.is_some() {
                    actual_expired_count += 1;
                }
                assert_eq!(actual_expired_count, ref_timer.total_expired);
            }
            2 => {
                // Start
                let res_actual = timer.start(now);
                let res_ref = ref_timer.start(current_offset);
                assert_eq!(
                    res_actual, res_ref,
                    "Start mismatch at step {step}, seed {seed:#x}"
                );
            }
            3 => {
                // Pause
                let res_actual = timer.pause(now);
                let res_ref = ref_timer.pause(current_offset);
                assert_eq!(
                    res_actual, res_ref,
                    "Pause mismatch at step {step}, seed {seed:#x}"
                );
            }
            4 => {
                // Resume
                let res_actual = timer.resume(now);
                let res_ref = ref_timer.resume(current_offset);
                assert_eq!(
                    res_actual, res_ref,
                    "Resume mismatch at step {step}, seed {seed:#x}"
                );
            }
            5 => {
                // Reset
                timer.reset();
                ref_timer.reset();
            }
            _ => unreachable!(),
        }

        // Post-condition: Invariant I01, I02, I10
        assert_eq!(timer.initial_duration(), initial);
        let post_snap = timer.snapshot(now);
        assert!(post_snap.remaining <= initial);
        if post_snap.status == TimerStatus::Ready {
            assert_eq!(post_snap.remaining, initial);
        } else if post_snap.status == TimerStatus::Finished {
            assert_eq!(post_snap.remaining, Duration::ZERO);
        }
    }
}

#[test]
fn test_seeded_trajectory_c0ffee() {
    run_trajectory(0x00C0_FFEE, 2000);
}

#[test]
fn test_seeded_trajectory_5eed2026() {
    run_trajectory(0x5EED_2026, 2000);
}

#[test]
fn test_seeded_trajectory_0badc0de() {
    run_trajectory(0x0BAD_C0DE, 2000);
}
