use std::collections::HashSet;
use trie::Trie;

/// A lightweight, deterministic SplitMix64 pseudo-random number generator.
/// Zero external dependencies, 100% reproducible on all platforms.
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn gen_range(&mut self, min: usize, max: usize) -> usize {
        assert!(min <= max);
        min + (self.next_u64() as usize % (max - min + 1))
    }

    fn gen_word(&mut self, alphabet_len: usize, min_len: usize, max_len: usize) -> String {
        let len = self.gen_range(min_len, max_len);
        let mut s = String::with_capacity(len);
        for _ in 0..len {
            let letter_idx = (self.next_u64() as usize % alphabet_len) as u8;
            s.push((b'A' + letter_idx) as char);
        }
        s
    }
}

fn run_reference_model_test(
    seed: u64,
    num_ops: usize,
    alphabet_len: usize,
    min_len: usize,
    max_len: usize,
) {
    let mut rng = Rng::new(seed);
    let mut trie = Trie::new();
    let mut oracle: HashSet<String> = HashSet::new();

    // Pool of candidate words to encourage repeats, deletes, and shared prefixes
    let pool_size = 500;
    let word_pool: Vec<String> = (0..pool_size)
        .map(|_| rng.gen_word(alphabet_len, min_len, max_len))
        .collect();

    for op_idx in 0..num_ops {
        let op_type = rng.gen_range(0, 9);
        let word_idx = rng.gen_range(0, word_pool.len() - 1);
        let word = &word_pool[word_idx];

        match op_type {
            0..=4 => {
                // 50% Insert
                let trie_res = trie.insert(word).expect("valid word");
                let oracle_res = oracle.insert(word.clone());
                assert_eq!(
                    trie_res, oracle_res,
                    "seed 0x{seed:X}, op {op_idx}: insert mismatch for '{word}'"
                );
            }
            5..=7 => {
                // 30% Delete
                let trie_res = trie.remove(word).expect("valid word");
                let oracle_res = oracle.remove(word);
                assert_eq!(
                    trie_res, oracle_res,
                    "seed 0x{seed:X}, op {op_idx}: remove mismatch for '{word}'"
                );
            }
            _ => {
                // 20% Contains check
                let trie_res = trie.contains(word);
                let oracle_res = oracle.contains(word);
                assert_eq!(
                    trie_res, oracle_res,
                    "seed 0x{seed:X}, op {op_idx}: contains mismatch for '{word}'"
                );
            }
        }

        assert_eq!(
            trie.len(),
            oracle.len(),
            "seed 0x{seed:X}, op {op_idx}: length mismatch"
        );

        // Checkpoint validation
        if (op_idx + 1) % 500 == 0 || op_idx + 1 == num_ops {
            assert!(
                trie.check_invariants().is_ok(),
                "seed 0x{seed:X}, op {op_idx}: invariant failure"
            );

            // Compare sorted complete word list
            let mut iterated = Vec::new();
            trie.visit_words(|w| iterated.push(w.to_string()));

            let mut oracle_sorted: Vec<String> = oracle.iter().cloned().collect();
            oracle_sorted.sort();

            assert_eq!(
                iterated, oracle_sorted,
                "seed 0x{seed:X}, op {op_idx}: sorted iteration mismatch"
            );
        }
    }
}

#[test]
fn randomized_reference_seed_c0ffee() {
    // Balanced 1,000 operations
    run_reference_model_test(0x00C0_FFEE, 1000, 26, 1, 8);
}

#[test]
fn randomized_reference_seed_5eed2026() {
    // Shared-prefix-heavy churn (restricted alphabet 'A'..'H')
    run_reference_model_test(0x5EED_2026, 10000, 8, 2, 6);
}

#[test]
fn randomized_reference_seed_badc0de() {
    // High-volume churn test (20,000 operations)
    run_reference_model_test(0x0BAD_C0DE, 20000, 12, 1, 10);
}
