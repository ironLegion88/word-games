//! Word Trie library crate.
//!
//! A high-performance, memory-resident dictionary based on an indexed arena trie
//! with safe node allocation, real branch pruning on deletion, free-list reuse,
//! and fragmentation-triggered compaction.

pub mod error;
pub mod letter;

pub use error::{LoadError, WordError};
pub use letter::{normalize_word, Letter, ALPHABET_SIZE};
