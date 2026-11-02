//! Tests for Boggle board construction, validation, and neighbor adjacency.

use boggle::board::{Board, Tile, MAX_BOARD_SIZE, MIN_BOARD_SIZE};
use boggle::error::BoardError;
use trie::Letter;

#[test]
fn test_valid_board_sizes() {
    for size in MIN_BOARD_SIZE..=MAX_BOARD_SIZE {
        let tiles = vec![Tile::from_char('A').unwrap(); size * size];
        let board = Board::new(size, tiles).expect("valid board size");
        assert_eq!(board.size(), size);
        assert_eq!(board.cell_count(), size * size);
        assert_eq!(board.tiles().len(), size * size);
    }
}

#[test]
fn test_invalid_board_sizes_rejected() {
    for invalid_size in [0, 1, 2, 3, 7, 8, 10] {
        let tiles = vec![Tile::from_char('A').unwrap(); invalid_size * invalid_size];
        let res = Board::new(invalid_size, tiles);
        assert!(matches!(res, Err(BoardError::InvalidSize { .. })));
    }
}

#[test]
fn test_tile_count_mismatch_rejected() {
    let tiles = vec![Tile::from_char('A').unwrap(); 15]; // 15 instead of 16 for 4x4
    let res = Board::new(4, tiles);
    assert_eq!(
        res,
        Err(BoardError::InvalidTileCount {
            expected: 16,
            actual: 15
        })
    );
}

#[test]
fn test_precomputed_adjacency_properties() {
    for size in [4, 5, 6] {
        let tiles = vec![Tile::from_char('A').unwrap(); size * size];
        let board = Board::new(size, tiles).unwrap();

        let total = size * size;
        for i in 0..total {
            let r = i / size;
            let c = i % size;
            let neighbors = board.neighbors(i);
            let mask = board.neighbor_mask(i);

            // Check neighbor count by position
            let is_corner = (r == 0 || r == size - 1) && (c == 0 || c == size - 1);
            let is_edge = (r == 0 || r == size - 1 || c == 0 || c == size - 1) && !is_corner;

            if is_corner {
                assert_eq!(
                    neighbors.len(),
                    3,
                    "corner ({r},{c}) must have 3 neighbors on size {size}"
                );
            } else if is_edge {
                assert_eq!(
                    neighbors.len(),
                    5,
                    "edge ({r},{c}) must have 5 neighbors on size {size}"
                );
            } else {
                assert_eq!(
                    neighbors.len(),
                    8,
                    "interior ({r},{c}) must have 8 neighbors on size {size}"
                );
            }

            // No self-edges
            assert!(!neighbors.contains(&i), "cell {i} has self-edge");

            // No duplicate neighbors
            let mut unique = neighbors.to_vec();
            unique.sort_unstable();
            unique.dedup();
            assert_eq!(
                unique.len(),
                neighbors.len(),
                "cell {i} has duplicate neighbors"
            );

            // Symmetry check: j in neighbors(i) <=> i in neighbors(j)
            for &j in neighbors {
                assert!(
                    board.neighbors(j).contains(&i),
                    "asymmetry: {j} in neighbors({i}), but {i} not in neighbors({j})"
                );
            }

            // Neighbor mask matches neighbor vector
            let mut expected_mask = 0u64;
            for &n in neighbors {
                expected_mask |= 1u64 << n;
            }
            assert_eq!(mask, expected_mask, "neighbor mask mismatch for cell {i}");
        }
    }
}

#[test]
fn test_bit_35_coverage_on_6x6_board() {
    let size = 6;
    let tiles = vec![Tile::from_char('A').unwrap(); size * size];
    let board = Board::new(size, tiles).unwrap();

    let cell_35 = 35;
    assert_eq!(cell_35, size * size - 1);

    // Cell 35 is bottom-right corner (row 5, col 5). Neighbors: (4,4)=28, (4,5)=29, (5,4)=34
    let neighbors = board.neighbors(cell_35);
    assert_eq!(neighbors.len(), 3);
    assert!(neighbors.contains(&28));
    assert!(neighbors.contains(&29));
    assert!(neighbors.contains(&34));

    let mask_35 = board.neighbor_mask(cell_35);
    let expected = (1u64 << 28) | (1u64 << 29) | (1u64 << 34);
    assert_eq!(mask_35, expected);

    // Bit 35 bitmask validity
    let bit_35 = 1u64 << 35;
    assert_ne!(bit_35, 0);
}

#[test]
fn test_tile_qu_normalization() {
    let q_tile = Tile::from_char('q').unwrap();
    assert_eq!(q_tile, Tile::Qu);
    assert_eq!(q_tile.spelled_len(), 2);
    let (letters, len) = q_tile.spelled_letters();
    assert_eq!(len, 2);
    assert_eq!(letters[0], Letter::from_char('Q').unwrap());
    assert_eq!(letters[1], Letter::from_char('U').unwrap());
    assert_eq!(q_tile.to_string(), "Qu");

    let letter_c = Tile::from_char('c').unwrap();
    assert_eq!(letter_c, Tile::Letter(Letter::from_char('C').unwrap()));
    assert_eq!(letter_c.spelled_len(), 1);
    assert_eq!(letter_c.to_string(), "C");
}

#[test]
fn test_board_from_str_tiles() {
    let input = "
        C A T S
        D O G S
        Qu R E W
        B I R D
    ";
    let board = Board::from_str_tiles(4, input).expect("valid board string");
    assert_eq!(board.size(), 4);
    assert_eq!(board.tile(0), Tile::from_char('C').unwrap());
    assert_eq!(board.tile(8), Tile::Qu);
    assert_eq!(board.tile(15), Tile::from_char('D').unwrap());

    // Continuous string input with 'Q'
    let raw = "CATSDOGSQREWBIRD";
    let board2 = Board::from_str_tiles(4, raw).unwrap();
    assert_eq!(board2.tile(8), Tile::Qu);
    assert_eq!(board, board2);
}

#[test]
fn test_board_format_grid() {
    let raw = "CATSDOGSFROGBIRD";
    let board = Board::from_str_tiles(4, raw).unwrap();
    let grid_str = board.format_grid();
    assert!(grid_str.contains("| C  | A  | T  | S  |"));
    assert!(grid_str.contains("+----+----+----+----+"));
}
