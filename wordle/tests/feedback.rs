//! Comprehensive tests for Wordle feedback algorithms (WL-F1 and WL-F2).
//!
//! Includes:
//! 1. Table-driven known duplicate traps.
//! 2. Independent test oracle using unmatched target multiset occurrence removal.
//! 3. Test-only Bulls/Cows aggregate cross-check.
//! 4. Exhaustive 59,049 pairs over alphabet {A, B, C} at length 5.
//! 5. Seeded full-alphabet randomized differential testing across 3 fixed seeds.
//! 6. Real dictionary/answer pair differential testing.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use trie::letter::Letter;

use wordle::feedback::{evaluate_frequency, evaluate_markers, Feedback, LetterFeedback};
use wordle::word::{Word, WORD_LENGTH};

/// Independent test oracle using an unmatched-target multiset with occurrence removal.
///
/// Designed independently from both production scorers (`evaluate_markers` and `evaluate_frequency`).
/// Does not call or wrap either production scorer.
fn independent_multiset_oracle(target: &Word, guess: &Word) -> Feedback {
    let mut feedback = [LetterFeedback::Absent; WORD_LENGTH];
    let mut unmatched_target = Vec::with_capacity(WORD_LENGTH);

    // Pass 1: Identify exact matches. Non-matching target letters are placed in the multiset.
    for i in 0..WORD_LENGTH {
        if target[i] == guess[i] {
            feedback[i] = LetterFeedback::Correct;
        } else {
            unmatched_target.push(target[i]);
        }
    }

    // Pass 2: Left-to-right pass over non-matching guess letters.
    // If guess letter is present in the unmatched multiset, consume one occurrence and mark Present.
    for i in 0..WORD_LENGTH {
        if feedback[i] != LetterFeedback::Correct {
            let guess_letter = guess[i];
            if let Some(pos) = unmatched_target.iter().position(|&l| l == guess_letter) {
                feedback[i] = LetterFeedback::Present;
                unmatched_target.remove(pos);
            }
        }
    }

    feedback
}

/// Attributed test-only aggregate cross-check adapted from Mastermind's `(bulls, cows)`.
///
/// Counts bulls (exact) and cows (misplaced) from frequency vectors.
/// Used solely as an aggregate check: `bulls == count(Correct)` and `cows == count(Present)`.
fn reference_mastermind_aggregate(target: &Word, guess: &Word) -> (usize, usize) {
    let mut bulls = 0;
    let mut target_freq = [0usize; 26];
    let mut guess_freq = [0usize; 26];

    for i in 0..WORD_LENGTH {
        if target[i] == guess[i] {
            bulls += 1;
        }
        target_freq[target[i].index()] += 1;
        guess_freq[guess[i].index()] += 1;
    }

    let mut total_matches = 0;
    for c in 0..26 {
        total_matches += target_freq[c].min(guess_freq[c]);
    }

    let cows = total_matches - bulls;
    (bulls, cows)
}

fn assert_feedback_equivalence(target: &Word, guess: &Word) {
    let f1 = evaluate_markers(target, guess);
    let f2 = evaluate_frequency(target, guess);
    let oracle = independent_multiset_oracle(target, guess);

    assert_eq!(
        f1, oracle,
        "WL-F1 marker scorer failed oracle on target={target}, guess={guess}"
    );
    assert_eq!(
        f2, oracle,
        "WL-F2 frequency scorer failed oracle on target={target}, guess={guess}"
    );

    // Aggregate corroboration
    let (exp_bulls, exp_cows) = reference_mastermind_aggregate(target, guess);
    let actual_bulls = f1.iter().filter(|&&m| m == LetterFeedback::Correct).count();
    let actual_cows = f1.iter().filter(|&&m| m == LetterFeedback::Present).count();

    assert_eq!(actual_bulls, exp_bulls);
    assert_eq!(actual_cows, exp_cows);
}

// ---------------------------------------------------------------------------
// 1. Table-driven tests for known edge cases and duplicate traps
// ---------------------------------------------------------------------------

#[test]
fn test_known_table_cases() {
    let cases = [
        // (target, guess, expected_tags)
        ("CRANE", "CRANE", "CCCCC"),
        ("CRANE", "TOILS", "....."),
        ("APPLE", "ALLEY", "CP.P."),
        ("LEVEL", "EERIE", "PC..."),
        ("WATER", "EERIE", "P.P.."),
        ("SLATE", "SALET", "CPPPP"),
        ("SPEED", "ERASE", "P..PP"),
        ("ROBOT", "BOOST", "PCP.C"),
        ("ABBEY", "BABES", "PPCC."),
        ("NANNY", "BANAL", ".CC.."),
    ];

    for (t_str, g_str, expected_tags) in cases {
        let target = Word::parse(t_str).unwrap();
        let guess = Word::parse(g_str).unwrap();

        let f1 = evaluate_markers(&target, &guess);
        let f2 = evaluate_frequency(&target, &guess);
        let oracle = independent_multiset_oracle(&target, &guess);

        let tags1: String = f1.iter().map(|m| m.tag()).collect();
        let tags2: String = f2.iter().map(|m| m.tag()).collect();
        let tags_oracle: String = oracle.iter().map(|m| m.tag()).collect();

        assert_eq!(
            tags1, expected_tags,
            "Mismatch on ({t_str}, {g_str}) in WL-F1"
        );
        assert_eq!(
            tags2, expected_tags,
            "Mismatch on ({t_str}, {g_str}) in WL-F2"
        );
        assert_eq!(
            tags_oracle, expected_tags,
            "Mismatch on ({t_str}, {g_str}) in Oracle"
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Exhaustive 59,049 pairs over alphabet {A, B, C} at length 5
// ---------------------------------------------------------------------------

#[test]
fn test_exhaustive_59049_pairs_over_abc() {
    // 3^5 = 243 possible 5-letter words over {A, B, C}
    // 243 * 243 = 59,049 pairs
    let chars = *b"ABC";
    let mut words = Vec::with_capacity(243);

    for &c0 in &chars {
        for &c1 in &chars {
            for &c2 in &chars {
                for &c3 in &chars {
                    for &c4 in &chars {
                        let letters = [
                            Letter::from_ascii_byte(c0).unwrap(),
                            Letter::from_ascii_byte(c1).unwrap(),
                            Letter::from_ascii_byte(c2).unwrap(),
                            Letter::from_ascii_byte(c3).unwrap(),
                            Letter::from_ascii_byte(c4).unwrap(),
                        ];
                        words.push(Word::from_letters(letters));
                    }
                }
            }
        }
    }

    assert_eq!(words.len(), 243);

    let mut count = 0;
    for target in &words {
        for guess in &words {
            assert_feedback_equivalence(target, guess);
            count += 1;
        }
    }

    assert_eq!(count, 59_049, "Must evaluate exactly 59,049 pairs");
}

// ---------------------------------------------------------------------------
// 3. Seeded randomized differential tests across 3 fixed seeds
// ---------------------------------------------------------------------------

#[test]
fn test_seeded_differential_random_streams() {
    let seeds = [0x5EED_2026_u64, 0xC0FFEE_u64, 0xBADC0DE_u64];
    const ITERATIONS_PER_SEED: usize = 2_000;

    for &seed in &seeds {
        let mut rng = StdRng::seed_from_u64(seed);

        for _ in 0..ITERATIONS_PER_SEED {
            let mut target_letters = [Letter::from_ascii_byte(b'A').unwrap(); WORD_LENGTH];
            let mut guess_letters = [Letter::from_ascii_byte(b'A').unwrap(); WORD_LENGTH];

            for i in 0..WORD_LENGTH {
                target_letters[i] = Letter::from_index(rng.gen_range(0..26)).unwrap();
                guess_letters[i] = Letter::from_index(rng.gen_range(0..26)).unwrap();
            }

            let target = Word::from_letters(target_letters);
            let guess = Word::from_letters(guess_letters);

            assert_feedback_equivalence(&target, &guess);
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Real curated answers differential tests
// ---------------------------------------------------------------------------

#[test]
fn test_curated_answers_cross_differential() {
    let words = [
        "APPLE", "ALLEY", "CRANE", "SLATE", "WATER", "LEVEL", "EERIE", "PLANT", "LIGHT", "SOUND",
        "ROBOT", "BOOST", "SPEED", "ERASE", "CIVIL", "VIVID", "QUEEN", "KNIFE",
    ];

    for &t_str in &words {
        for &g_str in &words {
            let target = Word::parse(t_str).unwrap();
            let guess = Word::parse(g_str).unwrap();
            assert_feedback_equivalence(&target, &guess);
        }
    }
}
