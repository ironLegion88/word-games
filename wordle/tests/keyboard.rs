use trie::letter::Letter;
use wordle::feedback::LetterFeedback;
use wordle::keyboard::{KeyStatus, Keyboard};

#[test]
fn test_keyboard_monotonic_precedence() {
    let mut kb = Keyboard::new();
    let letter_a = Letter::from_char('A').unwrap();

    // 1. Initial state is Unknown
    assert_eq!(kb.status(letter_a), KeyStatus::Unknown);

    // 2. Unknown -> Absent
    kb.observe(letter_a, LetterFeedback::Absent);
    assert_eq!(kb.status(letter_a), KeyStatus::Absent);

    // 3. Absent -> Present
    kb.observe(letter_a, LetterFeedback::Present);
    assert_eq!(kb.status(letter_a), KeyStatus::Present);

    // 4. Present -> Correct
    kb.observe(letter_a, LetterFeedback::Correct);
    assert_eq!(kb.status(letter_a), KeyStatus::Correct);

    // 5. Subsequent Absent MUST NOT downgrade Correct
    kb.observe(letter_a, LetterFeedback::Absent);
    assert_eq!(kb.status(letter_a), KeyStatus::Correct);

    // 6. Subsequent Present MUST NOT downgrade Correct
    kb.observe(letter_a, LetterFeedback::Present);
    assert_eq!(kb.status(letter_a), KeyStatus::Correct);
}

#[test]
fn test_keyboard_duplicate_letter_trap() {
    // Guess has two Es, target has one E.
    // First E gets Present, second E gets Absent.
    // Keyboard for E must remain Present!
    let mut kb = Keyboard::new();
    let letter_e = Letter::from_char('E').unwrap();

    kb.observe(letter_e, LetterFeedback::Present);
    assert_eq!(kb.status(letter_e), KeyStatus::Present);

    kb.observe(letter_e, LetterFeedback::Absent);
    assert_eq!(
        kb.status(letter_e),
        KeyStatus::Present,
        "Absent duplicate must not downgrade Present to Absent"
    );
}

#[test]
fn test_keyboard_format_plain() {
    let mut kb = Keyboard::new();
    kb.observe(Letter::from_char('Q').unwrap(), LetterFeedback::Correct);
    kb.observe(Letter::from_char('W').unwrap(), LetterFeedback::Present);
    kb.observe(Letter::from_char('E').unwrap(), LetterFeedback::Absent);

    let rows = kb.format_plain();
    assert_eq!(rows.len(), 3);
    assert!(rows[0].starts_with("Q! W? E."));
}
