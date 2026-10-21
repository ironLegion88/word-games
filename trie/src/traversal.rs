use crate::letter::Letter;
use crate::node::{NodeId, NO_CHILD};
use crate::Trie;

/// An immutable, read-only cursor for traversing the trie.
///
/// Borrows `&'a Trie` and holds a current position in the node arena.
/// Cursors do not allocate heap memory and do not expose internal node IDs.
#[derive(Copy, Clone, Debug)]
pub struct TrieCursor<'a> {
    trie: &'a Trie,
    current: NodeId,
}

impl<'a> TrieCursor<'a> {
    /// Creates a new cursor positioned at the root of the given trie.
    #[inline]
    pub(crate) fn new(trie: &'a Trie) -> Self {
        Self {
            trie,
            current: NodeId::ROOT,
        }
    }

    /// Advances the cursor to the child node corresponding to the given `Letter`.
    ///
    /// Returns `Some(TrieCursor)` if the edge exists in the trie, or `None` if absent.
    #[inline]
    pub fn advance(self, letter: Letter) -> Option<Self> {
        let child_raw = self.trie.nodes[self.current.as_usize()].children[letter.index()];
        if child_raw == NO_CHILD {
            None
        } else {
            Some(Self {
                trie: self.trie,
                current: NodeId(child_raw),
            })
        }
    }

    /// Returns `true` if the node at the cursor's current position represents a complete dictionary word.
    #[inline]
    pub fn is_word(&self) -> bool {
        self.trie.nodes[self.current.as_usize()].is_word
    }

    /// Returns `true` if the node at the cursor's current position has one or more outgoing child edges.
    #[inline]
    pub fn has_children(&self) -> bool {
        self.trie.nodes[self.current.as_usize()].has_children()
    }

    /// Returns the number of words stored in the subtree rooted at this cursor's current position.
    #[inline]
    pub fn subtree_words(&self) -> u32 {
        self.trie.nodes[self.current.as_usize()].subtree_words
    }
}
