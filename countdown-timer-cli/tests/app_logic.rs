use std::io::Cursor;
use std::time::{Duration, Instant};

use countdown_timer::TimerStatus;
use countdown_timer_cli::{map_key_event, prompt_duration, render_view, App, AppCommand};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

fn make_key_event(code: KeyCode, kind: KeyEventKind, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind,
        state: KeyEventState::NONE,
    }
}

#[test]
fn test_u01_initial_app_state() {
    let app = App::new(Duration::from_secs(90)).unwrap();
    let snap = app.timer.snapshot(Instant::now());
    assert_eq!(snap.status, TimerStatus::Ready);
    assert_eq!(snap.remaining, Duration::from_secs(90));
    assert_eq!(app.message, None);
    assert!(!app.should_quit);
    assert!(app.needs_redraw);
}

#[test]
fn test_u02_to_u05_space_toggle_cycle() {
    let t0 = Instant::now();
    let mut app = App::new(Duration::from_secs(10)).unwrap();

    // U02: Space in Ready -> Running
    app.on_command(AppCommand::Toggle, t0);
    assert_eq!(app.timer.snapshot(t0).status, TimerStatus::Running);

    // U03: Space during Running -> Paused
    let t1 = t0 + Duration::from_secs(3);
    app.on_command(AppCommand::Toggle, t1);
    let snap_paused = app.timer.snapshot(t1);
    assert_eq!(snap_paused.status, TimerStatus::Paused);
    assert_eq!(snap_paused.remaining, Duration::from_secs(7));

    // U04: Space during Paused -> Running at fresh timestamp
    let t2 = t1 + Duration::from_secs(20);
    app.on_command(AppCommand::Toggle, t2);
    let snap_resumed = app.timer.snapshot(t2);
    assert_eq!(snap_resumed.status, TimerStatus::Running);
    assert_eq!(snap_resumed.remaining, Duration::from_secs(7));

    // Run to finished
    let t3 = t2 + Duration::from_secs(7);
    app.on_tick(t3);
    assert_eq!(app.timer.snapshot(t3).status, TimerStatus::Finished);
    assert_eq!(app.message, Some("Time's up!".to_string()));

    // U05: Space during Finished -> No action; reset hint displayed
    app.on_command(AppCommand::Toggle, t3 + Duration::from_secs(1));
    assert_eq!(app.timer.snapshot(t3).status, TimerStatus::Finished);
    assert!(app.message.as_ref().unwrap().contains("reset"));
}

#[test]
fn test_u06_reset_from_each_state() {
    let t0 = Instant::now();

    // Reset from Ready
    let mut app = App::new(Duration::from_secs(30)).unwrap();
    app.on_command(AppCommand::Reset, t0);
    assert_eq!(app.timer.snapshot(t0).status, TimerStatus::Ready);
    assert_eq!(app.timer.snapshot(t0).remaining, Duration::from_secs(30));

    // Reset from Running
    app.on_command(AppCommand::Toggle, t0);
    app.on_command(AppCommand::Reset, t0 + Duration::from_secs(5));
    assert_eq!(app.timer.snapshot(t0).status, TimerStatus::Ready);
    assert_eq!(app.timer.snapshot(t0).remaining, Duration::from_secs(30));

    // Reset from Paused
    app.on_command(AppCommand::Toggle, t0);
    app.on_command(AppCommand::Toggle, t0 + Duration::from_secs(5));
    app.on_command(AppCommand::Reset, t0 + Duration::from_secs(10));
    assert_eq!(app.timer.snapshot(t0).status, TimerStatus::Ready);
    assert_eq!(app.timer.snapshot(t0).remaining, Duration::from_secs(30));

    // Reset from Finished clears message
    app.on_command(AppCommand::Toggle, t0);
    app.on_tick(t0 + Duration::from_secs(30));
    assert_eq!(app.message, Some("Time's up!".to_string()));
    app.on_command(AppCommand::Reset, t0 + Duration::from_secs(35));
    assert_eq!(app.timer.snapshot(t0).status, TimerStatus::Ready);
    assert_eq!(app.timer.snapshot(t0).remaining, Duration::from_secs(30));
    assert_eq!(app.message, None);
}

#[test]
fn test_u07_quit_keys() {
    let keys = [
        make_key_event(KeyCode::Char('q'), KeyEventKind::Press, KeyModifiers::NONE),
        make_key_event(KeyCode::Char('Q'), KeyEventKind::Press, KeyModifiers::NONE),
        make_key_event(KeyCode::Esc, KeyEventKind::Press, KeyModifiers::NONE),
        make_key_event(
            KeyCode::Char('c'),
            KeyEventKind::Press,
            KeyModifiers::CONTROL,
        ),
        make_key_event(
            KeyCode::Char('C'),
            KeyEventKind::Press,
            KeyModifiers::CONTROL,
        ),
    ];

    for key in keys {
        assert_eq!(map_key_event(key), Some(AppCommand::Quit));
    }
}

#[test]
fn test_u08_key_repeat_and_release_ignored() {
    let repeat = make_key_event(KeyCode::Char(' '), KeyEventKind::Repeat, KeyModifiers::NONE);
    let release = make_key_event(
        KeyCode::Char(' '),
        KeyEventKind::Release,
        KeyModifiers::NONE,
    );
    assert_eq!(map_key_event(repeat), None);
    assert_eq!(map_key_event(release), None);

    let other_key = make_key_event(KeyCode::Char('x'), KeyEventKind::Press, KeyModifiers::NONE);
    assert_eq!(map_key_event(other_key), None);
}

#[test]
fn test_u09_resize_sets_redraw() {
    let mut app = App::new(Duration::from_secs(10)).unwrap();
    app.needs_redraw = false;
    app.on_resize();
    assert!(app.needs_redraw);
}

#[test]
fn test_u10_action_at_exact_deadline_polls_first() {
    let t0 = Instant::now();
    let mut app = App::new(Duration::from_secs(10)).unwrap();
    app.on_command(AppCommand::Toggle, t0); // Start

    let deadline = t0 + Duration::from_secs(10);
    // User presses Space at exact deadline
    app.on_command(AppCommand::Toggle, deadline);

    // Poll must commit Finished and display completion
    let snap = app.timer.snapshot(deadline);
    assert_eq!(snap.status, TimerStatus::Finished);
    assert!(app.message.is_some());
}

#[test]
fn test_u11_u12_completion_message_emitted_once() {
    let t0 = Instant::now();
    let mut app = App::new(Duration::from_secs(5)).unwrap();
    app.on_command(AppCommand::Toggle, t0);

    let deadline = t0 + Duration::from_secs(5);
    app.on_tick(deadline);
    assert_eq!(app.message, Some("Time's up!".to_string()));

    // Clear needs_redraw and tick again
    app.needs_redraw = false;
    app.on_tick(deadline + Duration::from_secs(1));
    // No repeated expiration trigger
    assert_eq!(app.message, Some("Time's up!".to_string()));
    assert!(!app.needs_redraw);
}

#[test]
fn test_u13_u14_view_formatting_and_initial_duration_visible() {
    let t0 = Instant::now();
    let mut app = App::new(Duration::from_secs(3665)).unwrap();
    app.on_command(AppCommand::Toggle, t0);

    let t1 = t0 + Duration::from_secs(65);
    let snap = app.timer.snapshot(t1);
    let rendered = render_view(snap, app.message.as_deref(), 80, 24);
    let joined = rendered.join("\n");

    // U14: Original duration visible
    assert!(joined.contains("Original: 01:01:05"));
    // U16: Remaining duration formatted over one hour
    assert!(joined.contains("01:00:00"));
}

#[test]
fn test_u15_small_and_unusual_dimensions() {
    let snap = App::new(Duration::from_secs(10))
        .unwrap()
        .timer
        .snapshot(Instant::now());

    // Should not panic on any of these
    let _ = render_view(snap, None, 0, 0);
    let _ = render_view(snap, None, 1, 1);
    let _ = render_view(snap, None, 5, 2);
    let _ = render_view(snap, None, 20, 5);
    let _ = render_view(snap, None, 300, 100);
}

#[test]
fn test_interactive_prompt_duration() {
    // Valid on first try
    let mut input = Cursor::new("42\n");
    let mut output = Vec::new();
    let res = prompt_duration(&mut input, &mut output).unwrap();
    assert_eq!(res, Some(Duration::from_secs(42)));

    // Invalid then valid retry
    let mut input_retry = Cursor::new("invalid\n0\n90\n");
    let mut output_retry = Vec::new();
    let res_retry = prompt_duration(&mut input_retry, &mut output_retry).unwrap();
    assert_eq!(res_retry, Some(Duration::from_secs(90)));
    let out_str = String::from_utf8_lossy(&output_retry);
    assert!(out_str.contains("Invalid input"));

    // Clean EOF
    let mut input_eof = Cursor::new("");
    let mut output_eof = Vec::new();
    let res_eof = prompt_duration(&mut input_eof, &mut output_eof).unwrap();
    assert_eq!(res_eof, None);
}
