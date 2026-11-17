# Implementation Agent Prompt — Countdown Timer (Core + Standalone CLI)

You are an experienced **Rust systems engineer, terminal application engineer, test engineer and responsible Git maintainer**. You are implementing a reusable countdown timer and standalone interactive terminal application in the existing `https://github.com/ironLegion88/word-games` repository.

## 1. Your exact task

Implement **only**:

1. A `countdown-timer` **library crate**: clock-correct, deterministic, `std`-only Rust engine with start, pause, resume, reset, pure snapshots, exact remaining duration and one-time expiration event.
2. A `countdown-timer-cli` **standalone binary crate**: duration selection, prominent live remaining-time display, original duration, status, Space start/pause/resume, `r` reset, `q`/Esc/Ctrl+C quit, visible completion, responsive cross-platform terminal behavior.
3. The corresponding tests, Rustdoc, usage documentation and truthful verification record.

**Hard scope boundary:** Do **not** modify, wrap, extend, integrate, or time `spelling-bee`, `boggle`, `wordle`, `word-games-cli`, `trie`, `trie-cli`, any game session, solver, benchmark or test. No game integration at all. Do not build multi-timer/concurrent timers, sequential routines, sound, GUI, Ratatui, Tokio, wall-clock persistence, networking, or future scaffolding. The independent timer will be integrated elsewhere in a separate task.

## 2. Required documents — read completely before editing

Read, in order:

1. The provided *Countdown Timer* one-page assignment PDF (mandatory functionality vs optional extensions).
2. `docs/timer/DESIGN_SPEC.md` — **binding architecture, public API signatures and semantics**.
3. `docs/timer/IMPLEMENTATION_PLAN.md` — **binding phase ordering, implementation details and acceptance gates**.
4. `docs/timer/TESTING.md` — **binding deterministic test matrix, terminal checks and verification record**.
5. Existing root `Cargo.toml`, `Cargo.lock`, `README.md`, `.github/workflows/ci.yml`, `docs/TESTING.md`, `.gitignore`, and current repository history/status.

Do not substitute older `docs/DESIGN_SPECIFICATION.md`, `docs/IMPLEMENTATION_PLAN.md`, existing game agent prompts, or archived work for the timer-specific documents. If one of the three timer documents is missing, locate the supplied file rather than inventing its contents. The four files in this package are designed to be committed to the paths above; do not overwrite unrelated existing files.

## 3. Binding architecture and decisions

- Use `Duration` as the authoritative quantity and caller-provided `Instant` for clock observations. **Never** decrement a mutable second counter, count render ticks as time, use `SystemTime` for elapsed time, or call `Instant::now()` inside the core.
- Model a private state enum with `Ready`, `Running { started_at, remaining_at_start }`, `Paused { remaining }`, and `Finished`. Keep the original duration immutable after construction.
- The engine exposes typed public `TimerStatus`, `TimerSnapshot`, `TimerEvent`, `TimerAction`, `TimerError`; use the **exact** public method names/signatures in the design spec, including `poll(now)` (not `tick`) and `display_seconds_ceil`.
- Core rejects zero durations, but doesn't impose the CLI's 24-hour maximum. CLI input is integer seconds `1..=86400`.
- `snapshot(&self, now)` is nonmutating and may report **logical Finished** when a Running timer has exceeded the deadline; it **must not consume an event**. Only `poll(&mut self, now)` commits internal Finished and emits `Some(Expired)` once. Late `pause(now)` returns `DeadlineReached` without consuming the pending event. Reset explicitly cancels pending events.
- Use `&self` for read-only snapshots/formatting, `&mut self` for transitions, one owner for each live timer; no `Rc`, `Arc`, `Mutex`, threads, callbacks, interior mutability, trait objects or unnecessary generic clock abstractions.
- For remaining time calculate `budget.saturating_sub(now.saturating_duration_since(started_at))`; avoid `Instant` deadline addition. Pure time formatting rounds fractional seconds upward so the UI never shows zero before expiration.
- Core has **no dependencies** beyond `std`; CLI depends on the core and a suitable version of Crossterm (design recommends `0.29`). Do not add Clap, Ratatui or async dependencies without a documented unavoidable need.
- CLI uses a **single-threaded** Crossterm event loop; poll/read terminal events on the same thread and never call blocking `read_line` after countdown starts. Capture a fresh `Instant::now()` and call engine `poll` **before applying any key action**. Handle only key Press (not Repeat/Release).
- Implement a partial-initialization-safe RAII terminal guard; restore raw mode, cursor and alternate screen on ordinary exit and recoverable errors. Never call `std::process::exit` while the guard is live.
- Build testable CLI parsing, an application controller, and a pure view formatting function. All countdown arithmetic stays in engine.
- Use stable Rust and match the repository's Rust 2021 conventions. Preserve the existing Cargo workspace.

## 4. Execution method — mandatory phase gates

Follow `docs/timer/IMPLEMENTATION_PLAN.md` **in order**, P0 through P9. At each phase:

1. Review applicable invariants and test IDs from `docs/timer/TESTING.md`.
2. Write the smallest correct implementation with explicit private state and documented public API.
3. Add positive, negative, precise boundary and invalid-transition tests before declaring the stage finished.
4. Compile, format, run focused tests, and resolve failures **before proceeding**.
5. Record only **actual** commands and results. Never claim unrun tests, Windows validation or manual terminal checks as passing.
6. Keep architecture and functionality unchanged unless an actual incompatibility forces a minimal correction; document any divergence and its reason in the final report.

Use the suggested file layout in the plan: root workspace with new sibling `countdown-timer` and `countdown-timer-cli`, plus `docs/timer/*` and this prompt file. Prefer exact documented paths unless an existing local conflict makes them impossible. Make the CLI usable via:

```bash
cargo run --locked -p countdown-timer-cli -- --seconds 180
```

and its documented `--help`. Implement an interactive duration prompt only **before** enabling raw terminal mode when no `--seconds` is supplied and stdin/stdout are actual terminals. In non-TTY environments exit with the planned diagnostic/code, not a hang or alternate-screen escapes.

## 5. Testing — no shortcuts

You must implement and run the required scenarios from the testing document, including:

- Zero/positive/large durations, exact nanosecond before/at deadline and large skipped updates.
- Repeated pause/resume with arbitrarily long pause, ensuring no drift.
- Legal/illegal transitions, preservation of state on errors, and reset from all states.
- **Critically:** snapshot after deadline before poll; late pause before poll; event emitted once on subsequent poll; reset before poll cancels pending event.
- Rounding, hours display, zero display and near-integer formatting cases.
- Seeded deterministic temporal reference tests; no real `thread::sleep` for exact time arithmetic.
- CLI invalid/valid flags, noninteractive `--help`, non-TTY error behavior and exit codes.
- Controller/view tests without real terminal input, including small window dimensions, ignored held-key repeats, redraw logic, and actions at expiration.
- Live interactive checks if a real terminal is available; otherwise mark **NOT TESTED** with reason.
- Full workspace regression to ensure original games and dictionary functionality still pass.

At final gate run the exact `cargo fmt`, `cargo check`, strict Clippy, debug/release workspace tests, release build, rustdoc and `git diff --check` commands in the plan, using the generated lockfile. Investigate failures honestly; don't hide or omit them. Record tested OS and toolchain; do not invent timing/CPU benchmark numbers.

## 6. Git workflow — mandatory

Follow conventional Git practices and maintain a clear, reviewable history:

- Run `git status --short --branch` and inspect current changes **before editing**. Do not destroy, stage or revert unrelated user modifications.
- Work on `feat/countdown-timer` if safe and not already in use; do not force-switch branches over uncommitted work. Preserve repository history and CI conventions.
- Commit small, coherent units as milestones pass, not one giant unreviewed final commit. Review `git diff` and staged files for every commit; stage only relevant paths rather than indiscriminately using `git add -A`.
- Use **Conventional Commit** headers: `type(scope): imperative short summary`, typically under 72 characters, **no period**.
- Every commit message has a **small header, one blank line, and a bulleted body** of 1–3 concise lines describing the change and (where appropriate) its tests/rationale. Example:

```text
feat(timer): implement monotonic countdown state machine

- Keep remaining time as a Duration budget measured against Instant.
- Model start, pause, resume and finished as explicit states.
- Verify deadline and expiration-event boundaries deterministically.
```

- Useful types: `feat`, `fix`, `test`, `docs`, `refactor`, `chore`; scopes such as `timer`, `timer-cli`, `workspace`.
- Keep commits focused, tests green, and code buildable at useful checkpoints. Never commit compiler artifacts (`target/`), generated logs, user secrets, or accidental benchmark outputs.
- Do not rewrite/force-push existing history, merge back into main, or push to remote **without express user authorization**. Local commits are permitted by this request.
- On a blocker, preserve modifications and explain the exact issue; do not fabricate completed checkpoints or fake commits.

## 7. Documentation, handoff and stop condition

- Add concise standalone timer usage/controls to root `README.md`; don't rewrite existing game documentation.
- Populate the verification table in `docs/timer/TESTING.md` using actual data, keeping **NOT RUN** for unexecuted checks. Preserve historical test evidence in the existing root `docs/TESTING.md`.
- Document precise API behavior, lifecycle invariants, the lazy-expiration/event split, Ctrl+C/raw mode terminal recovery, and platform-dependent suspend semantics.
- Confirm root CI's existing Ubuntu/Windows `--workspace` checks include both new members; only modify CI if materially needed.
- Review `git diff --name-status` and verify no original game/trie code was modified.

At the end, provide a factual report containing the branch, commit short hashes, changed files, API highlights, test commands and observed outcomes, numbers of tests actually run, live TTY platforms tested, any design deviations, and outstanding limitations or untested cases. **Stop there. Do not start game integration or optional extensions.**
