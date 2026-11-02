# Wordle Curated Answer Subset Provenance and Curation Notes

**Version:** 1.0.0 (2026-11-02)  
**File path:** `assets/wordle_curated_answers.txt`  
**Total entries:** 489 words  
**SHA-256:** `8DBC0086DA0C351C96D4D0064D3411058277E5A8BA566C4E8296A0ADF2E73C25`  
**Reference dictionary:** `assets/dictionary.txt` (178,691 words, SHA-256: `EDA7BC8B86A534DE8065A7D0CC091DC918D400475061749D05A5711AFDA12A3B`)

---

## 1. Purpose and Policy

The Wordle game design specifies that allowed player guesses are distinguished from potential puzzle secret answers:
1. **Allowed Guesses (`allowed_guesses`):** All valid 5-letter words contained in `assets/dictionary.txt` (8,938 total words).
2. **Curated Answers (`curated_answers`):** A curated subset of recognizable 5-letter English words supplied as the bundled default candidate answer pool (target: $\ge 100$ entries; bundled count: 489 entries).

### Explicit Non-Claims:
- This bundled list is **not** an official or proprietary Wordle answer corpus extracted from any third-party website, game application, or copyrighted source.
- This list makes **no claim** of corpus-measured empirical frequency ranking. Words were selected for common familiarity and phonetic transparency rather than any proprietary statistical frequency index.
- Overrides via the `--answers <PATH>` CLI option are fully supported and do **not** require $\ge 100$ words. Tiny user or test fixtures (e.g., 1 or 2 words) are explicitly permitted provided they satisfy the dictionary subset invariant.

---

## 2. Selection Criteria and Invariants

Every entry in `assets/wordle_curated_answers.txt` adheres strictly to the following invariants:
1. **Alphabet and Length:** Exactly 5 uppercase ASCII alphabetic characters (`A`..`Z`).
2. **Dictionary Invariant:** Every curated answer is guaranteed to exist in the 8,938 five-letter entries of `assets/dictionary.txt` without exception (0 missing out of 489).
3. **Format:** Exactly one uppercase word per line, terminated with Unix newline (`\n`), sorted lexicographically, and strictly free of duplicates.
4. **Vocabulary Selection:** Composed of common, recognizable everyday English words (e.g., `APPLE`, `BREAD`, `CHAIR`, `DANCE`, `EAGLE`, `FLAME`, `GRAPE`, `HOUSE`, `WATER`, `WORLD`).

