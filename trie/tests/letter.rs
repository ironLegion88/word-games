use std::borrow::Cow;
use trie::{normalize_word, Letter, WordError, ALPHABET_SIZE};

#[test]
fn letter_a_through_z() {
    assert_eq!(ALPHABET_SIZE, 26);
    for i in 0..26 {
        let upper_char = (b'A' + i as u8) as char;
        let lower_char = (b'a' + i as u8) as char;

        let letter_upper = Letter::from_char(upper_char).expect("valid uppercase ASCII");
        let letter_lower = Letter::from_char(lower_char).expect("valid lowercase ASCII");

        assert_eq!(letter_upper, letter_lower);
        assert_eq!(letter_upper.index(), i);
        assert_eq!(letter_upper.to_char(), upper_char);
        assert_eq!(letter_upper.to_ascii_byte(), b'A' + i as u8);

        let letter_from_idx = Letter::from_index(i).expect("valid index");
        assert_eq!(letter_from_idx, letter_upper);
    }

    assert!(Letter::from_index(26).is_none());
    assert!(Letter::from_index(100).is_none());
}

#[test]
fn letter_invalid_ascii() {
    let invalid_chars = [' ', '\t', '\n', '\r', '0', '9', '-', '\'', '.', ',', ';', '!', '@', '_'];
    for &c in &invalid_chars {
        assert!(Letter::from_char(c).is_none(), "expected None for '{c}'");
        assert!(Letter::from_ascii_byte(c as u8).is_none(), "expected None for byte b'{c}'");
    }
}

#[test]
fn letter_unicode() {
    let unicode_chars = ['é', 'ñ', 'ü', 'ß', 'α', 'Ω', '中', '🦀'];
    for &c in &unicode_chars {
        assert!(Letter::from_char(c).is_none(), "expected None for unicode '{c}'");
    }
}

#[test]
fn case_upper_identity() {
    let words = ["A", "CAT", "HELLO", "SUPERDISCOVERY", "ZZZ"];
    for &w in &words {
        let res = normalize_word(w).expect("valid uppercase");
        match res {
            Cow::Borrowed(b) => assert_eq!(b, w),
            Cow::Owned(_) => panic!("expected borrowed string for already uppercase '{w}'"),
        }
    }
}

#[test]
fn case_lower() {
    let words = [("cat", "CAT"), ("dog", "DOG"), ("zzyvas", "ZZYVAS")];
    for (lower, upper) in words {
        let res = normalize_word(lower).expect("valid lowercase");
        match res {
            Cow::Owned(s) => assert_eq!(s, upper),
            Cow::Borrowed(_) => panic!("expected owned string for lowercase '{lower}'"),
        }
    }
}

#[test]
fn case_mixed() {
    let words = [("cAt", "CAT"), ("DoG", "DOG"), ("zZyVaS", "ZZYVAS")];
    for (mixed, upper) in words {
        let res = normalize_word(mixed).expect("valid mixed case");
        assert_eq!(res.as_ref(), upper);
    }
}

#[test]
fn empty_rejected() {
    assert_eq!(normalize_word(""), Err(WordError::EmptyWord));
}

#[test]
fn table_driven_malformed_tokens_rejected() {
    let cases = [
        (" space", ' ', 0),
        ("word with space", ' ', 4),
        ("trailing ", ' ', 8),
        ("hyphen-word", '-', 6),
        ("can't", '\'', 3),
        ("word123", '1', 4),
        ("dog;", ';', 3),
        ("cat,dog", ',', 3),
        ("café", 'é', 3),
        ("naïve", 'ï', 2),
        ("über", 'ü', 0),
        ("tab\tword", '\t', 3),
        ("newline\nword", '\n', 7),
    ];

    for (input, expected_char, expected_pos) in cases {
        let err = normalize_word(input).expect_err(&format!("expected error for '{input}'"));
        match err {
            WordError::InvalidCharacter { character, position } => {
                assert_eq!(character, expected_char, "mismatched char for '{input}'");
                assert_eq!(position, expected_pos, "mismatched pos for '{input}'");
            }
            _ => panic!("unexpected error type for '{input}': {err:?}"),
        }
    }
}

#[test]
fn normalization_idempotent() {
    let words = ["Rust", "cOdE", "DICTIONARY", "programming"];
    for &w in &words {
        let first = normalize_word(w).expect("valid");
        let second = normalize_word(first.as_ref()).expect("valid");
        assert_eq!(first, second);
        match second {
            Cow::Borrowed(_) => {}
            Cow::Owned(_) => panic!("second normalization must borrow, not allocate!"),
        }
    }
}
