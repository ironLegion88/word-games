//! Procedural puzzle generation, anchor selection, and difficulty controls for Spelling Bee.

use crate::error::GenerationError;
use crate::puzzle::SpellingBeePuzzle;
use crate::solver::{solve, SolverStrategy};
use rand::Rng;
use std::collections::BTreeSet;
use trie::{Letter, Trie};

/// Pre-indexed list of eligible anchor words containing exactly 7 distinct letters.
#[derive(Clone, Debug)]
pub struct AnchorIndex {
    /// List of dictionary words that contain exactly 7 distinct letters.
    anchors: Vec<String>,
}

impl AnchorIndex {
    /// Builds an anchor index by scanning the dictionary for words with exactly 7 distinct letters.
    pub fn build(trie: &Trie) -> Result<Self, GenerationError> {
        let mut anchors = Vec::new();

        trie.visit_words(|word| {
            if word.len() < 7 {
                return;
            }

            let mut mask = 0u32;
            for b in word.bytes() {
                let idx = (b - b'A') as u32;
                mask |= 1 << idx;
            }

            if mask.count_ones() == 7 {
                anchors.push(word.to_string());
            }
        });

        if anchors.is_empty() {
            return Err(GenerationError::EmptyAnchorIndex);
        }

        anchors.sort_unstable();
        anchors.dedup();

        Ok(Self { anchors })
    }

    /// Creates an index from an explicit list of anchor words (useful for testing).
    pub fn from_vec(mut words: Vec<String>) -> Result<Self, GenerationError> {
        if words.is_empty() {
            return Err(GenerationError::EmptyAnchorIndex);
        }
        words.sort_unstable();
        words.dedup();
        Ok(Self { anchors: words })
    }

    /// Returns the number of eligible anchor words in the index.
    #[inline]
    pub fn len(&self) -> usize {
        self.anchors.len()
    }

    /// Returns `true` if the index contains no anchor words.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.anchors.is_empty()
    }

    /// Returns a slice of all indexed anchor words.
    #[inline]
    pub fn anchors(&self) -> &[String] {
        &self.anchors
    }

    /// Picks an anchor word using the provided RNG.
    pub fn choose_anchor<R: Rng>(&self, rng: &mut R) -> &str {
        let idx = rng.gen_range(0..self.anchors.len());
        &self.anchors[idx]
    }
}

/// Difficulty presets representing completion workload categories.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DifficultyPreset {
    Easy,
    Medium,
    Hard,
}

impl DifficultyPreset {
    /// Returns the default solution count bounds (min_words, max_words) for this preset.
    ///
    /// Calibrated against the supplied dictionary:
    /// - Easy: 15..=35 solutions
    /// - Medium: 36..=70 solutions
    /// - Hard: 71..=150 solutions
    pub fn bounds(&self) -> (usize, usize) {
        match self {
            Self::Easy => (15, 35),
            Self::Medium => (36, 70),
            Self::Hard => (71, 150),
        }
    }

    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "easy" => Some(Self::Easy),
            "medium" | "med" => Some(Self::Medium),
            "hard" => Some(Self::Hard),
            _ => None,
        }
    }
}

/// Configuration options for procedural puzzle generation.
#[derive(Clone, Debug)]
pub struct GenerationConfig {
    /// Optional explicit center letter override.
    pub center_override: Option<char>,
    /// Minimum required solution count.
    pub min_words: usize,
    /// Maximum allowed solution count.
    pub max_words: usize,
    /// Maximum number of generation attempts before returning an error.
    pub max_retries: usize,
    /// Difficulty preset if specified.
    pub preset: Option<DifficultyPreset>,
    /// Solver strategy to use when counting solutions during generation.
    pub solver_strategy: SolverStrategy,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            center_override: None,
            min_words: 10,
            max_words: 250,
            max_retries: 250,
            preset: None,
            solver_strategy: SolverStrategy::default(),
        }
    }
}

impl GenerationConfig {
    /// Applies a difficulty preset to the configuration, intersecting with any user bounds.
    pub fn with_preset(mut self, preset: DifficultyPreset) -> Self {
        let (preset_min, preset_max) = preset.bounds();
        self.preset = Some(preset);
        self.min_words = self.min_words.max(preset_min);
        self.max_words = self.max_words.min(preset_max);
        self
    }
}

/// Generates a valid Spelling Bee puzzle and its solution set according to the configuration.
pub fn generate_puzzle<R: Rng>(
    index: &AnchorIndex,
    trie: &Trie,
    config: &GenerationConfig,
    rng: &mut R,
) -> Result<(SpellingBeePuzzle, Vec<String>), GenerationError> {
    if config.min_words > config.max_words {
        return Err(GenerationError::InvalidBounds {
            min: config.min_words,
            max: config.max_words,
        });
    }

    for _ in 0..config.max_retries {
        let anchor = index.choose_anchor(rng);

        let mut distinct_letters = BTreeSet::new();
        for b in anchor.bytes() {
            if let Some(l) = Letter::from_char(b as char) {
                distinct_letters.insert(l);
            }
        }

        if distinct_letters.len() != 7 {
            continue;
        }

        let letters_vec: Vec<Letter> = distinct_letters.into_iter().collect();

        let center = if let Some(c) = config.center_override {
            let cl = match Letter::from_char(c) {
                Some(l) => l,
                None => continue,
            };
            if !letters_vec.contains(&cl) {
                continue;
            }
            cl
        } else {
            let c_idx = rng.gen_range(0..7);
            letters_vec[c_idx]
        };

        let outer_chars: Vec<char> = letters_vec
            .iter()
            .filter(|&&l| l != center)
            .map(|l| l.to_char())
            .collect();

        let puzzle = match SpellingBeePuzzle::new(center.to_char(), &outer_chars) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let solutions = solve(&puzzle, trie, config.solver_strategy);

        if solutions.len() >= config.min_words && solutions.len() <= config.max_words {
            return Ok((puzzle, solutions));
        }
    }

    Err(GenerationError::NoSuitablePuzzle {
        retries: config.max_retries,
    })
}
