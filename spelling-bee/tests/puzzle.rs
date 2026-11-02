use spelling_bee::{GuessError, PuzzleError, SpellingBeePuzzle};

#[test]
fn test_puzzle_valid_construction() {
    let puzzle = SpellingBeePuzzle::new('C', &['A', 'I', 'M', 'O', 'R', 'T']).unwrap();
    assert_eq!(puzzle.center().to_char(), 'C');
    assert_eq!(puzzle.display_letters(), "C A I M O R T");

    let letters = puzzle.letters();
    assert_eq!(letters[0].to_char(), 'C');
    let outer_chars: Vec<char> = letters[1..].iter().map(|l| l.to_char()).collect();
    assert_eq!(outer_chars, vec!['A', 'I', 'M', 'O', 'R', 'T']);
}

#[test]
fn test_puzzle_case_insensitivity() {
    let puzzle = SpellingBeePuzzle::new('c', &['a', 'i', 'm', 'o', 'r', 't']).unwrap();
    assert_eq!(puzzle.center().to_char(), 'C');
    assert_eq!(puzzle.display_letters(), "C A I M O R T");
}

#[test]
fn test_puzzle_from_raw() {
    let puzzle = SpellingBeePuzzle::from_raw("CAIMORT").unwrap();
    assert_eq!(puzzle.center().to_char(), 'C');
    assert_eq!(puzzle.display_letters(), "C A I M O R T");

    let err = SpellingBeePuzzle::from_raw("CAIMOR").unwrap_err();
    assert!(matches!(
        err,
        PuzzleError::InvalidDistinctCount {
            expected: 7,
            actual: 6,
            ..
        }
    ));

    let err_empty = SpellingBeePuzzle::from_raw("").unwrap_err();
    assert_eq!(err_empty, PuzzleError::EmptyInput);
}

#[test]
fn test_puzzle_from_letters_and_center() {
    let puzzle = SpellingBeePuzzle::from_letters_and_center("AIMORTC", 'C').unwrap();
    assert_eq!(puzzle.center().to_char(), 'C');
    assert_eq!(puzzle.display_letters(), "C A I M O R T");

    let err = SpellingBeePuzzle::from_letters_and_center("AIMORTX", 'C').unwrap_err();
    assert!(matches!(
        err,
        PuzzleError::CenterNotInLetters { center: 'C', .. }
    ));
}

#[test]
fn test_puzzle_rejects_duplicate_letters() {
    let err = SpellingBeePuzzle::new('C', &['A', 'A', 'M', 'O', 'R', 'T']).unwrap_err();
    assert!(matches!(err, PuzzleError::InvalidDistinctCount { .. }));

    let err_center_dup = SpellingBeePuzzle::new('C', &['C', 'A', 'M', 'O', 'R', 'T']).unwrap_err();
    assert!(matches!(
        err_center_dup,
        PuzzleError::InvalidDistinctCount { .. }
    ));
}

#[test]
fn test_puzzle_rejects_non_ascii() {
    let err = SpellingBeePuzzle::new('1', &['A', 'I', 'M', 'O', 'R', 'T']).unwrap_err();
    assert!(matches!(err, PuzzleError::InvalidCharacter { ch: '1' }));

    let err_unicode = SpellingBeePuzzle::new('C', &['A', 'I', 'M', 'O', 'R', 'é']).unwrap_err();
    assert!(matches!(err_unicode, PuzzleError::InvalidCharacter { .. }));
}

#[test]
fn test_pangram_detection() {
    let puzzle = SpellingBeePuzzle::from_raw("CAIMORT").unwrap();

    // Word uses all 7 letters
    assert!(puzzle.is_pangram("AROMATIC"));
    assert!(puzzle.is_pangram("aromatic"));
    // Word with repeated letters using all 7
    assert!(puzzle.is_pangram("AAROMATICC"));

    // Missing 'R'
    assert!(!puzzle.is_pangram("ATOMIC"));
    // Missing 'C'
    assert!(!puzzle.is_pangram("MAJOR"));
}

#[test]
fn test_candidate_validation() {
    let puzzle = SpellingBeePuzzle::from_raw("CAIMORT").unwrap();

    // Valid candidates
    assert!(puzzle.validate_candidate("COMIC").is_ok());
    assert!(puzzle.validate_candidate("comic").is_ok());
    assert!(puzzle.validate_candidate("AROMATIC").is_ok());

    // Too short (< 4)
    let err_short = puzzle.validate_candidate("CAT").unwrap_err();
    assert_eq!(
        err_short,
        GuessError::TooShort {
            min_length: 4,
            actual_length: 3
        }
    );

    // Missing center ('C')
    let err_missing = puzzle.validate_candidate("ATOM").unwrap_err();
    assert_eq!(err_missing, GuessError::MissingCenter { center: 'C' });

    // Letter not in puzzle
    let err_invalid = puzzle.validate_candidate("DOG").unwrap_err();
    assert!(matches!(err_invalid, GuessError::InvalidLetter { .. }));

    // Non-ascii character
    let err_malformed = puzzle.validate_candidate("CAT!").unwrap_err();
    assert!(matches!(
        err_malformed,
        GuessError::MalformedInput { ch: '!' }
    ));
}
