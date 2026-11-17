use std::time::Duration;

/// Returns the whole number of remaining seconds rounded up on any nonzero fractional nanoseconds.
///
/// Returns zero if and only if `remaining` is zero.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use countdown_timer::display_seconds_ceil;
///
/// assert_eq!(display_seconds_ceil(Duration::ZERO), 0);
/// assert_eq!(display_seconds_ceil(Duration::from_nanos(1)), 1);
/// assert_eq!(display_seconds_ceil(Duration::from_millis(999)), 1);
/// assert_eq!(display_seconds_ceil(Duration::from_secs(1)), 1);
/// assert_eq!(display_seconds_ceil(Duration::from_secs(1) + Duration::from_nanos(1)), 2);
/// ```
#[must_use]
pub fn display_seconds_ceil(remaining: Duration) -> u64 {
    if remaining.subsec_nanos() > 0 {
        remaining.as_secs().saturating_add(1)
    } else {
        remaining.as_secs()
    }
}

/// Formats a duration as a countdown clock string using rounded-up whole seconds.
///
/// Durations under one hour are formatted as `MM:SS`. Durations of one hour or more
/// are formatted as `HH:MM:SS` with at least two digits for hours.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use countdown_timer::format_countdown;
///
/// assert_eq!(format_countdown(Duration::ZERO), "00:00");
/// assert_eq!(format_countdown(Duration::from_nanos(1)), "00:01");
/// assert_eq!(format_countdown(Duration::from_secs(59)), "00:59");
/// assert_eq!(format_countdown(Duration::from_secs(60)), "01:00");
/// assert_eq!(format_countdown(Duration::from_secs(3599)), "59:59");
/// assert_eq!(format_countdown(Duration::from_secs(3600)), "01:00:00");
/// assert_eq!(format_countdown(Duration::from_secs(86400)), "24:00:00");
/// ```
#[must_use]
pub fn format_countdown(duration: Duration) -> String {
    let total_seconds = display_seconds_ceil(duration);
    if total_seconds < 3600 {
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{minutes:02}:{seconds:02}")
    } else {
        let hours = total_seconds / 3600;
        let remainder = total_seconds % 3600;
        let minutes = remainder / 60;
        let seconds = remainder % 60;
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_seconds_ceil_boundaries() {
        assert_eq!(display_seconds_ceil(Duration::ZERO), 0);
        assert_eq!(display_seconds_ceil(Duration::from_nanos(1)), 1);
        assert_eq!(display_seconds_ceil(Duration::from_millis(999)), 1);
        assert_eq!(display_seconds_ceil(Duration::from_secs(1)), 1);
        assert_eq!(
            display_seconds_ceil(Duration::from_secs(1) + Duration::from_nanos(1)),
            2
        );
        assert_eq!(display_seconds_ceil(Duration::MAX), u64::MAX);
    }

    #[test]
    fn test_format_countdown_table() {
        // Table F01-F16
        assert_eq!(format_countdown(Duration::ZERO), "00:00");
        assert_eq!(format_countdown(Duration::from_nanos(1)), "00:01");
        assert_eq!(format_countdown(Duration::from_millis(999)), "00:01");
        assert_eq!(format_countdown(Duration::from_secs(1)), "00:01");
        assert_eq!(
            format_countdown(Duration::from_secs(1) + Duration::from_nanos(1)),
            "00:02"
        );
        assert_eq!(format_countdown(Duration::from_secs(59)), "00:59");
        assert_eq!(
            format_countdown(Duration::from_secs(59) + Duration::from_nanos(1)),
            "01:00"
        );
        assert_eq!(format_countdown(Duration::from_secs(60)), "01:00");
        assert_eq!(
            format_countdown(Duration::from_secs(60) + Duration::from_nanos(1)),
            "01:01"
        );
        assert_eq!(format_countdown(Duration::from_secs(3599)), "59:59");
        assert_eq!(
            format_countdown(Duration::from_secs(3599) + Duration::from_nanos(1)),
            "01:00:00"
        );
        assert_eq!(format_countdown(Duration::from_secs(3600)), "01:00:00");
        assert_eq!(format_countdown(Duration::from_secs(3601)), "01:00:01");
        assert_eq!(format_countdown(Duration::from_secs(86400)), "24:00:00");
        assert_eq!(
            format_countdown(Duration::from_secs(100 * 3600)),
            "100:00:00"
        );
    }
}
