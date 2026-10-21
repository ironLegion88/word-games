use std::io::Cursor;
use trie::{LoadError, Trie};

#[test]
fn format_one_word_per_line() {
    let input = "DOG\nCAT\nBIRD";
    let trie = Trie::from_reader(Cursor::new(input)).expect("valid load");
    assert_eq!(trie.len(), 3);
    assert!(trie.contains("DOG"));
    assert!(trie.contains("CAT"));
    assert!(trie.contains("BIRD"));
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn format_comma_only() {
    let input = "DOG,CAT,BIRD";
    let trie = Trie::from_reader(Cursor::new(input)).expect("valid load");
    assert_eq!(trie.len(), 3);
    assert!(trie.contains("DOG"));
    assert!(trie.contains("CAT"));
    assert!(trie.contains("BIRD"));
}

#[test]
fn format_mixed_delimiters_and_whitespace() {
    let input = "DOG, CAT\tBIRD\nFOX,   BEAR\r\n";
    let trie = Trie::from_reader(Cursor::new(input)).expect("valid load");
    assert_eq!(trie.len(), 5);
    assert!(trie.contains("DOG"));
    assert!(trie.contains("CAT"));
    assert!(trie.contains("BIRD"));
    assert!(trie.contains("FOX"));
    assert!(trie.contains("BEAR"));
}

#[test]
fn format_empty_and_repeated_delimiters() {
    let input = "\n\t  DOG,, ,CAT,\n\n\r\n  BIRD,  \n";
    let trie = Trie::from_reader(Cursor::new(input)).expect("valid load");
    assert_eq!(trie.len(), 3);
    assert!(trie.contains("DOG"));
    assert!(trie.contains("CAT"));
    assert!(trie.contains("BIRD"));
}

#[test]
fn format_eof_without_final_newline() {
    let input = "FIRST\nSECOND\nLAST";
    let trie = Trie::from_reader(Cursor::new(input)).expect("valid load");
    assert_eq!(trie.len(), 3);
    assert!(trie.contains("LAST"));
}

#[test]
fn format_utf8_bom_tolerated() {
    let input = "\u{FEFF}APPLE,BANANA\nCHERRY";
    let trie = Trie::from_reader(Cursor::new(input)).expect("valid load with BOM");
    assert_eq!(trie.len(), 3);
    assert!(trie.contains("APPLE"));
    assert!(trie.contains("BANANA"));
    assert!(trie.contains("CHERRY"));
}

#[test]
fn duplicate_tokens_in_batch() {
    let input = "DOG,CAT,DOG,BIRD,CAT,DOG";
    let mut trie = Trie::new();
    let report = trie
        .extend_from_reader(Cursor::new(input))
        .expect("valid load");
    assert_eq!(report.tokens_seen, 6);
    assert_eq!(report.inserted, 3);
    assert_eq!(report.already_present, 3);
    assert_eq!(trie.len(), 3);
}

#[test]
fn empty_file_produces_empty_trie() {
    let input = "";
    let trie = Trie::from_reader(Cursor::new(input)).expect("valid load");
    assert_eq!(trie.len(), 0);
    assert!(trie.is_empty());
    assert!(trie.check_invariants().is_ok());

    let delimiters_only = "  , \t \n\r\n ,,, \t ";
    let trie2 = Trie::from_reader(Cursor::new(delimiters_only)).expect("valid load");
    assert_eq!(trie2.len(), 0);
    assert!(trie2.check_invariants().is_ok());
}

#[test]
fn invalid_characters_rejected_with_location() {
    let cases = [
        ("DOG;CAT", ';', 1, 4),
        ("DOG-HEAD", '-', 1, 4),
        ("DOG2", '2', 1, 4),
        ("DOG\nCAT\nB@T", '@', 3, 2),
        ("DOG\nCAT\ncafé", 'é', 3, 4),
    ];

    for (input, expected_char, expected_line, expected_col) in cases {
        let err = Trie::from_reader(Cursor::new(input)).expect_err("should reject invalid token");
        match err {
            LoadError::InvalidWord {
                source,
                line,
                column,
                ..
            } => {
                assert_eq!(line, expected_line);
                assert_eq!(column, expected_col);
                match source {
                    trie::WordError::InvalidCharacter { character, .. } => {
                        assert_eq!(character, expected_char);
                    }
                    _ => panic!("unexpected inner error: {source:?}"),
                }
            }
            _ => panic!("unexpected outer error: {err:?}"),
        }
    }
}

#[test]
fn failure_atomicity_on_invalid_input() {
    let mut trie = Trie::new();
    trie.insert("EXISTING").unwrap();
    let stats_before = trie.stats();

    // Extend with input that fails on the 3rd word
    let bad_input = "ALPHA, BETA, BAD;WORD, DELTA";
    let res = trie.extend_from_reader(Cursor::new(bad_input));
    assert!(res.is_err());

    // Trie must be COMPLETELY unmodified
    assert_eq!(trie.stats(), stats_before);
    assert_eq!(trie.len(), 1);
    assert!(trie.contains("EXISTING"));
    assert!(!trie.contains("ALPHA"));
    assert!(!trie.contains("BETA"));
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn failure_atomicity_on_batch_delete() {
    let mut trie = Trie::new();
    trie.insert("ALPHA").unwrap();
    trie.insert("BETA").unwrap();
    let stats_before = trie.stats();

    // Batch delete with an invalid word
    let res = trie.remove_words(["ALPHA", "INVALID!WORD", "BETA"]);
    assert!(res.is_err());

    // Both ALPHA and BETA must still be in the trie
    assert_eq!(trie.stats(), stats_before);
    assert!(trie.contains("ALPHA"));
    assert!(trie.contains("BETA"));
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn batch_remove_words_report() {
    let mut trie = Trie::new();
    trie.insert("ALPHA").unwrap();
    trie.insert("BETA").unwrap();
    trie.insert("GAMMA").unwrap();

    let report = trie
        .remove_words(["ALPHA", "ALPHA", "DELTA", "BETA"])
        .expect("valid words");
    assert_eq!(report.requested, 4);
    assert_eq!(report.removed, 2); // ALPHA, BETA
    assert_eq!(report.not_found, 2); // Duplicate ALPHA, DELTA

    assert_eq!(trie.len(), 1);
    assert!(trie.contains("GAMMA"));
    assert!(!trie.contains("ALPHA"));
    assert!(!trie.contains("BETA"));
    assert!(trie.check_invariants().is_ok());
}

#[test]
fn filtering_present_and_absent_preserves_order_and_duplicates() {
    let mut trie = Trie::new();
    trie.insert("CAT").unwrap();
    trie.insert("DOG").unwrap();
    trie.insert("CAR").unwrap();

    let input = "cat, BIRD, dog, CAT, FOX, CA, DOG, FISH";

    let present = trie
        .filter_present_from_reader(Cursor::new(input))
        .expect("valid parse");
    // CA is a prefix, but not a full word -> should not be present!
    assert_eq!(present, vec!["CAT", "DOG", "CAT", "DOG"]);

    let absent = trie
        .filter_absent_from_reader(Cursor::new(input))
        .expect("valid parse");
    assert_eq!(absent, vec!["BIRD", "FOX", "CA", "FISH"]);
}

#[test]
fn equivalence_of_file_loading_and_individual_insertion() {
    let input = "APPLE,BANANA,CHERRY,DATE,ELDERBERRY,FIG,GRAPE";

    let trie1 = Trie::from_reader(Cursor::new(input)).unwrap();

    let mut trie2 = Trie::new();
    for word in input.split(',') {
        trie2.insert(word).unwrap();
    }

    assert_eq!(trie1.len(), trie2.len());
    assert_eq!(trie1.stats().live_nodes, trie2.stats().live_nodes);

    let mut words1 = Vec::new();
    trie1.visit_words(|w| words1.push(w.to_string()));

    let mut words2 = Vec::new();
    trie2.visit_words(|w| words2.push(w.to_string()));

    assert_eq!(words1, words2);
}
