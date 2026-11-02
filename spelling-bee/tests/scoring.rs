use spelling_bee::calculate_word_score;

#[test]
fn test_scoring_rules() {
    // Under 4 letters: 0
    assert_eq!(calculate_word_score("CAT", false), 0);
    assert_eq!(calculate_word_score("A", false), 0);
    assert_eq!(calculate_word_score("", false), 0);

    // 4-letter word: 1 point
    assert_eq!(calculate_word_score("ACAI", false), 1);
    assert_eq!(calculate_word_score("COMA", false), 1);

    // 5-letter word: 5 points
    assert_eq!(calculate_word_score("COMIC", false), 5);

    // 6-letter word: 6 points
    assert_eq!(calculate_word_score("CITRIC", false), 6);

    // 7-letter word: 7 points without pangram, 14 with pangram
    assert_eq!(calculate_word_score("TACTICS", false), 7);
    assert_eq!(calculate_word_score("AROMTIC", true), 14);

    // 8-letter pangram: 8 + 7 = 15 points
    assert_eq!(calculate_word_score("AROMATIC", true), 15);

    // 9-letter pangram: 9 + 7 = 16 points
    assert_eq!(calculate_word_score("ATTRACTOR", true), 16);
}
