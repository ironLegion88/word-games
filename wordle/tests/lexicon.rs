use std::path::Path;
use trie::Trie;
use wordle::{Word, WordleError, WordleLexicon};

#[test]
fn test_lexicon_from_trie_and_curated_bundled() {
    let dict_paths = [
        Path::new("assets/dictionary.txt"),
        Path::new("word-games/assets/dictionary.txt"),
        Path::new("../assets/dictionary.txt"),
    ];
    let dict_path = dict_paths
        .iter()
        .find(|p| p.exists())
        .expect("dictionary.txt must exist");

    let trie = Trie::from_file(dict_path).unwrap();
    let lexicon =
        WordleLexicon::from_trie(&trie, wordle::lexicon::BUNDLED_CURATED_ANSWERS_SRC).unwrap();

    // Invariants
    assert_eq!(lexicon.allowed_guesses().len(), 8_938);
    assert_eq!(lexicon.curated_answers().len(), 489);

    // Sorted and unique
    let allowed = lexicon.allowed_guesses();
    for i in 1..allowed.len() {
        assert!(allowed[i - 1] < allowed[i]);
    }

    // Subset invariant: every answer is in allowed
    for ans in lexicon.curated_answers() {
        assert!(
            lexicon.is_allowed(ans),
            "Answer {ans} must be in allowed guesses"
        );
    }

    assert!(lexicon.is_allowed(&Word::parse("CRANE").unwrap()));
    assert!(lexicon.is_answer(&Word::parse("CRANE").unwrap()));
}

#[test]
fn test_answers_override_deduplication_and_counts() {
    let allowed = vec![
        Word::parse("APPLE").unwrap(),
        Word::parse("CRANE").unwrap(),
        Word::parse("SLATE").unwrap(),
        Word::parse("WATER").unwrap(),
    ];
    let curated = vec![Word::parse("CRANE").unwrap()];
    let lex = WordleLexicon::new_validated(allowed, curated).unwrap();

    let override_text = "
        apple
        crane
        APPLE
        water
        SLATE
        crane
    ";

    let (new_lex, report) = lex.with_answers_override(override_text).unwrap();
    assert_eq!(report.total_lines, 6);
    assert_eq!(report.unique_answers, 4);
    assert_eq!(report.duplicates_filtered, 2);
    assert_eq!(new_lex.curated_answers().len(), 4);
}

#[test]
fn test_answers_override_all_or_nothing_atomicity() {
    let allowed = vec![
        Word::parse("APPLE").unwrap(),
        Word::parse("CRANE").unwrap(),
        Word::parse("WATER").unwrap(),
    ];
    let curated = vec![Word::parse("CRANE").unwrap()];
    let lex = WordleLexicon::new_validated(allowed, curated).unwrap();

    // Line 2 is not in allowed dictionary ("SLATE" not present in this tiny allowed list)
    let override_text = "APPLE\nSLATE\nWATER\n";
    let res = lex.with_answers_override(override_text);
    assert!(matches!(
        res,
        Err(WordleError::InvalidAnswerOverride { line_number: 2, .. })
    ));

    // Existing lexicon remains untouched
    assert_eq!(lex.curated_answers().len(), 1);
    assert!(lex.is_answer(&Word::parse("CRANE").unwrap()));
}

#[test]
fn test_tiny_answers_override_valid() {
    let allowed = vec![Word::parse("APPLE").unwrap(), Word::parse("CRANE").unwrap()];
    let curated = vec![Word::parse("CRANE").unwrap()];
    let lex = WordleLexicon::new_validated(allowed, curated).unwrap();

    // 1-word override is completely valid
    let (new_lex, report) = lex.with_answers_override("APPLE\n").unwrap();
    assert_eq!(report.unique_answers, 1);
    assert_eq!(report.duplicates_filtered, 0);
    assert_eq!(new_lex.curated_answers().len(), 1);
    assert!(new_lex.is_answer(&Word::parse("APPLE").unwrap()));
}

#[test]
fn test_empty_override_rejected() {
    let allowed = vec![Word::parse("CRANE").unwrap()];
    let curated = vec![Word::parse("CRANE").unwrap()];
    let lex = WordleLexicon::new_validated(allowed, curated).unwrap();

    assert_eq!(
        lex.with_answers_override(""),
        Err(WordleError::EmptyAnswers)
    );
    assert_eq!(
        lex.with_answers_override("   \n\t  \n"),
        Err(WordleError::EmptyAnswers)
    );
}
