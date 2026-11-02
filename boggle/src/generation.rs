//! Board generation algorithms for Boggle.
//!
//! Provides three selectable board generation strategies:
//! - `BG-G1`: Independent uniform A-Z sampling (default baseline), with Q mapped to `Qu`.
//! - `BG-G2`: Frequency-weighted letter sampling (Milestone BG-G2).
//! - `BG-G3`: Synthetic 6-face dice sampling with Fisher-Yates cell shuffle (Milestone BG-G3).
//!
//! Also provides bounded retry logic for normal play to ensure generated boards contain
//! at least one valid word.

use rand::Rng;
use trie::{Letter, Trie, ALPHABET_SIZE};

use crate::board::{Board, Tile, MAX_BOARD_SIZE, MIN_BOARD_SIZE};
use crate::error::GenerationError;
use crate::solver::{solve, SolverStrategy};

/// Default retry budget for generating a playable board.
pub const DEFAULT_RETRY_BUDGET: usize = 100;

/// Board generation strategy selector.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BoardGenerator {
    /// Independent uniform random letter sampling (BG-G1).
    #[default]
    Uniform,
    /// Letter frequency-weighted sampling (BG-G2).
    Weighted,
    /// Synthetic 6-face dice simulation (BG-G3).
    Dice,
}

impl BoardGenerator {
    /// Parses a generator from a CLI string argument.
    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "uniform" => Some(Self::Uniform),
            "weighted" => Some(Self::Weighted),
            "dice" => Some(Self::Dice),
            _ => None,
        }
    }
}

/// Generates a raw board of dimension `size` using the specified generator and injected RNG.
pub fn generate_raw_board<R: Rng>(
    size: usize,
    generator: BoardGenerator,
    rng: &mut R,
) -> Result<Board, GenerationError> {
    match generator {
        BoardGenerator::Uniform => generate_uniform(size, rng),
        BoardGenerator::Weighted => generate_weighted(size, rng),
        BoardGenerator::Dice => generate_dice(size, rng),
    }
}

/// BG-G1: Generates a board by uniformly sampling letters A-Z for each cell.
///
/// Any sampled 'Q' is automatically mapped to `Tile::Qu`.
pub fn generate_uniform<R: Rng>(size: usize, rng: &mut R) -> Result<Board, GenerationError> {
    if !(MIN_BOARD_SIZE..=MAX_BOARD_SIZE).contains(&size) {
        return Err(GenerationError::InvalidSize { size });
    }

    let cell_count = size * size;
    let mut tiles = Vec::with_capacity(cell_count);

    for _ in 0..cell_count {
        let idx = rng.gen_range(0..ALPHABET_SIZE);
        let letter = Letter::from_index(idx).expect("valid index");
        tiles.push(Tile::from_letter(letter));
    }

    Board::new(size, tiles).map_err(|_| GenerationError::InvalidSize { size })
}

/// Placeholder for BG-G2 weighted generation (implemented in Milestone BG-G2).
pub fn generate_weighted<R: Rng>(size: usize, rng: &mut R) -> Result<Board, GenerationError> {
    // Falls back to uniform until BG-G2 milestone
    generate_uniform(size, rng)
}

/// Placeholder for BG-G3 dice generation (implemented in Milestone BG-G3).
pub fn generate_dice<R: Rng>(size: usize, rng: &mut R) -> Result<Board, GenerationError> {
    // Falls back to uniform until BG-G3 milestone
    generate_uniform(size, rng)
}

/// Generates a playable board for normal play, guaranteeing at least one valid word.
///
/// Retries generation up to `retry_budget` times. If all attempts yield zero words,
/// returns `Err(GenerationError::NoPlayableBoard)`.
pub fn generate_playable_board<R: Rng>(
    size: usize,
    generator: BoardGenerator,
    trie: &Trie,
    solver_strategy: SolverStrategy,
    rng: &mut R,
    retry_budget: usize,
) -> Result<(Board, Vec<String>), GenerationError> {
    for _ in 0..retry_budget {
        let board = generate_raw_board(size, generator, rng)?;
        let solutions =
            solve(&board, trie, solver_strategy).map_err(|e| GenerationError::DiceConfigError {
                reason: e.to_string(),
            })?;
        if !solutions.is_empty() {
            return Ok((board, solutions));
        }
    }

    Err(GenerationError::NoPlayableBoard {
        retries: retry_budget,
    })
}
