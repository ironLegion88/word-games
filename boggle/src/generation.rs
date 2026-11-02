//! Board generation algorithms for Boggle.
//!
//! Provides three selectable board generation strategies:
//! - `BG-G1`: Independent uniform A-Z sampling (default baseline), with Q mapped to `Qu`.
//! - `BG-G2`: Frequency-weighted letter sampling derived from dictionary spellings with add-one smoothing.
//! - `BG-G3`: Synthetic 6-face dice simulation with largest-remainder allocation and Fisher-Yates cell shuffle.
//!
//! # Dictionary Sampling Bias (BG-G2 / BG-G3)
//! The letter frequencies used in BG-G2 and BG-G3 are derived directly from headword spellings
//! in `dictionary.txt` (SHA-256: `eda7bc8b86a534de8065a7d0cc091dc918d400475061749d05a5711afda12a3b`)
//! with add-one Laplace smoothing.
//!
//! These weights represent dictionary vocabulary distribution rather than natural-language
//! prose usage frequencies:
//! 1. Every distinct dictionary word has equal weight regardless of real-world conversational frequency.
//! 2. Inflectional affixes (such as plural '-S', past-tense '-ED', and '-ING') elevate counts of those letters.
//! 3. Rare or technical dictionary words contribute disproportionately compared to natural texts.

use rand::Rng;
use trie::{Letter, Trie, ALPHABET_SIZE};

use crate::board::{Board, Tile, MAX_BOARD_SIZE, MIN_BOARD_SIZE};
use crate::error::GenerationError;
use crate::solver::{solve, SolverStrategy};

/// Default retry budget for generating a playable board.
pub const DEFAULT_RETRY_BUDGET: usize = 100;

/// Version identifier for the BG-G1 uniform generator.
pub const BG_G1_VERSION: &str = "BG-G1-UNIFORM-V1";
/// Version identifier for the BG-G2 frequency-weighted generator.
pub const BG_G2_VERSION: &str = "BG-G2-WEIGHTED-V1";
/// Version identifier for the BG-G3 synthetic dice generator.
pub const BG_G3_VERSION: &str = "BG-G3-DICE-V1";

/// SHA-256 hash of the authoritative `assets/dictionary.txt` corpus.
pub const DICTIONARY_CORPUS_SHA256: &str =
    "eda7bc8b86a534de8065a7d0cc091dc918d400475061749d05a5711afda12a3b";

/// Versioned letter weights derived from `assets/dictionary.txt` with add-one smoothing.
///
/// Order: A through Z (`Letter::from_index(0..26)`).
pub const BG_G2_WEIGHTS: [u64; 26] = [
    120955, // A
    30125,  // B
    64171,  // C
    54874,  // D
    182744, // E
    20031,  // F
    43538,  // G
    36765,  // H
    140313, // I
    2675,   // J
    14452,  // K
    84620,  // L
    44856,  // M
    106773, // N
    103498, // O
    46601,  // P
    2585,   // Q
    112469, // R
    150217, // S
    104046, // T
    52110,  // U
    15430,  // V
    12419,  // W
    4762,   // X
    25871,  // Y
    7602,   // Z
];

/// Cumulative distribution table for fast binary-search sampling.
pub struct CumulativeWeights {
    cumulative: [u64; 26],
    total: u64,
}

impl CumulativeWeights {
    /// Constructs a cumulative distribution table from per-letter weights.
    pub const fn from_weights(weights: &[u64; 26]) -> Self {
        let mut cumulative = [0u64; 26];
        let mut sum = 0u64;
        let mut i = 0;
        while i < 26 {
            sum += weights[i];
            cumulative[i] = sum;
            i += 1;
        }
        Self {
            cumulative,
            total: sum,
        }
    }

    /// Samples a letter index `0..26` using an injected random number generator.
    #[inline]
    pub fn sample<R: Rng>(&self, rng: &mut R) -> usize {
        let r = rng.gen_range(0..self.total);
        self.cumulative.partition_point(|&w| w <= r)
    }
}

/// Precomputed cumulative table for `BG_G2_WEIGHTS`.
pub static BG_G2_CUMULATIVE: CumulativeWeights = CumulativeWeights::from_weights(&BG_G2_WEIGHTS);

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

/// Derives per-letter frequency weights from a dictionary trie with add-one Laplace smoothing.
pub fn derive_dictionary_weights(trie: &Trie) -> [u64; 26] {
    let mut counts = [1u64; 26];
    trie.visit_words(|word| {
        for b in word.bytes() {
            if b.is_ascii_alphabetic() {
                let idx = (b.to_ascii_uppercase() - b'A') as usize;
                if idx < 26 {
                    counts[idx] += 1;
                }
            }
        }
    });
    counts
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

/// BG-G2: Generates a board using frequency-weighted letter sampling from dictionary weights.
pub fn generate_weighted<R: Rng>(size: usize, rng: &mut R) -> Result<Board, GenerationError> {
    generate_weighted_with_weights(size, &BG_G2_CUMULATIVE, rng)
}

/// Generates a board using custom cumulative letter weights.
pub fn generate_weighted_with_weights<R: Rng>(
    size: usize,
    cumulative: &CumulativeWeights,
    rng: &mut R,
) -> Result<Board, GenerationError> {
    if !(MIN_BOARD_SIZE..=MAX_BOARD_SIZE).contains(&size) {
        return Err(GenerationError::InvalidSize { size });
    }

    let cell_count = size * size;
    let mut tiles = Vec::with_capacity(cell_count);

    for _ in 0..cell_count {
        let idx = cumulative.sample(rng);
        let letter = Letter::from_index(idx).expect("valid index");
        tiles.push(Tile::from_letter(letter));
    }

    Board::new(size, tiles).map_err(|_| GenerationError::InvalidSize { size })
}

/// Derives synthetic 6-face dice for an `N × N` grid using weighted largest remainders.
///
/// Follows Section 18 of the specification:
/// 1. Total face slots = `6 * N * N`.
/// 2. Allocates face counts using largest remainders with alphabetical tie breaking.
/// 3. Orders face letters deterministically and distributes slots round-robin to `N * N` dice.
/// 4. Maps 'Q' to `Tile::Qu`.
pub fn derive_dice_faces(size: usize, weights: &[u64; 26]) -> Vec<[Tile; 6]> {
    let total_dice = size * size;
    let total_slots = 6 * total_dice;

    let total_weight: u64 = weights.iter().sum();
    assert!(total_weight > 0, "total weight must be positive");

    // Compute quotas and integer floors
    let mut floors = [0usize; 26];
    let mut remainders = [0.0f64; 26];
    let mut allocated_sum = 0usize;

    for i in 0..26 {
        let quota = (weights[i] as f64) * (total_slots as f64) / (total_weight as f64);
        let floor = quota.floor() as usize;
        floors[i] = floor;
        remainders[i] = quota - floor as f64;
        allocated_sum += floor;
    }

    let needed = total_slots - allocated_sum;

    // Sort letter indices by remainder descending, with alphabetical (index) tie breaking
    let mut indices: Vec<usize> = (0..26).collect();
    indices.sort_by(|&a, &b| {
        remainders[b]
            .partial_cmp(&remainders[a])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.cmp(&b))
    });

    for &idx in indices.iter().take(needed) {
        floors[idx] += 1;
    }

    // Expand into ordered sequence of tiles
    let mut sequence = Vec::with_capacity(total_slots);
    for (i, &floor_count) in floors.iter().enumerate() {
        let letter = Letter::from_index(i).expect("valid letter index");
        let tile = Tile::from_letter(letter);
        for _ in 0..floor_count {
            sequence.push(tile);
        }
    }
    assert_eq!(sequence.len(), total_slots);

    // Distribute round-robin to N*N dice (6 faces each)
    let mut dice = vec![[Tile::Qu; 6]; total_dice];
    let mut die_face_counts = vec![0usize; total_dice];

    for (k, &tile) in sequence.iter().enumerate() {
        let die_idx = k % total_dice;
        let face_idx = die_face_counts[die_idx];
        dice[die_idx][face_idx] = tile;
        die_face_counts[die_idx] += 1;
    }

    for count in die_face_counts {
        assert_eq!(count, 6, "every die must have exactly 6 faces");
    }

    dice
}

/// Lazy or precomputed synthetic dice for size 4x4.
pub fn get_dice_for_size(size: usize) -> Result<Vec<[Tile; 6]>, GenerationError> {
    if !(MIN_BOARD_SIZE..=MAX_BOARD_SIZE).contains(&size) {
        return Err(GenerationError::InvalidSize { size });
    }
    Ok(derive_dice_faces(size, &BG_G2_WEIGHTS))
}

/// BG-G3: Generates a board by rolling one face from each synthetic die and shuffling cell assignments.
///
/// 1. Evaluates `N * N` synthetic 6-face dice.
/// 2. Rolls one face uniformly from each die.
/// 3. Performs a seeded Fisher-Yates shuffle of the `N * N` rolled tiles.
/// 4. Places the shuffled tiles onto the board cells.
pub fn generate_dice<R: Rng>(size: usize, rng: &mut R) -> Result<Board, GenerationError> {
    let dice = get_dice_for_size(size)?;
    let total_dice = size * size;

    let mut rolled_tiles = Vec::with_capacity(total_dice);
    for die in &dice {
        let face_idx = rng.gen_range(0..6);
        rolled_tiles.push(die[face_idx]);
    }

    // Seeded Fisher-Yates shuffle
    for i in (1..total_dice).rev() {
        let j = rng.gen_range(0..=i);
        rolled_tiles.swap(i, j);
    }

    Board::new(size, rolled_tiles).map_err(|_| GenerationError::InvalidSize { size })
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
