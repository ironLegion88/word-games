//! Tests for Boggle session management, gameplay, feedback, and scoring invariants.

use boggle::board::Board;
use boggle::session::{BoggleSession, GuessOutcome};
use boggle::solver::SolverStrategy;
use trie::Trie;

fn make_fixture_session() -> (BoggleSession, Trie) {
    let mut trie = Trie::new();
    trie.insert("CATS").unwrap();
    trie.insert("DOGS").unwrap();
    trie.insert("BIRD").unwrap();
    trie.insert("COED").unwrap();
    trie.insert("ZEBRA").unwrap(); // not on board

    let input = "
        C A T S
        D O G S
        Qu R E W
        B I R D
    ";
    let board = Board::from_str_tiles(4, input).unwrap();
    let session = BoggleSession::from_trie(board, &trie, SolverStrategy::Scan).unwrap();
    (session, trie)
}

#[test]
fn test_session_basic_guesses_and_scoring() {
    let (mut session, _) = make_fixture_session();
    assert_eq!(session.cached_score(), 0);
    assert_eq!(session.solutions().len(), 4); // CATS, DOGS, BIRD, COED

    // Guess "CATS" (length 4 -> 1 pt)
    let res1 = session.submit_guess("cats");
    assert!(matches!(
        res1,
        GuessOutcome::Accepted {
            ref word,
            points: 1,
            total_score: 1,
            is_complete: false
        } if word == "CATS"
    ));
    assert_eq!(session.cached_score(), 1);

    // Duplicate guess "CATS"
    let res_dup = session.submit_guess("CATS");
    assert!(matches!(res_dup, GuessOutcome::Duplicate { ref word } if word == "CATS"));
    assert_eq!(session.cached_score(), 1); // no points added

    // Too short (< 3)
    let res_short = session.submit_guess("CA");
    assert!(matches!(res_short, GuessOutcome::TooShort { .. }));
    assert_eq!(session.cached_score(), 1);

    // Invalid characters
    let res_inv = session.submit_guess("CAT!");
    assert!(matches!(res_inv, GuessOutcome::InvalidWord { .. }));
    assert_eq!(session.cached_score(), 1);

    // Honest NotASolution for dictionary word not on board
    let res_not = session.submit_guess("ZEBRA");
    assert!(matches!(res_not, GuessOutcome::NotASolution { ref word } if word == "ZEBRA"));
    assert_eq!(session.cached_score(), 1);

    // Another valid guess "COED" (length 4 -> 1 pt)
    let res2 = session.submit_guess("coed");
    assert!(matches!(
        res2,
        GuessOutcome::Accepted {
            ref word,
            points: 1,
            total_score: 2,
            is_complete: false
        } if word == "COED"
    ));
    assert_eq!(session.cached_score(), 2);
}

#[test]
fn test_session_invariants_hold_after_mutations() {
    let (mut session, _) = make_fixture_session();

    session.submit_guess("CATS");
    session.submit_guess("CATS"); // duplicate
    session.submit_guess("FOO"); // not solution
    session.submit_guess("DOGS");
    session.submit_guess("BIRD");

    session.assert_invariants();
    assert_eq!(session.cached_score(), 3);
    assert_eq!(session.recalculate_score(), 3);
}

#[test]
fn test_dictionary_mutation_does_not_affect_active_session() {
    let (mut session, mut trie) = make_fixture_session();

    // Verify initial solutions count is 4
    assert_eq!(session.solutions().len(), 4);

    // Mutate the trie: remove "CATS", add new words
    trie.remove("CATS").unwrap();
    trie.insert("EXTRA").unwrap();

    // The active session still has CATS as a valid solution!
    let res = session.submit_guess("CATS");
    assert!(matches!(res, GuessOutcome::Accepted { .. }));
    assert_eq!(session.cached_score(), 1);
}

#[test]
fn test_quit_summary_and_terminal_immutability() {
    let (mut session, _) = make_fixture_session();

    session.submit_guess("CATS");
    let summary = session.quit();

    assert_eq!(summary.score, 1);
    assert_eq!(summary.found_count, 1);
    assert_eq!(summary.total_solutions, 4);
    assert_eq!(summary.found_words, vec!["CATS"]);
    assert_eq!(summary.missed_words.len(), 3); // BIRD, COED, DOGS

    assert!(session.is_finished());

    // Subsequent submissions must be rejected with GameFinished and cause NO state changes
    let post_res = session.submit_guess("DOGS");
    assert_eq!(post_res, GuessOutcome::GameFinished);
    assert_eq!(session.cached_score(), 1);
    assert_eq!(session.accepted_guesses().len(), 1);
}

#[test]
fn test_all_solutions_found_completes_session() {
    let (mut session, _) = make_fixture_session();

    session.submit_guess("BIRD");
    session.submit_guess("CATS");
    session.submit_guess("COED");
    let last = session.submit_guess("DOGS");

    assert!(matches!(
        last,
        GuessOutcome::Accepted {
            is_complete: true,
            total_score: 4,
            ..
        }
    ));

    assert!(session.is_finished());

    // Post-completion submission rejected
    let post = session.submit_guess("CATS");
    assert_eq!(post, GuessOutcome::GameFinished);
}
