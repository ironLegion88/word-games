//! Word Trie library crate.
//!
//! A high-performance, memory-resident dictionary based on an indexed arena trie
//! with safe node allocation, real branch pruning on deletion, free-list reuse,
//! and fragmentation-triggered compaction.

pub mod error;
pub mod letter;
pub mod loading;
pub mod node;
pub mod traversal;

pub use error::{LoadError, WordError};
pub use letter::{normalize_word, Letter, ALPHABET_SIZE};
pub use loading::{parse_tokens, BatchReport, LoadReport, ParsedToken};
pub use node::NO_CHILD;
pub use traversal::TrieCursor;

use node::{Node, NodeId};

/// Minimum number of freed slots required before automatic compaction is considered.
pub const DEFAULT_COMPACT_MIN_FREES: usize = 4096;
/// Compaction ratio numerator (e.g. 1 for 1/3 of arena).
pub const DEFAULT_COMPACT_RATIO_NUM: usize = 1;
/// Compaction ratio denominator (e.g. 3 for 1/3 of arena).
pub const DEFAULT_COMPACT_RATIO_DEN: usize = 3;

/// Detailed statistics about the internal arena and contents of the trie.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TrieStats {
    /// Total number of unique words stored in the trie.
    pub words: usize,
    /// Total number of live (reachable) nodes, including the root.
    pub live_nodes: usize,
    /// Total number of allocated node slots in the arena vector (`nodes.len()`).
    pub total_slots: usize,
    /// Number of free/reclaimable node slots available on the free list.
    pub free_slots: usize,
    /// Allocated heap capacity of the arena vector (`nodes.capacity()`).
    pub capacity_slots: usize,
}

/// A high-performance, memory-resident word trie based on an indexed arena.
///
/// Nodes are stored contiguously in a private vector arena and indexed by `NodeId`.
/// Child lookup is direct and O(1) per ASCII letter. Deletions perform true physical
/// branch pruning, recycling slots to an internal free list for immediate reuse,
/// and trigger automatic compaction when fragmentation exceeds thresholds.
///
/// # Examples
/// ```
/// use trie::Trie;
///
/// let mut trie = Trie::new();
/// assert!(trie.is_empty());
///
/// // Insert words (automatically normalized to uppercase)
/// assert!(trie.insert("CAR").unwrap());
/// assert!(trie.insert("cart").unwrap());
/// assert!(!trie.insert("CAR").unwrap()); // duplicate returns false
///
/// // Membership and prefix queries
/// assert_eq!(trie.len(), 2);
/// assert!(trie.contains("CAR"));
/// assert!(trie.contains("car")); // case-insensitive
/// assert!(!trie.contains("DOG"));
/// assert!(trie.has_prefix("CA"));
///
/// // Pruning deletion
/// assert!(trie.remove("cart").unwrap());
/// assert!(!trie.contains("cart"));
/// assert!(trie.contains("car")); // prefix branch preserved
/// ```
#[derive(Clone, Debug)]
pub struct Trie {
    /// Contiguous arena storage of nodes. Slot 0 is always the root.
    pub(crate) nodes: Vec<Node>,
    /// LIFO pool of recycled node slot indices available for safe reuse.
    pub(crate) free_list: Vec<NodeId>,
    /// Number of unique words stored in the trie.
    pub(crate) word_count: usize,
    /// Number of reachable live nodes in the trie (including root).
    pub(crate) live_node_count: usize,
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}

impl Trie {
    /// Creates a new, empty `Trie` containing only the root node.
    pub fn new() -> Self {
        Self {
            nodes: vec![Node::empty()],
            free_list: Vec::new(),
            word_count: 0,
            live_node_count: 1,
        }
    }

    /// Returns the number of unique words currently stored in the trie.
    #[inline]
    pub fn len(&self) -> usize {
        self.word_count
    }

    /// Returns `true` if the trie contains no words.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.word_count == 0
    }

    /// Returns current statistics on words, live nodes, arena slots, and memory usage.
    pub fn stats(&self) -> TrieStats {
        TrieStats {
            words: self.word_count,
            live_nodes: self.live_node_count,
            total_slots: self.nodes.len(),
            free_slots: self.free_list.len(),
            capacity_slots: self.nodes.capacity(),
        }
    }

    /// Inserts a word into the trie.
    ///
    /// The input string is normalized: lowercase ASCII letters are automatically converted
    /// to uppercase. Invalid characters (spaces, hyphens, numbers, non-ASCII) or an empty word
    /// return `Err(WordError)` without modifying the trie.
    ///
    /// Returns `Ok(true)` if the word was newly inserted, or `Ok(false)` if the word was already present.
    pub fn insert(&mut self, word: &str) -> Result<bool, WordError> {
        let letters: Vec<Letter> = letter::validate_and_parse_letters(word)?.collect();

        // Trace existing path to check if already present and count missing nodes
        let mut curr = NodeId::ROOT;
        let mut existing_len = 0;
        for &letter in &letters {
            match self.nodes[curr.as_usize()].get_child(letter) {
                Some(child) => {
                    curr = child;
                    existing_len += 1;
                }
                None => break,
            }
        }

        // If entire path already exists as nodes
        if existing_len == letters.len() {
            if self.nodes[curr.as_usize()].is_word {
                return Ok(false); // Already present
            }
            // Path existed as prefix of another word; mark terminal and update subtree counts
            let mut walk = NodeId::ROOT;
            self.nodes[walk.as_usize()].subtree_words += 1;
            for &letter in &letters {
                walk = self.nodes[walk.as_usize()]
                    .get_child(letter)
                    .expect("node exists");
                self.nodes[walk.as_usize()].subtree_words += 1;
            }
            self.nodes[walk.as_usize()].is_word = true;
            self.word_count += 1;
            return Ok(true);
        }

        // Preflight capacity check for the missing suffix
        let needed_nodes = letters.len() - existing_len;
        if self.nodes.len().saturating_add(needed_nodes) > node::MAX_NODE_ID as usize + 1 {
            return Err(WordError::CapacityExceeded {
                message: format!(
                    "cannot allocate {needed_nodes} nodes without exceeding maximum allowable NodeId"
                ),
            });
        }

        // Mutate along path
        let mut walk = NodeId::ROOT;
        self.nodes[walk.as_usize()].subtree_words += 1;

        for &letter in &letters {
            let next_node = match self.nodes[walk.as_usize()].get_child(letter) {
                Some(child) => child,
                None => {
                    let new_id = self.allocate_node()?;
                    self.nodes[walk.as_usize()].set_child(letter, new_id);
                    new_id
                }
            };
            walk = next_node;
            self.nodes[walk.as_usize()].subtree_words += 1;
        }

        self.nodes[walk.as_usize()].is_word = true;
        self.word_count += 1;
        Ok(true)
    }

    /// Removes a word from the trie, performing real branch pruning and free-list recycling.
    ///
    /// Accepts lowercase and uppercase ASCII words. If the word is not in the trie, returns `Ok(false)`
    /// without modifying the trie. If the input is malformed (empty or invalid characters),
    /// returns `Err(WordError)`.
    ///
    /// When a word is removed, any nodes that no longer lead to another valid word are detached,
    /// their arena slots are cleared and pushed onto the free list for immediate reuse.
    /// If arena fragmentation reaches the compaction threshold, automatic compaction is triggered.
    ///
    /// Returns `Ok(true)` if the word was present and removed, or `Ok(false)` if the word was absent.
    pub fn remove(&mut self, word: &str) -> Result<bool, WordError> {
        let changed = self.remove_internal(word, true)?;
        Ok(changed)
    }

    /// Internal removal helper allowing compaction to be deferred during bulk batch deletions.
    pub(crate) fn remove_internal(
        &mut self,
        word: &str,
        allow_compaction: bool,
    ) -> Result<bool, WordError> {
        let letters: Vec<Letter> = letter::validate_and_parse_letters(word)?.collect();

        // 1. Walk path and record parent/letter transitions
        let mut path: Vec<(NodeId, Letter)> = Vec::with_capacity(letters.len());
        let mut curr = NodeId::ROOT;

        for &letter in &letters {
            match self.nodes[curr.as_usize()].get_child(letter) {
                Some(child) => {
                    path.push((curr, letter));
                    curr = child;
                }
                None => return Ok(false), // Prefix does not exist
            }
        }

        // 2. Check if terminal node is actually marked as a word
        if !self.nodes[curr.as_usize()].is_word {
            return Ok(false); // Proper prefix exists, but word not present
        }

        // 3. Clear word flag and decrement word count
        self.nodes[curr.as_usize()].is_word = false;
        self.word_count -= 1;
        self.nodes[curr.as_usize()].subtree_words -= 1;

        // 4. Walk backward along the path toward the root
        let mut child_id = curr;
        for (parent_id, letter) in path.into_iter().rev() {
            self.nodes[parent_id.as_usize()].subtree_words -= 1;

            if self.nodes[child_id.as_usize()].subtree_words == 0 {
                // Child has no remaining words in its subtree; detach from parent and reclaim
                self.nodes[parent_id.as_usize()].clear_child(letter);
                self.free_node(child_id);
            }

            child_id = parent_id;
        }

        // 5. Evaluate automatic compaction threshold if enabled
        if allow_compaction {
            self.maybe_compact();
        }

        Ok(true)
    }

    /// Evaluates fragmentation and compacts the arena if the free list exceeds configured thresholds.
    pub(crate) fn maybe_compact(&mut self) {
        if self.should_compact(
            DEFAULT_COMPACT_MIN_FREES,
            DEFAULT_COMPACT_RATIO_NUM,
            DEFAULT_COMPACT_RATIO_DEN,
        ) {
            self.compact();
        }
    }

    /// Determines if current free list meets given threshold requirements.
    #[inline]
    pub(crate) fn should_compact(&self, min_frees: usize, num: usize, den: usize) -> bool {
        self.free_list.len() >= min_frees && self.free_list.len() * den >= self.nodes.len() * num
    }

    /// Compacts the arena into a dense array containing only live reachable nodes.
    ///
    /// Every live node is assigned a new consecutive slot starting at index 0 (the root).
    /// All child pointers are remapped, the free list is cleared, and arena capacity is shrunk.
    pub fn compact(&mut self) {
        if self.free_list.is_empty() {
            return;
        }

        let old_len = self.nodes.len();
        let mut old_to_new = vec![NO_CHILD; old_len];

        let mut new_nodes = Vec::with_capacity(self.live_node_count);
        let mut queue = std::collections::VecDeque::new();

        old_to_new[NodeId::ROOT.as_usize()] = 0;
        queue.push_back(NodeId::ROOT);

        // First pass: assign new consecutive IDs to reachable nodes
        while let Some(old_id) = queue.pop_front() {
            let old_node = self.nodes[old_id.as_usize()].clone();
            new_nodes.push(old_node);

            for &child_raw in &self.nodes[old_id.as_usize()].children {
                if child_raw != NO_CHILD {
                    let child_old_idx = child_raw as usize;
                    if old_to_new[child_old_idx] == NO_CHILD {
                        let child_new_id = (queue.len() + new_nodes.len()) as u32;
                        old_to_new[child_old_idx] = child_new_id;
                        queue.push_back(NodeId(child_raw));
                    }
                }
            }
        }

        // Second pass: rewrite all child references in new_nodes
        for node in &mut new_nodes {
            for child_raw in &mut node.children {
                if *child_raw != NO_CHILD {
                    *child_raw = old_to_new[*child_raw as usize];
                }
            }
        }

        new_nodes.shrink_to_fit();

        self.nodes = new_nodes;
        self.free_list.clear();
        assert_eq!(self.nodes.len(), self.live_node_count);
    }

    /// Checks if a word is in the trie.
    ///
    /// Accepts lowercase and uppercase ASCII words. If the word is invalid (contains non-ASCII,
    /// punctuation, spaces, or is empty), returns `false` without panicking.
    pub fn contains(&self, word: &str) -> bool {
        self.try_contains(word).unwrap_or(false)
    }

    /// Checks if a word is in the trie, returning a `WordError` if the input is malformed.
    pub fn try_contains(&self, word: &str) -> Result<bool, WordError> {
        if word.is_empty() {
            return Err(WordError::EmptyWord);
        }

        let mut curr = NodeId::ROOT;
        for (i, &b) in word.as_bytes().iter().enumerate() {
            let letter = match Letter::from_ascii_byte(b) {
                Some(l) => l,
                None => {
                    let ch = word[i..].chars().next().unwrap_or(b as char);
                    return Err(WordError::InvalidCharacter {
                        character: ch,
                        position: i,
                    });
                }
            };
            match self.nodes[curr.as_usize()].get_child(letter) {
                Some(child) => curr = child,
                None => return Ok(false),
            }
        }
        Ok(self.nodes[curr.as_usize()].is_word)
    }

    /// Checks if the trie contains any word with the given prefix.
    ///
    /// Accepts uppercase and lowercase ASCII letters. The empty prefix (`""`) always returns `true`.
    /// Malformed prefixes containing non-ASCII or punctuation return `false`.
    pub fn has_prefix(&self, prefix: &str) -> bool {
        if prefix.is_empty() {
            return true;
        }

        let mut curr = NodeId::ROOT;
        for &b in prefix.as_bytes() {
            let letter = match Letter::from_ascii_byte(b) {
                Some(l) => l,
                None => return false,
            };
            match self.nodes[curr.as_usize()].get_child(letter) {
                Some(child) => curr = child,
                None => return false,
            }
        }
        true
    }

    /// Creates an immutable cursor positioned at the root of the trie.
    pub fn cursor(&self) -> TrieCursor<'_> {
        TrieCursor::new(self)
    }

    /// Enumerates every active dictionary word in strictly lexicographic (alphabetical) order.
    ///
    /// The traversal is performed iteratively and does not recurse, preventing stack overflow
    /// on arbitrary word depths.
    pub fn visit_words(&self, mut visitor: impl FnMut(&str)) {
        if self.word_count == 0 {
            return;
        }

        let mut stack: Vec<(NodeId, usize)> = Vec::new();
        let mut current_word = String::new();

        if self.nodes[NodeId::ROOT.as_usize()].is_word {
            visitor(&current_word);
        }

        stack.push((NodeId::ROOT, 0));

        while let Some((curr_id, next_idx)) = stack.pop() {
            let curr_node = &self.nodes[curr_id.as_usize()];

            let mut found_next = false;
            for letter_idx in next_idx..ALPHABET_SIZE {
                let child_raw = curr_node.children[letter_idx];
                if child_raw != NO_CHILD {
                    stack.push((curr_id, letter_idx + 1));

                    let child_id = NodeId(child_raw);
                    let letter = Letter::from_index(letter_idx).expect("valid index");
                    current_word.push(letter.to_char());

                    if self.nodes[child_id.as_usize()].is_word {
                        visitor(&current_word);
                    }

                    stack.push((child_id, 0));
                    found_next = true;
                    break;
                }
            }

            if !found_next && curr_id != NodeId::ROOT {
                current_word.pop();
            }
        }
    }

    /// Allocates a node slot, either reusing a recycled slot from the free list or growing the arena vector.
    pub(crate) fn allocate_node(&mut self) -> Result<NodeId, WordError> {
        if let Some(reused_id) = self.free_list.pop() {
            self.nodes[reused_id.as_usize()].reset();
            self.live_node_count += 1;
            Ok(reused_id)
        } else {
            let next_index = self.nodes.len();
            let new_id = NodeId::try_from_usize(next_index)?;
            self.nodes.push(Node::empty());
            self.live_node_count += 1;
            Ok(new_id)
        }
    }

    /// Frees a node slot, clearing its contents and pushing its ID onto the free list.
    ///
    /// # Safety Invariant
    /// The caller must ensure that the parent node's edge to this child has already been detached,
    /// and that the node is not reachable from the root. The root node (`NodeId::ROOT`) can never be freed.
    pub(crate) fn free_node(&mut self, id: NodeId) {
        assert_ne!(id, NodeId::ROOT, "cannot free the root node");
        assert!(id.as_usize() < self.nodes.len(), "node id out of bounds");
        self.nodes[id.as_usize()].reset();
        self.free_list.push(id);
        self.live_node_count = self.live_node_count.saturating_sub(1);
    }

    /// Validates all structural invariants of the trie.
    pub fn check_invariants(&self) -> Result<(), String> {
        if self.nodes.is_empty() {
            return Err("nodes arena is empty: root node missing".into());
        }

        if self.free_list.contains(&NodeId::ROOT) {
            return Err("root node (0) found in free list".into());
        }

        let mut free_set = std::collections::HashSet::new();
        for &id in &self.free_list {
            if id.as_usize() >= self.nodes.len() {
                return Err(format!(
                    "free list id {} out of bounds (arena len {})",
                    id.0,
                    self.nodes.len()
                ));
            }
            if !free_set.insert(id) {
                return Err(format!("duplicate id {} found in free list", id.0));
            }
        }

        let mut visited = std::collections::HashSet::new();
        visited.insert(NodeId::ROOT);

        fn verify_node(
            trie: &Trie,
            id: NodeId,
            visited: &mut std::collections::HashSet<NodeId>,
            free_set: &std::collections::HashSet<NodeId>,
        ) -> Result<(usize, u32), String> {
            let node = &trie.nodes[id.as_usize()];
            if free_set.contains(&id) {
                return Err(format!("reachable node {} is present in free list", id.0));
            }

            let mut words_in_subtree = if node.is_word { 1u32 } else { 0u32 };
            let mut total_reachable = 1usize;

            for (idx, &child_raw) in node.children.iter().enumerate() {
                if child_raw != NO_CHILD {
                    if child_raw as usize >= trie.nodes.len() {
                        return Err(format!(
                            "node {} has out-of-bounds child {} for letter index {}",
                            id.0, child_raw, idx
                        ));
                    }
                    let child_id = NodeId(child_raw);
                    if !visited.insert(child_id) {
                        return Err(format!(
                            "cycle or shared child edge detected at node {}",
                            child_raw
                        ));
                    }
                    let (child_nodes, child_subwords) =
                        verify_node(trie, child_id, visited, free_set)?;
                    total_reachable += child_nodes;
                    words_in_subtree += child_subwords;
                }
            }

            if node.subtree_words != words_in_subtree {
                return Err(format!(
                    "subtree_words mismatch at node {}: stored {}, computed {}",
                    id.0, node.subtree_words, words_in_subtree
                ));
            }

            if id != NodeId::ROOT && words_in_subtree == 0 {
                return Err(format!(
                    "non-root reachable node {} has 0 subtree words",
                    id.0
                ));
            }

            Ok((total_reachable, words_in_subtree))
        }

        let (reachable_count, total_subtree_words) =
            verify_node(self, NodeId::ROOT, &mut visited, &free_set)?;

        if reachable_count != self.live_node_count {
            return Err(format!(
                "live_node_count mismatch: tracked {}, actual reachable {}",
                self.live_node_count, reachable_count
            ));
        }

        if total_subtree_words as usize != self.word_count {
            return Err(format!(
                "word_count mismatch: tracked {}, root subtree words {}",
                self.word_count, total_subtree_words
            ));
        }

        if reachable_count + self.free_list.len() != self.nodes.len() {
            return Err(format!(
                "arena slot leak: reachable ({}) + free ({}) != arena len ({})",
                reachable_count,
                self.free_list.len(),
                self.nodes.len()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_and_reuse_free_slots() {
        let mut trie = Trie::new();
        assert_eq!(trie.stats().total_slots, 1);
        assert_eq!(trie.stats().free_slots, 0);

        let id1 = trie.allocate_node().expect("alloc id1");
        assert_eq!(id1.0, 1);
        assert_eq!(trie.stats().total_slots, 2);
        assert_eq!(trie.stats().live_nodes, 2);

        let id2 = trie.allocate_node().expect("alloc id2");
        assert_eq!(id2.0, 2);
        assert_eq!(trie.stats().total_slots, 3);
        assert_eq!(trie.stats().live_nodes, 3);

        // Free id2
        trie.free_node(id2);
        assert_eq!(trie.stats().free_slots, 1);
        assert_eq!(trie.stats().live_nodes, 2);

        // Next allocation reuses id2
        let id3 = trie.allocate_node().expect("alloc id3");
        assert_eq!(id3, id2);
        assert_eq!(trie.stats().free_slots, 0);
        assert_eq!(trie.stats().total_slots, 3);
        assert_eq!(trie.stats().live_nodes, 3);
    }

    #[test]
    #[should_panic(expected = "cannot free the root node")]
    fn cannot_free_root() {
        let mut trie = Trie::new();
        trie.free_node(NodeId::ROOT);
    }

    #[test]
    fn invariant_detects_slot_leak() {
        let mut trie = Trie::new();
        trie.nodes.push(Node::empty());
        let err = trie.check_invariants().expect_err("should detect leak");
        assert!(err.contains("arena slot leak"), "actual: {err}");
    }

    #[test]
    fn invariant_detects_count_mismatch() {
        let mut trie = Trie::new();
        trie.word_count = 5;
        let err = trie.check_invariants().expect_err("should detect mismatch");
        assert!(err.contains("word_count mismatch"), "actual: {err}");
    }
}
