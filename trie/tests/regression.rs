use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use trie::Trie;

#[test]
fn full_dictionary_corpus_regression() {
    let dict_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("assets")
        .join("dictionary.txt");

    assert!(
        dict_path.exists(),
        "dictionary fixture not found at {}",
        dict_path.display()
    );

    // 1. Load dictionary using public from_reader API
    let file = File::open(&dict_path).expect("open dictionary.txt");
    let reader = BufReader::new(file);

    let start_load = std::time::Instant::now();
    let trie = Trie::from_reader(reader).expect("load dictionary");
    let load_duration = start_load.elapsed();
    println!("Loaded dictionary in {:?}", load_duration);

    // 2. Exact word count
    assert_eq!(trie.len(), 178_691, "expected exactly 178,691 unique words");

    // 3. Exact uncompressed trie node count (including root)
    let stats = trie.stats();
    assert_eq!(
        stats.live_nodes, 395_185,
        "expected exactly 395,185 live reachable nodes"
    );
    assert_eq!(stats.words, 178_691);
    assert_eq!(stats.free_slots, 0, "fresh load should have no free slots");
    assert_eq!(
        stats.total_slots, 395_185,
        "fresh load arena should be dense"
    );

    // 4. Verify structural invariants
    assert!(trie.check_invariants().is_ok());

    // 5. Verify boundary words and representative entries
    assert!(trie.contains("AA"));
    assert!(trie.contains("ZZZ"));
    assert!(trie.contains("ZYZZYVA"));
    assert!(trie.contains("ZYZZYVAS"));
    assert!(trie.contains("AARDVARK"));
    assert!(trie.contains("ZYGOTE"));

    // Case-insensitive verification
    assert!(trie.contains("aa"));
    assert!(trie.contains("zzz"));
    assert!(trie.contains("aardvark"));

    // Known absent words
    assert!(!trie.contains(""));
    assert!(!trie.contains("A")); // Dictionary minimum word length is 2
    assert!(!trie.contains("ZZZZ"));
    assert!(!trie.contains("NONEXISTENTWORD123"));
    assert!(!trie.contains("XYZXYZ"));
    assert!(!trie.contains("Ashoka")); // Ashoka is not in standard English dictionary

    // 6. Verify full enumeration equality and word length bounds
    let mut min_len = usize::MAX;
    let mut max_len = 0;
    let mut prev_word: Option<String> = None;
    let mut enumerated_count = 0;

    trie.visit_words(|w| {
        enumerated_count += 1;
        let len = w.len();
        if len < min_len {
            min_len = len;
        }
        if len > max_len {
            max_len = len;
        }

        if let Some(ref prev) = prev_word {
            assert!(
                prev.as_str() < w,
                "lexicographic order violation: '{prev}' should be < '{w}'"
            );
        }
        prev_word = Some(w.to_string());
    });

    assert_eq!(enumerated_count, 178_691);
    assert_eq!(min_len, 2, "expected minimum word length of 2");
    assert_eq!(max_len, 15, "expected maximum word length of 15");
}

#[test]
fn corpus_deletion_and_reinsertion_sample() {
    let dict_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("assets")
        .join("dictionary.txt");

    let file = File::open(&dict_path).expect("open dictionary.txt");
    let mut trie = Trie::from_reader(BufReader::new(file)).expect("load dictionary");

    // Sample 2,000 words starting with 'B'
    let mut b_words = Vec::new();
    trie.visit_words(|w| {
        if w.starts_with('B') && b_words.len() < 2000 {
            b_words.push(w.to_string());
        }
    });

    assert_eq!(b_words.len(), 2000);

    // Delete sampled words
    let mut oracle_removed = HashSet::new();
    for w in &b_words {
        assert!(trie.remove(w).unwrap());
        oracle_removed.insert(w.clone());
    }

    assert_eq!(trie.len(), 178_691 - 2000);

    for w in &b_words {
        assert!(!trie.contains(w));
    }
    // Sibling words starting with 'A' and 'C' should still be intact
    assert!(trie.contains("AA"));
    assert!(trie.contains("CAT"));

    assert!(trie.check_invariants().is_ok());

    // Reinsert sampled words
    for w in &b_words {
        assert!(trie.insert(w).unwrap());
    }

    assert_eq!(trie.len(), 178_691);
    for w in &b_words {
        assert!(trie.contains(w));
    }
    assert!(trie.check_invariants().is_ok());
}
