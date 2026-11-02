# Boggle Algorithm Benchmark Report

- **Run ID:** `run-20261102`
- **Mode:** `FULL`
- **Toolchain:** `rustc 1.98.1` / `cargo 1.98.1`
- **Dictionary SHA-256:** `eda7bc8b86a534de8065a7d0cc091dc918d400475061749d05a5711afda12a3b`
- **Benchmark Seed:** `0x5EED2026`

## 1. Study 1: Complete Solution Enumeration (BG-S1E vs BG-S2)

| Size | Boards | Avg Words | Scan Avg Time | Trie Avg Time | Speedup | Set Equality |
|---|---|---|---|---|---|---|
| 4x4 | 10 | 34.1 | 40.99 ms | 53.97 µs | **759.5x** | 100% matched |
| 5x5 | 10 | 87.0 | 49.27 ms | 140.08 µs | **351.7x** | 100% matched |
| 6x6 | 10 | 97.7 | 52.19 ms | 146.81 µs | **355.5x** | 100% matched |

## 2. Study 2: Setup-Inclusive Snapshot vs On-Demand Queries

Compares end-to-end query trace cost on identical 4x4 boards across trace lengths:
- **On-Demand Baseline (BG-S1):** `Σ (t_dictionary_lookup + t_board_path)`
- **Trie Snapshot (BG-S2):** `t_setup + Σ t_snapshot_binary_search`

| Trace Length | On-Demand Total | Snapshot Setup | Snapshot Queries | Snapshot Total | Snapshot Advantage / Breakeven |
|---|---|---|---|---|---|
| 1 | 0.90 µs | 31.10 µs | 0.30 µs | 31.40 µs | Setup not repaid yet (+30.50 µs) |
| 10 | 2.50 µs | 31.10 µs | 1.30 µs | 32.40 µs | Setup not repaid yet (+29.90 µs) |
| 100 | 18.30 µs | 31.10 µs | 5.80 µs | 36.90 µs | Setup not repaid yet (+18.60 µs) |
| 1000 | 98.70 µs | 31.10 µs | 31.40 µs | 62.50 µs | 1.6x faster |

## 3. Study 3: Board Generation Quality Across Sizes 4, 5, 6

### 3.1 Raw Board Statistics (Production Trie Solver)

| Generator | Size | Boards | Avg Words | Avg Score | Max Length | Vowel % | Qu % | Zero-Word % | Gen Time | Solve Time |
|---|---|---|---|---|---|---|---|---|---|---|
| `uniform` | 4x4 | 100 | 31.2 | 36.2 | 7 | 20.4% | 0.50 | 3.0% | 2.89 µs | 43.6 µs |
| `uniform` | 5x5 | 100 | 69.5 | 90.8 | 9 | 23.0% | 0.99 | 1.0% | 3.75 µs | 74.3 µs |
| `uniform` | 6x6 | 100 | 123.5 | 167.0 | 9 | 21.6% | 1.43 | 0.0% | 5.30 µs | 149.1 µs |
| `weighted` | 4x4 | 100 | 112.5 | 174.1 | 9 | 36.9% | 0.03 | 1.0% | 2.07 µs | 108.9 µs |
| `weighted` | 5x5 | 100 | 247.7 | 449.4 | 10 | 39.6% | 0.11 | 0.0% | 4.72 µs | 270.3 µs |
| `weighted` | 6x6 | 100 | 448.3 | 948.6 | 12 | 37.3% | 0.05 | 0.0% | 7.78 µs | 800.9 µs |
| `dice` | 4x4 | 100 | 133.0 | 214.2 | 10 | 35.9% | 0.00 | 0.0% | 4.37 µs | 123.4 µs |
| `dice` | 5x5 | 100 | 256.4 | 463.7 | 11 | 37.4% | 0.00 | 0.0% | 11.13 µs | 299.6 µs |
| `dice` | 6x6 | 100 | 461.9 | 988.7 | 12 | 37.7% | 0.00 | 0.0% | 11.26 µs | 539.3 µs |

### 3.2 Playable Board Selection Policy (Conditioned Non-Empty Boards)

| Generator | Size | Play Rounds | Draws / Accepted Board | Exhaustion Rate | Avg Accepted Words | Avg Accepted Score |
|---|---|---|---|---|---|---|
| `uniform` | 4x4 | 30 | 1.00 | 0.0% | 36.7 | 45.6 |
| `uniform` | 5x5 | 30 | 1.00 | 0.0% | 58.1 | 70.9 |
| `uniform` | 6x6 | 30 | 1.00 | 0.0% | 137.5 | 188.2 |
| `weighted` | 4x4 | 30 | 1.00 | 0.0% | 124.4 | 206.5 |
| `weighted` | 5x5 | 30 | 1.00 | 0.0% | 298.0 | 588.9 |
| `weighted` | 6x6 | 30 | 1.00 | 0.0% | 433.1 | 907.6 |
| `dice` | 4x4 | 30 | 1.00 | 0.0% | 137.7 | 217.1 |
| `dice` | 5x5 | 30 | 1.00 | 0.0% | 281.3 | 513.1 |
| `dice` | 6x6 | 30 | 1.00 | 0.0% | 425.6 | 853.6 |

