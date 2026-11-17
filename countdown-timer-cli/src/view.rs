use countdown_timer::{format_countdown, TimerSnapshot, TimerStatus};

/// Generates lines of text representing the terminal view given the current snapshot and dimensions.
///
/// This function is pure and never performs terminal I/O. It guarantees no panics on small,
/// zero, or unusual dimensions through checked/saturating arithmetic.
#[must_use]
pub fn render_view(
    snapshot: TimerSnapshot,
    message: Option<&str>,
    width: u16,
    height: u16,
) -> Vec<String> {
    let w = width as usize;
    let h = height as usize;

    if w == 0 || h == 0 {
        return Vec::new();
    }

    let status_str = match snapshot.status {
        TimerStatus::Ready => "READY",
        TimerStatus::Running => "RUNNING",
        TimerStatus::Paused => "PAUSED",
        TimerStatus::Finished => "FINISHED",
    };

    let remaining_str = format_countdown(snapshot.remaining);
    let original_str = format!("Original: {}", format_countdown(snapshot.initial));
    let controls_str = "[Space] Toggle   [r] Reset   [q/Esc] Quit";

    // Compact fallback for very constrained viewports
    if h < 6 || w < 30 {
        let mut lines = Vec::new();
        let line1 = format!("[{status_str}] {remaining_str}");
        lines.push(clip_or_pad(&line1, w));
        if h >= 2 {
            lines.push(clip_or_pad(&original_str, w));
        }
        if h >= 3 {
            let line3 = message.unwrap_or(controls_str);
            lines.push(clip_or_pad(line3, w));
        }
        return lines;
    }

    // Normal multi-line layout
    let mut content_lines = Vec::new();
    content_lines.push("COUNTDOWN TIMER".to_string());
    content_lines.push(String::new());
    content_lines.push(format!("[ {status_str} ]"));
    content_lines.push(String::new());
    content_lines.push(format!("    {remaining_str}    "));
    content_lines.push(String::new());
    content_lines.push(format!("({original_str})"));
    content_lines.push(String::new());

    if let Some(msg) = message {
        content_lines.push(msg.to_string());
    } else {
        content_lines.push(String::new());
    }

    content_lines.push(String::new());
    content_lines.push(controls_str.to_string());

    // Center content vertically and horizontally within (w, h)
    let content_len = content_lines.len();
    let vertical_pad = h.saturating_sub(content_len) / 2;

    let mut result = Vec::with_capacity(h);
    for _ in 0..vertical_pad {
        result.push(String::new());
    }

    for line in content_lines {
        if result.len() < h {
            result.push(center_line(&line, w));
        }
    }

    while result.len() < h {
        result.push(String::new());
    }

    result
}

fn clip_or_pad(s: &str, width: usize) -> String {
    if s.len() > width {
        s.chars().take(width).collect()
    } else {
        s.to_string()
    }
}

fn center_line(line: &str, width: usize) -> String {
    if line.len() >= width {
        clip_or_pad(line, width)
    } else {
        let pad = (width.saturating_sub(line.len())) / 2;
        let mut s = String::with_capacity(width);
        for _ in 0..pad {
            s.push(' ');
        }
        s.push_str(line);
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_render_view_dimensions() {
        let snap = TimerSnapshot {
            initial: Duration::from_secs(90),
            remaining: Duration::from_secs(90),
            status: TimerStatus::Ready,
        };

        // Width/height 0
        assert!(render_view(snap, None, 0, 0).is_empty());
        assert!(render_view(snap, None, 0, 20).is_empty());
        assert!(render_view(snap, None, 80, 0).is_empty());

        // Tiny viewport
        let lines_tiny = render_view(snap, None, 15, 2);
        assert_eq!(lines_tiny.len(), 2);
        assert!(lines_tiny[0].contains("READY"));

        // Normal viewport
        let lines_normal = render_view(snap, Some("Time's up!"), 80, 24);
        assert_eq!(lines_normal.len(), 24);
        let joined = lines_normal.join("\n");
        assert!(joined.contains("COUNTDOWN TIMER"));
        assert!(joined.contains("01:30"));
        assert!(joined.contains("Original: 01:30"));
        assert!(joined.contains("Time's up!"));
    }
}
