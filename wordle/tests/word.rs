use trie::letter::Letter;
use wordle::{Word, WordleError, WORD_LENGTH};

#[test]
fn test_word_parse_valid_lowercase_uppercase_mixed() {
    let w1 = Word::parse("crane").unwrap();
    let w2 = Word::parse("CRANE").unwrap();
    let w3 = Word::parse("CrAnE").unwrap();
    assert_eq!(w1, w2);
    assert_eq!(w2, w3);
    assert_eq!(w1.as_string(), "CRANE");
    assert_eq!(w1.to_bytes(), *b"CRANE");
}

#[test]
fn test_word_parse_whitespace_trimmed() {
    let w = Word::parse("  slate  \n").unwrap();
    assert_eq!(w.as_string(), "SLATE");
}

#[test]
fn test_word_parse_empty_rejected() {
    assert_eq!(Word::parse(""), Err(WordleError::EmptyInput));
    assert_eq!(Word::parse("    "), Err(WordleError::EmptyInput));
    assert_eq!(Word::parse("\t\n"), Err(WordleError::EmptyInput));
}

#[test]
fn test_word_parse_invalid_length() {
    assert_eq!(
        Word::parse("A"),
        Err(WordleError::InvalidLength {
            expected: WORD_LENGTH,
            actual: 1
        })
    );
    assert_eq!(
        Word::parse("FOUR"),
        Err(WordleError::InvalidLength {
            expected: WORD_LENGTH,
            actual: 4
        })
    );
    assert_eq!(
        Word::parse("SIXERS"),
        Err(WordleError::InvalidLength {
            expected: WORD_LENGTH,
            actual: 6
        })
    );
}

#[test]
fn test_word_parse_invalid_characters() {
    assert_eq!(
        Word::parse("CR4NE"),
        Err(WordleError::InvalidCharacter {
            character: '4',
            position: 2
        })
    );
    assert_eq!(
        Word::parse("CR!NE"),
        Err(WordleError::InvalidCharacter {
            character: '!',
            position: 2
        })
    );
    assert_eq!(
        Word::parse("CR NE"),
        Err(WordleError::InvalidCharacter {
            character: ' ',
            position: 2
        })
    );
    assert_eq!(
        Word::parse("CR-NE"),
        Err(WordleError::InvalidCharacter {
            character: '-',
            position: 2
        })
    );
}

#[test]
fn test_word_indexing_and_from_letters() {
    let letters = [
        Letter::from_char('P').unwrap(),
        Letter::from_char('L').unwrap(),
        Letter::from_char('A').unwrap(),
        Letter::from_char('N').unwrap(),
        Letter::from_char('T').unwrap(),
    ];
    let w = Word::from_letters(letters);
    assert_eq!(w[0], Letter::from_char('P').unwrap());
    assert_eq!(w[1], Letter::from_char('L').unwrap());
    assert_eq!(w[2], Letter::from_char('A').unwrap());
    assert_eq!(w[3], Letter::from_char('N').unwrap());
    assert_eq!(w[4], Letter::from_char('T').unwrap());
    assert_eq!(w.as_string(), "PLANT");
}
