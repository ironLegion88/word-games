//! Boggle board representation, cell adjacency, and tile definitions.
//!
//! Enforces:
//! - Board dimension `N ∈ {4, 5, 6}` and exact tile count `N * N`.
//! - Immutable flattened tile vector with `Letter` or `Qu` tiles.
//! - Symmetric 8-neighbor precomputed adjacency without self-edges or duplicate neighbors.
//! - `u64` bitmask cell tracking accommodating up to 36 cells (bit 35 included).

use std::fmt;
use trie::Letter;

use crate::error::BoardError;

/// Supported minimum board dimension.
pub const MIN_BOARD_SIZE: usize = 4;
/// Supported maximum board dimension.
pub const MAX_BOARD_SIZE: usize = 6;

/// Constant representation of the letter 'Q'.
pub const LETTER_Q: Letter = match Letter::from_ascii_byte(b'Q') {
    Some(l) => l,
    None => unreachable!(),
};

/// Constant representation of the letter 'U'.
pub const LETTER_U: Letter = match Letter::from_ascii_byte(b'U') {
    Some(l) => l,
    None => unreachable!(),
};

/// A single face/tile on a Boggle board.
///
/// Under the standard Boggle rules, 'Q' is paired with 'u' as a single tile `Qu`,
/// which consumes one cell on the board while spelling two letters (`QU`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tile {
    /// A single standard letter 'A'..'Z' (excluding 'Q' which is represented as `Qu`).
    Letter(Letter),
    /// A `Qu` tile representing the two-letter sequence "QU".
    Qu,
}

impl Tile {
    /// Constructs a `Tile` from a `Letter`.
    ///
    /// If the letter is 'Q', it is automatically normalized to `Tile::Qu`.
    #[inline]
    pub const fn from_letter(letter: Letter) -> Self {
        if letter.index() == LETTER_Q.index() {
            Self::Qu
        } else {
            Self::Letter(letter)
        }
    }

    /// Constructs a `Tile` from a `char`.
    ///
    /// Normalizes 'Q' or 'q' to `Tile::Qu`. Returns `None` if `ch` is not an ASCII alphabetic character.
    #[inline]
    pub fn from_char(ch: char) -> Option<Self> {
        if ch == 'Q' || ch == 'q' {
            Some(Self::Qu)
        } else {
            Letter::from_char(ch).map(Self::from_letter)
        }
    }

    /// Returns the number of spelled letters contributed by this tile.
    ///
    /// Standard letters contribute 1; `Qu` contributes 2.
    #[inline]
    pub const fn spelled_len(self) -> usize {
        match self {
            Self::Letter(_) => 1,
            Self::Qu => 2,
        }
    }

    /// Returns the spelled letters contributed by this tile as a slice and count.
    #[inline]
    pub fn spelled_letters(self) -> ([Letter; 2], usize) {
        match self {
            Self::Letter(l) => ([l, l], 1),
            Self::Qu => ([LETTER_Q, LETTER_U], 2),
        }
    }

    /// Returns the uppercase string representation of this tile ("QU" or single letter).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Letter(l) => {
                const LETTERS: [&str; 26] = [
                    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P",
                    "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
                ];
                LETTERS[l.index()]
            }
            Self::Qu => "Qu",
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An immutable Boggle board of dimension `size × size`.
///
/// Contains precomputed symmetric 8-neighbor adjacency and neighbor bitmasks
/// for fast DFS without allocations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    size: usize,
    tiles: Vec<Tile>,
    neighbors: Vec<Vec<usize>>,
    neighbor_masks: Vec<u64>,
}

impl Board {
    /// Creates a new `Board` from dimension `size` and an exact `Vec<Tile>`.
    ///
    /// Returns `Err(BoardError)` if `size ∉ {4, 5, 6}` or `tiles.len() != size * size`.
    pub fn new(size: usize, tiles: Vec<Tile>) -> Result<Self, BoardError> {
        if !(MIN_BOARD_SIZE..=MAX_BOARD_SIZE).contains(&size) {
            return Err(BoardError::InvalidSize { size });
        }
        let expected_count = size * size;
        if tiles.len() != expected_count {
            return Err(BoardError::InvalidTileCount {
                expected: expected_count,
                actual: tiles.len(),
            });
        }

        let (neighbors, neighbor_masks) = Self::compute_adjacency(size);

        Ok(Self {
            size,
            tiles,
            neighbors,
            neighbor_masks,
        })
    }

    /// Precomputes 8-neighbor adjacency and bitmasks for every cell in an `N × N` grid.
    fn compute_adjacency(size: usize) -> (Vec<Vec<usize>>, Vec<u64>) {
        let total = size * size;
        let mut neighbors = Vec::with_capacity(total);
        let mut neighbor_masks = Vec::with_capacity(total);

        for cell in 0..total {
            let r = (cell / size) as isize;
            let c = (cell % size) as isize;
            let mut cell_neighbors = Vec::with_capacity(8);
            let mut mask = 0u64;

            for dr in -1..=1 {
                for dc in -1..=1 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    let nr = r + dr;
                    let nc = c + dc;
                    if nr >= 0 && nr < size as isize && nc >= 0 && nc < size as isize {
                        let n_idx = (nr as usize) * size + (nc as usize);
                        cell_neighbors.push(n_idx);
                        mask |= 1u64 << n_idx;
                    }
                }
            }

            neighbors.push(cell_neighbors);
            neighbor_masks.push(mask);
        }

        (neighbors, neighbor_masks)
    }

    /// Parses a `Board` from a string representation.
    ///
    /// Accepts letters separated by whitespace, commas, or continuous letter strings.
    /// Normalizes `'Q'` and `"Qu"` / `"QU"` to `Tile::Qu`.
    pub fn from_str_tiles(size: usize, input: &str) -> Result<Self, BoardError> {
        if !(MIN_BOARD_SIZE..=MAX_BOARD_SIZE).contains(&size) {
            return Err(BoardError::InvalidSize { size });
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(BoardError::EmptyInput);
        }

        let mut tiles = Vec::with_capacity(size * size);
        let chars: Vec<char> = trimmed.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            if ch.is_ascii_whitespace() || ch == ',' || ch == '|' || ch == '/' {
                i += 1;
                continue;
            }

            if ch == 'Q' || ch == 'q' {
                // Check if followed by 'U' or 'u'
                if i + 1 < chars.len() && (chars[i + 1] == 'U' || chars[i + 1] == 'u') {
                    tiles.push(Tile::Qu);
                    i += 2;
                } else {
                    tiles.push(Tile::Qu);
                    i += 1;
                }
            } else if let Some(tile) = Tile::from_char(ch) {
                tiles.push(tile);
                i += 1;
            } else {
                return Err(BoardError::InvalidCharacter { ch });
            }
        }

        Self::new(size, tiles)
    }

    /// Returns the dimension of the board (`4`, `5`, or `6`).
    #[inline]
    pub const fn size(&self) -> usize {
        self.size
    }

    /// Returns the total number of cells (`size * size`).
    #[inline]
    pub const fn cell_count(&self) -> usize {
        self.size * self.size
    }

    /// Returns the tile at the specified 0-based linear cell index.
    #[inline]
    pub fn tile(&self, index: usize) -> Tile {
        self.tiles[index]
    }

    /// Returns the tile at the specified 0-based (row, column).
    #[inline]
    pub fn tile_at(&self, row: usize, col: usize) -> Tile {
        self.tiles[row * self.size + col]
    }

    /// Returns a slice of all tiles in row-major order.
    #[inline]
    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    /// Returns the precomputed 8-neighbors of the given cell.
    #[inline]
    pub fn neighbors(&self, cell: usize) -> &[usize] {
        &self.neighbors[cell]
    }

    /// Returns the bitmask of all valid 8-neighbors of the given cell.
    #[inline]
    pub fn neighbor_mask(&self, cell: usize) -> u64 {
        self.neighbor_masks[cell]
    }

    /// Formats the board as a boxed text grid.
    pub fn format_grid(&self) -> String {
        let mut out = String::new();
        let col_width = 4;
        let horizontal_border = format!("{}+", "+----".repeat(self.size));

        for r in 0..self.size {
            out.push_str(&horizontal_border);
            out.push('\n');
            out.push('|');
            for c in 0..self.size {
                let tile = self.tile_at(r, c);
                let label = tile.as_str();
                let pad = col_width - label.len();
                let left_pad = pad / 2;
                let right_pad = pad - left_pad;
                out.push_str(&format!(
                    "{}{}{}|",
                    " ".repeat(left_pad),
                    label,
                    " ".repeat(right_pad)
                ));
            }
            out.push('\n');
        }
        out.push_str(&horizontal_border);
        out
    }
}
