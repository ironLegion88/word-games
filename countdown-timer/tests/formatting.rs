use std::time::Duration;

use countdown_timer::{display_seconds_ceil, format_countdown};

#[test]
fn test_f01_zero() {
    assert_eq!(display_seconds_ceil(Duration::ZERO), 0);
    assert_eq!(format_countdown(Duration::ZERO), "00:00");
}

#[test]
fn test_f02_one_nanosecond() {
    assert_eq!(display_seconds_ceil(Duration::from_nanos(1)), 1);
    assert_eq!(format_countdown(Duration::from_nanos(1)), "00:01");
}

#[test]
fn test_f03_999_milliseconds() {
    assert_eq!(display_seconds_ceil(Duration::from_millis(999)), 1);
    assert_eq!(format_countdown(Duration::from_millis(999)), "00:01");
}

#[test]
fn test_f04_one_second() {
    assert_eq!(display_seconds_ceil(Duration::from_secs(1)), 1);
    assert_eq!(format_countdown(Duration::from_secs(1)), "00:01");
}

#[test]
fn test_f05_one_second_one_nanosecond() {
    let d = Duration::from_secs(1) + Duration::from_nanos(1);
    assert_eq!(display_seconds_ceil(d), 2);
    assert_eq!(format_countdown(d), "00:02");
}

#[test]
fn test_f06_59_seconds() {
    assert_eq!(display_seconds_ceil(Duration::from_secs(59)), 59);
    assert_eq!(format_countdown(Duration::from_secs(59)), "00:59");
}

#[test]
fn test_f07_59_seconds_one_nanosecond() {
    let d = Duration::from_secs(59) + Duration::from_nanos(1);
    assert_eq!(display_seconds_ceil(d), 60);
    assert_eq!(format_countdown(d), "01:00");
}

#[test]
fn test_f08_60_seconds() {
    assert_eq!(display_seconds_ceil(Duration::from_secs(60)), 60);
    assert_eq!(format_countdown(Duration::from_secs(60)), "01:00");
}

#[test]
fn test_f09_60_seconds_one_nanosecond() {
    let d = Duration::from_secs(60) + Duration::from_nanos(1);
    assert_eq!(display_seconds_ceil(d), 61);
    assert_eq!(format_countdown(d), "01:01");
}

#[test]
fn test_f10_3599_seconds() {
    assert_eq!(display_seconds_ceil(Duration::from_secs(3599)), 3599);
    assert_eq!(format_countdown(Duration::from_secs(3599)), "59:59");
}

#[test]
fn test_f11_3599_seconds_one_nanosecond() {
    let d = Duration::from_secs(3599) + Duration::from_nanos(1);
    assert_eq!(display_seconds_ceil(d), 3600);
    assert_eq!(format_countdown(d), "01:00:00");
}

#[test]
fn test_f12_3600_seconds() {
    assert_eq!(display_seconds_ceil(Duration::from_secs(3600)), 3600);
    assert_eq!(format_countdown(Duration::from_secs(3600)), "01:00:00");
}

#[test]
fn test_f13_3601_seconds() {
    assert_eq!(display_seconds_ceil(Duration::from_secs(3601)), 3601);
    assert_eq!(format_countdown(Duration::from_secs(3601)), "01:00:01");
}

#[test]
fn test_f14_86400_seconds() {
    assert_eq!(display_seconds_ceil(Duration::from_secs(86400)), 86400);
    assert_eq!(format_countdown(Duration::from_secs(86400)), "24:00:00");
}

#[test]
fn test_f15_100_hours() {
    let d = Duration::from_secs(100 * 3600);
    assert_eq!(display_seconds_ceil(d), 360000);
    assert_eq!(format_countdown(d), "100:00:00");
}

#[test]
fn test_f16_very_large_duration() {
    let d = Duration::MAX;
    let secs = display_seconds_ceil(d);
    assert_eq!(secs, u64::MAX);
    let s = format_countdown(d);
    assert!(!s.is_empty());
}
