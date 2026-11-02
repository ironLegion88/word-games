//! Tests for independent per-word board DFS and on-demand validation (Milestone BG-S1).

use boggle::board::Board;
use boggle::path::{contains_word, find_word_path, validate_on_demand, OnDemandResult};
use trie::Trie;

fn make_test_board() -> Board {
    // 4x4 board:
    // C A T S
    // D O G S
    // Qu R E W
    // B I R D
    let input = "
        C A T S
        D O G S
        Qu R E W
        B I R D
    ";
    Board::from_str_tiles(4, input).unwrap()
}

#[test]
fn test_exact_horizontal_path() {
    let board = make_test_board();
    // "CATS" is row 0: cells 0, 1, 2, 3
    let path = find_word_path(&board, "CATS").expect("CATS exists");
    assert_eq!(path, vec![0, 1, 2, 3]);
    assert!(contains_word(&board, "CATS"));
}

#[test]
fn test_exact_vertical_path() {
    let board = make_test_board();
    // "C D Qu B" is col 0: cells 0, 4, 8, 12 -> spells "CDQUB"
    let path = find_word_path(&board, "CDQUB").expect("CDQUB exists");
    assert_eq!(path, vec![0, 4, 8, 12]);
}

#[test]
fn test_exact_diagonal_path() {
    let board = make_test_board();
    // Cells 0 (C), 5 (O), 10 (E), 15 (D) -> "COED"
    let path = find_word_path(&board, "COED").expect("COED exists diagonally");
    assert_eq!(path, vec![0, 5, 10, 15]);
    assert!(contains_word(&board, "COED"));
}

#[test]
fn test_reverse_movement() {
    let board = make_test_board();
    // "STAC" is reverse of "CATS": cells 3, 2, 1, 0
    let path = find_word_path(&board, "STAC").expect("STAC exists in reverse");
    assert_eq!(path, vec![3, 2, 1, 0]);
    assert!(contains_word(&board, "STAC"));
}

#[test]
fn test_illegal_jumps_rejected() {
    let board = make_test_board();
    // "CS" - C is at 0, S is at 3. 0 and 3 are NOT neighbors on 4x4!
    // Try word "CB" - C is at 0, B is at 12. Not neighbors!
    assert!(!contains_word(&board, "CB"));
    assert_eq!(find_word_path(&board, "CB"), None);
}

#[test]
fn test_cell_reuse_forbidden() {
    let board = make_test_board();
    // On the board, C (0) -> A (1) -> T (2).
    // Can we spell "CATAC"?
    // 0 -> 1 -> 2 -> 1 (reuse A) -> 0 (reuse C)?
    // Forbidden! Cell 1 and cell 0 cannot be reused in the same word.
    assert!(!contains_word(&board, "CATAC"));
    assert_eq!(find_word_path(&board, "CATAC"), None);
}

#[test]
fn test_repeated_letters_on_different_cells_allowed() {
    // Board with two distinct 'E' cells:
    // S E E D
    // A B C D
    // E F G H
    // I J K L
    let board = Board::from_str_tiles(
        4,
        "
        S E E D
        A B C D
        E F G H
        I J K L
    ",
    )
    .unwrap();

    // "SEED" uses S(0), E(1), E(2), D(3).
    // Cells 1 and 2 are different cells having the same letter 'E'. Valid!
    let path = find_word_path(&board, "SEED").expect("SEED exists on separate E cells");
    assert_eq!(path, vec![0, 1, 2, 3]);
    assert!(contains_word(&board, "SEED"));
}

#[test]
fn test_multiple_paths_for_same_word() {
    // Board with multiple ways to spell "DOG":
    // D O G A
    // O G D B
    // G D O C
    // A B C D
    let board = Board::from_str_tiles(
        4,
        "
        D O G A
        O G D B
        G D O C
        A B C D
    ",
    )
    .unwrap();

    // DOG can be spelled via (0, 1, 2), (0, 5, 2), (0, 4, 8), etc.
    assert!(contains_word(&board, "DOG"));
    let path = find_word_path(&board, "DOG").unwrap();
    assert_eq!(path.len(), 3);
}

#[test]
fn test_qu_tile_advances_two_letters_one_cell() {
    let board = make_test_board();
    // Row 2: cell 8 is Qu, cell 9 is R, cell 10 is E, cell 11 is W.
    // "QU" (cell 8) -> "E" (cell 5 or cell 10)
    // Let's check cell 8 (Qu) -> cell 9 (R) -> cell 5 (O) -> cell 1 (A) -> cell 2 (T) -> "QUROAT"
    // Also "QUI" -> cell 8 (Qu) -> cell 13 (I).
    // Cell 8 is Qu, cell 13 is I (they are neighbors: 8 is (2,0), 13 is (3,1)).
    // "QUI" has length 3, and uses 2 cells (8 and 13).
    let path = find_word_path(&board, "QUI").expect("QUI path exists");
    assert_eq!(path, vec![8, 13]);
    assert!(contains_word(&board, "QUI"));

    // Bare Q without U cannot be matched by Qu tile
    assert!(!contains_word(&board, "Q"));
    assert!(!contains_word(&board, "QA"));
}

#[test]
fn test_bit_35_path_on_6x6_board() {
    let mut tiles = vec![boggle::board::Tile::from_char('A').unwrap(); 36];
    // Put a specific word ending at cell 35:
    // cell 28: 'W', cell 34: 'I', cell 35: 'N'
    tiles[28] = boggle::board::Tile::from_char('W').unwrap();
    tiles[34] = boggle::board::Tile::from_char('I').unwrap();
    tiles[35] = boggle::board::Tile::from_char('N').unwrap();

    let board = Board::new(6, tiles).unwrap();
    let path = find_word_path(&board, "WIN").expect("WIN path reaching cell 35");
    assert_eq!(path, vec![28, 34, 35]);
    assert!(contains_word(&board, "WIN"));
}

#[test]
fn test_on_demand_validator() {
    let mut trie = Trie::new();
    trie.insert("CATS").unwrap();
    trie.insert("DOGS").unwrap();
    trie.insert("BIRD").unwrap();
    trie.insert("ZEBRA").unwrap(); // in dictionary, but not on board

    let board = make_test_board();

    // Valid on board and in trie
    let res = validate_on_demand(&board, &trie, "cats");
    assert!(
        matches!(res, OnDemandResult::Valid { ref word, ref path } if word == "CATS" && path == &[0, 1, 2, 3])
    );

    // Too short (< 3)
    let res_short = validate_on_demand(&board, &trie, "at");
    assert!(matches!(res_short, OnDemandResult::TooShort { .. }));

    // In dictionary, not on board
    let res_nob = validate_on_demand(&board, &trie, "ZEBRA");
    assert!(matches!(res_nob, OnDemandResult::NotOnBoard { .. }));

    // On board, not in dictionary
    let res_nod = validate_on_demand(&board, &trie, "COED");
    assert!(matches!(res_nod, OnDemandResult::NotInDictionary { .. }));

    // Invalid non-ascii character
    let res_inv = validate_on_demand(&board, &trie, "CAT!");
    assert!(matches!(res_inv, OnDemandResult::InvalidWord { .. }));
}
