use crate::error::WordError;
use crate::letter::{Letter, ALPHABET_SIZE};

/// Sentinel indicating that a child branch does not exist for a given letter.
pub const NO_CHILD: u32 = u32::MAX;

/// Maximum allowable node index. `u32::MAX` is reserved for `NO_CHILD`.
pub const MAX_NODE_ID: u32 = u32::MAX - 1;

/// Strongly-typed private identifier for an arena node index.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u32);

impl NodeId {
    /// The root node always occupies slot 0 in the arena.
    pub const ROOT: Self = Self(0);

    /// Converts the `NodeId` to `usize` for direct slice indexing.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Converts a `usize` index into a `NodeId`, checking that it does not exceed `MAX_NODE_ID`.
    #[inline]
    pub fn try_from_usize(index: usize) -> Result<Self, WordError> {
        if index <= MAX_NODE_ID as usize {
            Ok(Self(index as u32))
        } else {
            Err(WordError::CapacityExceeded {
                message: format!(
                    "arena index {index} exceeds maximum allowable NodeId {MAX_NODE_ID}"
                ),
            })
        }
    }
}

/// A node within the indexed arena trie.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// Direct child node IDs for letters A-Z (indexed 0..26).
    /// `NO_CHILD` represents an absent child edge.
    pub children: [u32; ALPHABET_SIZE],
    /// True if this node represents the end of a valid word in the dictionary.
    pub is_word: bool,
    /// Number of valid active words in the subtree rooted at this node (including this node if `is_word`).
    pub subtree_words: u32,
}

impl Node {
    /// Creates a fresh empty node with no children, not a word, and 0 subtree words.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            children: [NO_CHILD; ALPHABET_SIZE],
            is_word: false,
            subtree_words: 0,
        }
    }

    /// Resets this node's slot so it can be safely stored on the free list or reused.
    #[inline]
    pub fn reset(&mut self) {
        self.children = [NO_CHILD; ALPHABET_SIZE];
        self.is_word = false;
        self.subtree_words = 0;
    }

    /// Returns the child `NodeId` for the given `Letter`, or `None` if absent.
    #[inline]
    pub fn get_child(&self, letter: Letter) -> Option<NodeId> {
        let child_raw = self.children[letter.index()];
        if child_raw == NO_CHILD {
            None
        } else {
            Some(NodeId(child_raw))
        }
    }

    /// Sets the child edge for the given `Letter`.
    #[inline]
    pub fn set_child(&mut self, letter: Letter, child_id: NodeId) {
        self.children[letter.index()] = child_id.0;
    }

    /// Clears the child edge for the given `Letter` back to `NO_CHILD`.
    #[inline]
    pub fn clear_child(&mut self, letter: Letter) {
        self.children[letter.index()] = NO_CHILD;
    }

    /// Returns true if this node has any outgoing child edges.
    #[inline]
    pub fn has_children(&self) -> bool {
        self.children.iter().any(|&c| c != NO_CHILD)
    }

    /// Returns the number of outgoing child edges (degree).
    #[inline]
    pub fn degree(&self) -> usize {
        self.children.iter().filter(|&&c| c != NO_CHILD).count()
    }
}
