use std::time::{Duration, Instant};

use countdown_timer::{CountdownTimer, TimerAction, TimerError, TimerEvent, TimerStatus};

#[test]
fn test_e01_zero_duration() {
    assert_eq!(
        CountdownTimer::new(Duration::ZERO).unwrap_err(),
        TimerError::ZeroDuration
    );
}

#[test]
fn test_e02_nano_duration() {
    let timer = CountdownTimer::new(Duration::from_nanos(1)).unwrap();
    let now = Instant::now();
    let snap = timer.snapshot(now);
    assert_eq!(snap.status, TimerStatus::Ready);
    assert_eq!(snap.remaining, Duration::from_nanos(1));
    assert_eq!(snap.initial, Duration::from_nanos(1));
}

#[test]
fn test_e03_one_sec_duration() {
    let timer = CountdownTimer::new(Duration::from_secs(1)).unwrap();
    assert_eq!(timer.initial_duration(), Duration::from_secs(1));
}

#[test]
fn test_e04_24h_duration() {
    let d = Duration::from_secs(86400);
    let timer = CountdownTimer::new(d).unwrap();
    assert_eq!(timer.initial_duration(), d);
}

#[test]
fn test_e05_greater_than_24h_duration() {
    let d = Duration::from_secs(86400 * 7);
    let timer = CountdownTimer::new(d).unwrap();
    assert_eq!(timer.initial_duration(), d);
}

#[test]
fn test_e06_ready_snapshot_invariance() {
    let t0 = Instant::now();
    let timer = CountdownTimer::new(Duration::from_secs(60)).unwrap();
    let s0 = timer.snapshot(t0);
    let s1 = timer.snapshot(t0 + Duration::from_secs(86400 * 365));
    assert_eq!(s0.status, TimerStatus::Ready);
    assert_eq!(s0.remaining, Duration::from_secs(60));
    assert_eq!(s1.status, TimerStatus::Ready);
    assert_eq!(s1.remaining, Duration::from_secs(60));
}

#[test]
fn test_e07_two_snapshots_at_same_instant() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(10)).unwrap();
    timer.start(t0).unwrap();
    let s1 = timer.snapshot(t0 + Duration::from_secs(3));
    let s2 = timer.snapshot(t0 + Duration::from_secs(3));
    assert_eq!(s1, s2);
    assert_eq!(s1.remaining, Duration::from_secs(7));
    assert_eq!(s1.status, TimerStatus::Running);
}

#[test]
fn test_e08_running_with_past_timestamp() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(10)).unwrap();
    let t_start = t0 + Duration::from_secs(5);
    timer.start(t_start).unwrap();
    // Observation timestamp before started_at
    let snap = timer.snapshot(t0);
    assert_eq!(snap.status, TimerStatus::Running);
    assert_eq!(snap.remaining, Duration::from_secs(10));
}

#[test]
fn test_e09_huge_forward_synthetic_jump() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(10)).unwrap();
    timer.start(t0).unwrap();
    let huge_future = t0 + Duration::from_secs(86400 * 365 * 10);
    let snap = timer.snapshot(huge_future);
    assert_eq!(snap.status, TimerStatus::Finished);
    assert_eq!(snap.remaining, Duration::ZERO);
    assert_eq!(timer.poll(huge_future), Some(TimerEvent::Expired));
    assert_eq!(timer.poll(huge_future), None);
}

#[test]
fn test_e10_immutable_initial_duration_across_lifecycle() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(15)).unwrap();
    assert_eq!(timer.initial_duration(), Duration::from_secs(15));
    timer.start(t0).unwrap();
    assert_eq!(timer.initial_duration(), Duration::from_secs(15));
    timer.pause(t0 + Duration::from_secs(5)).unwrap();
    assert_eq!(timer.initial_duration(), Duration::from_secs(15));
    timer.resume(t0 + Duration::from_secs(10)).unwrap();
    assert_eq!(timer.initial_duration(), Duration::from_secs(15));
    timer.poll(t0 + Duration::from_secs(25));
    assert_eq!(timer.initial_duration(), Duration::from_secs(15));
    timer.reset();
    assert_eq!(timer.initial_duration(), Duration::from_secs(15));
}

#[test]
fn test_s01_to_s14_state_transitions() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(10)).unwrap();

    // S01: Ready -> start(t0)
    timer.start(t0).unwrap();
    let snap = timer.snapshot(t0);
    assert_eq!(snap.status, TimerStatus::Running);
    assert_eq!(snap.remaining, Duration::from_secs(10));

    // Reset for S02, S03
    timer.reset();
    // S02: Ready -> pause(t0)
    assert_eq!(
        timer.pause(t0).unwrap_err(),
        TimerError::InvalidTransition {
            action: TimerAction::Pause,
            status: TimerStatus::Ready
        }
    );
    assert_eq!(timer.snapshot(t0).status, TimerStatus::Ready);

    // S03: Ready -> resume(t0)
    assert_eq!(
        timer.resume(t0).unwrap_err(),
        TimerError::InvalidTransition {
            action: TimerAction::Resume,
            status: TimerStatus::Ready
        }
    );
    assert_eq!(timer.snapshot(t0).status, TimerStatus::Ready);

    // Start timer for S04-S06
    timer.start(t0).unwrap();

    // S04: Running -> start(t0+1s)
    assert_eq!(
        timer.start(t0 + Duration::from_secs(1)).unwrap_err(),
        TimerError::InvalidTransition {
            action: TimerAction::Start,
            status: TimerStatus::Running
        }
    );
    // S05: Running -> resume(t0+1s)
    assert_eq!(
        timer.resume(t0 + Duration::from_secs(1)).unwrap_err(),
        TimerError::InvalidTransition {
            action: TimerAction::Resume,
            status: TimerStatus::Running
        }
    );

    // S06: Running -> pause(t0+2.25s)
    let t_pause = t0 + Duration::from_millis(2250);
    timer.pause(t_pause).unwrap();
    let snap = timer.snapshot(t_pause);
    assert_eq!(snap.status, TimerStatus::Paused);
    assert_eq!(snap.remaining, Duration::from_millis(7750));

    // S07: Paused snapshot at t0+24h
    let snap_future = timer.snapshot(t0 + Duration::from_secs(86400));
    assert_eq!(snap_future.status, TimerStatus::Paused);
    assert_eq!(snap_future.remaining, Duration::from_millis(7750));

    // S08: Paused -> pause
    assert_eq!(
        timer.pause(t0 + Duration::from_secs(10)).unwrap_err(),
        TimerError::InvalidTransition {
            action: TimerAction::Pause,
            status: TimerStatus::Paused
        }
    );

    // S09: Paused -> start
    assert_eq!(
        timer.start(t0 + Duration::from_secs(10)).unwrap_err(),
        TimerError::InvalidTransition {
            action: TimerAction::Start,
            status: TimerStatus::Paused
        }
    );

    // S10: Paused -> resume(t1)
    let t1 = t0 + Duration::from_secs(100);
    timer.resume(t1).unwrap();
    let snap_resume = timer.snapshot(t1);
    assert_eq!(snap_resume.status, TimerStatus::Running);
    assert_eq!(snap_resume.remaining, Duration::from_millis(7750));

    // S11: After S10, at t1+7.749999999s: 1ns left, Running; poll returns None
    let just_before = t1 + Duration::from_nanos(7_749_999_999);
    let snap_before = timer.snapshot(just_before);
    assert_eq!(snap_before.status, TimerStatus::Running);
    assert_eq!(snap_before.remaining, Duration::from_nanos(1));
    assert_eq!(timer.poll(just_before), None);

    // S12: After S10, at t1+7.75s: poll returns Some(Expired), Finished, zero
    let t_exact = t1 + Duration::from_millis(7750);
    assert_eq!(timer.poll(t_exact), Some(TimerEvent::Expired));
    let snap_exact = timer.snapshot(t_exact);
    assert_eq!(snap_exact.status, TimerStatus::Finished);
    assert_eq!(snap_exact.remaining, Duration::ZERO);

    // S13: Poll same Finished again and at t1+1000s
    assert_eq!(timer.poll(t_exact), None);
    assert_eq!(timer.poll(t1 + Duration::from_secs(1000)), None);

    // S14: Finished -> start/pause/resume
    assert_eq!(
        timer.start(t_exact).unwrap_err(),
        TimerError::InvalidTransition {
            action: TimerAction::Start,
            status: TimerStatus::Finished
        }
    );
    assert_eq!(
        timer.pause(t_exact).unwrap_err(),
        TimerError::InvalidTransition {
            action: TimerAction::Pause,
            status: TimerStatus::Finished
        }
    );
    assert_eq!(
        timer.resume(t_exact).unwrap_err(),
        TimerError::InvalidTransition {
            action: TimerAction::Resume,
            status: TimerStatus::Finished
        }
    );
}

#[test]
fn test_s15_to_s18_resets() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(10)).unwrap();

    // S15: Reset from Ready
    timer.reset();
    assert_eq!(timer.snapshot(t0).status, TimerStatus::Ready);
    assert_eq!(timer.snapshot(t0).remaining, Duration::from_secs(10));

    // S16: Reset from Running
    timer.start(t0).unwrap();
    timer.reset();
    assert_eq!(timer.snapshot(t0).status, TimerStatus::Ready);
    assert_eq!(timer.snapshot(t0).remaining, Duration::from_secs(10));
    assert_eq!(timer.poll(t0 + Duration::from_secs(15)), None);

    // S17: Reset from Paused
    timer.start(t0).unwrap();
    timer.pause(t0 + Duration::from_secs(2)).unwrap();
    timer.reset();
    assert_eq!(timer.snapshot(t0).status, TimerStatus::Ready);
    assert_eq!(timer.snapshot(t0).remaining, Duration::from_secs(10));

    // S18: Reset from Finished
    timer.start(t0).unwrap();
    assert_eq!(
        timer.poll(t0 + Duration::from_secs(10)),
        Some(TimerEvent::Expired)
    );
    timer.reset();
    assert_eq!(timer.snapshot(t0).status, TimerStatus::Ready);
    assert_eq!(timer.snapshot(t0).remaining, Duration::from_secs(10));
    // Fresh start after reset can finish once
    timer.start(t0 + Duration::from_secs(20)).unwrap();
    assert_eq!(
        timer.poll(t0 + Duration::from_secs(30)),
        Some(TimerEvent::Expired)
    );
    assert_eq!(timer.poll(t0 + Duration::from_secs(30)), None);
}

#[test]
fn test_s19_s20_snapshot_at_deadline_before_poll() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(10)).unwrap();
    timer.start(t0).unwrap();

    let deadline = t0 + Duration::from_secs(10);
    // S19: Snapshot reports Finished without mutating or consuming event
    let snap = timer.snapshot(deadline);
    assert_eq!(snap.status, TimerStatus::Finished);
    assert_eq!(snap.remaining, Duration::ZERO);

    // S20: Later poll at same deadline emits exactly once
    assert_eq!(timer.poll(deadline), Some(TimerEvent::Expired));
    assert_eq!(timer.poll(deadline), None);
}

#[test]
fn test_s21_late_pause_at_deadline_rejects_and_preserves_event() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(10)).unwrap();
    timer.start(t0).unwrap();

    let deadline = t0 + Duration::from_secs(10);
    // S21: Pause at deadline returns DeadlineReached without swallowing event
    assert_eq!(
        timer.pause(deadline).unwrap_err(),
        TimerError::DeadlineReached
    );

    // Subsequent poll still emits Expired
    assert_eq!(timer.poll(deadline), Some(TimerEvent::Expired));
    assert_eq!(timer.poll(deadline), None);
}

#[test]
fn test_s22_pause_1ns_before_deadline() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(10)).unwrap();
    timer.start(t0).unwrap();

    let just_before = t0 + Duration::from_nanos(9_999_999_999);
    timer.pause(just_before).unwrap();
    let snap = timer.snapshot(just_before);
    assert_eq!(snap.status, TimerStatus::Paused);
    assert_eq!(snap.remaining, Duration::from_nanos(1));

    let t1 = t0 + Duration::from_secs(50);
    timer.resume(t1).unwrap();
    assert_eq!(timer.poll(t1), None);
    assert_eq!(
        timer.poll(t1 + Duration::from_nanos(1)),
        Some(TimerEvent::Expired)
    );
}

#[test]
fn test_s23_long_pause_zero_drift() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(10)).unwrap();
    timer.start(t0).unwrap();

    // Run 1 second
    timer.pause(t0 + Duration::from_secs(1)).unwrap();
    assert_eq!(
        timer.snapshot(t0 + Duration::from_secs(1)).remaining,
        Duration::from_secs(9)
    );

    // Pause for 5 years
    let five_years = Duration::from_secs(86400 * 365 * 5);
    let t1 = t0 + five_years;
    assert_eq!(timer.snapshot(t1).status, TimerStatus::Paused);
    assert_eq!(timer.snapshot(t1).remaining, Duration::from_secs(9));

    // Resume and run 2 seconds
    timer.resume(t1).unwrap();
    let t2 = t1 + Duration::from_secs(2);
    let snap = timer.snapshot(t2);
    assert_eq!(snap.status, TimerStatus::Running);
    assert_eq!(snap.remaining, Duration::from_secs(7));

    // Run remaining 7 seconds
    assert_eq!(
        timer.poll(t1 + Duration::from_secs(9)),
        Some(TimerEvent::Expired)
    );
}

#[test]
fn test_s24_poll_while_ready_or_paused() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(10)).unwrap();
    assert_eq!(timer.poll(t0), None);
    assert_eq!(timer.snapshot(t0).status, TimerStatus::Ready);

    timer.start(t0).unwrap();
    timer.pause(t0 + Duration::from_secs(2)).unwrap();
    assert_eq!(timer.poll(t0 + Duration::from_secs(20)), None);
    assert_eq!(
        timer.snapshot(t0 + Duration::from_secs(20)).status,
        TimerStatus::Paused
    );
}

#[test]
fn test_s25_repeated_snapshots_across_deadline_without_poll() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(5)).unwrap();
    timer.start(t0).unwrap();

    let past_deadline = t0 + Duration::from_secs(8);
    for _ in 0..10 {
        let snap = timer.snapshot(past_deadline);
        assert_eq!(snap.status, TimerStatus::Finished);
        assert_eq!(snap.remaining, Duration::ZERO);
    }
    // Poll still emits exactly once
    assert_eq!(timer.poll(past_deadline), Some(TimerEvent::Expired));
    assert_eq!(timer.poll(past_deadline), None);
}

#[test]
fn test_s26_reset_logically_expired_unpolled_cancels_event() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(5)).unwrap();
    timer.start(t0).unwrap();

    let past_deadline = t0 + Duration::from_secs(8);
    assert_eq!(timer.snapshot(past_deadline).status, TimerStatus::Finished);

    // Reset before poll
    timer.reset();
    assert_eq!(timer.snapshot(past_deadline).status, TimerStatus::Ready);
    assert_eq!(timer.poll(past_deadline), None);
}

#[test]
fn test_s27_invalid_transitions_are_atomic() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(10)).unwrap();

    // Start timer, run 3s
    timer.start(t0).unwrap();
    let t1 = t0 + Duration::from_secs(3);

    // Invalid start from Running
    let err = timer.start(t1).unwrap_err();
    assert_eq!(
        err,
        TimerError::InvalidTransition {
            action: TimerAction::Start,
            status: TimerStatus::Running
        }
    );
    // Verify state unchanged
    assert_eq!(timer.snapshot(t1).status, TimerStatus::Running);
    assert_eq!(timer.snapshot(t1).remaining, Duration::from_secs(7));

    // Invalid resume from Running
    let err = timer.resume(t1).unwrap_err();
    assert_eq!(
        err,
        TimerError::InvalidTransition {
            action: TimerAction::Resume,
            status: TimerStatus::Running
        }
    );
    assert_eq!(timer.snapshot(t1).status, TimerStatus::Running);
    assert_eq!(timer.snapshot(t1).remaining, Duration::from_secs(7));
}

#[test]
fn test_s28_start_reset_start_finish() {
    let t0 = Instant::now();
    let mut timer = CountdownTimer::new(Duration::from_secs(5)).unwrap();

    // First epoch
    timer.start(t0).unwrap();
    timer.reset();

    // Second epoch
    let t1 = t0 + Duration::from_secs(10);
    timer.start(t1).unwrap();
    let end = t1 + Duration::from_secs(5);
    assert_eq!(timer.poll(end), Some(TimerEvent::Expired));
    assert_eq!(timer.poll(end), None);
}
