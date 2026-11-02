# Component-by-Component Mastermind Reuse Map

**Planning baseline: 2026-11-02.** Source: supplied `mastermind-main.zip`, inspected as a sibling of the supplied Word Games repository. Target: `Assignment 3/word-games`; reference repository: `Assignment 3/mastermind`.

**All target code and test destinations below are PLANNED, not implemented.** The existing `word-games` source currently contains only `trie` and `trie-cli`. Each game agent must reconcile the applicable rows to actual symbols, tests and commits as it implements them. A planned path is not evidence of reuse.

## 1. Reuse boundary and evidence

Mastermind is currently a binary crate: modules are exposed from `src/main.rs`, with no `src/lib.rs`. Its Cargo package is edition 2024 and depends on `rand = "0.8"`. Word Games' existing crates use edition 2021. Do not add a dependency on a binary-only sibling crate, include its `main.rs`, or change target edition solely to copy code. Adapt selected components into the target modules with their actual imports and ownership boundaries.

The source repository remains read-only and independently buildable. No shared third repository, source symlink, path dependency, git submodule or source-repo refactor is required. Record source provenance in this map and concise comments on substantially ported algorithms; do not label entirely new code as copied or claim that every numeric component can be reused unchanged.

The uploaded archive has no Git history; no source commit is known. During implementation obtain the actual sibling source commit if available and record it along with hashes. If source files differ from this snapshot, inspect them again and update the mapping rather than blindly assuming the old behavior.

| Inspected source file | SHA-256 of supplied bytes |
|---|---|
| `src/scoring.rs` | `40c8f2e7aecffa4cb850066ef10e8e9d8c06a60a59d07c9f3a26a1f7426c500f` |
| `src/game.rs` | `62d256c495e7b7bf89c943d9ec4d96933fcab44d6e06c0d056e2001d402215e8` |
| `src/main.rs` | `ed32e94ac3b74a507d1baa044d73adb7217658596ca00a5a2694f44ab2a5b298` |
| `src/ui.rs` | `97fd8bd3582c3ebdeb34fc67c04661d26aa270c9cddeeca86922c3b72aebef06` |
| `src/config.rs` | `87561de460061c080a962e205d54c94aca6bc45e3864cfa9038eee3ea6939644` |

## 2. Algorithm and session components

“Adapt” means preserve useful logic while changing types/contracts. “Design reuse” means preserve organization or invariants rather than copy source. “Do not port” identifies behavior incompatible with the chosen Wordle rules. Each row starts `PLANNED`; replace that status with actual implementation/test evidence when it lands.

| ID | Mastermind source component | Disposition and planned target | Required changes / retained behavior | Planned verification | Stage / status |
|---|---|---|---|---|---|
| MM-01 | `scoring.rs::score_guess` | Adapt → `wordle/src/feedback.rs::evaluate_markers` (WL-F1) | Keep two passes, exact matches first, one target occurrence per match; replace `u32` pegs and `(bulls,cows)` output with validated letters and five positional marks; allocate partial matches left to right | Exact arrays, duplicate traps, independent multiset oracle and aggregate count cross-check | Wordle / PLANNED |
| MM-02 | Two `target_used` / `guess_used` marker arrays and inner-loop `break` | Adapt inside WL-F1 | Preserve consumption semantics; fixed five-position masks/arrays can replace heap vectors without changing the matching algorithm; document any allocation change | Excess repeated letters never gain extra yellow marks; full-array equality | Wordle / PLANNED |
| MM-03 | `scoring.rs::calculate_score` | Do not port into Wordle | Its base points/difficulty/attempt multiplier is outside the accepted Wordle rules | Summaries report win/loss and attempts, not an inherited numeric score | Wordle / PLANNED exclusion |
| MM-04 | `scoring.rs::ScoreBreakdown` | Do not port | No Wordle difficulty multipliers; an outcome/attempt count suffices | Public API and UI contain no unused score contract | Wordle / PLANNED exclusion |
| MM-05 | `game.rs::GameSession` | Design reuse/adapt → `wordle/src/session.rs::WordleGame` | Retain attempts, history and round lifecycle; own lexicon, make secret/config/history private, validate construction, add persistent terminal-state guard | Invalid/repeated/post-finish and replay invariant tests | Wordle / PLANNED |
| MM-06 | `GameSession::new` | Adapt to a fallible validated constructor | Replace user-reachable `assert_eq!` failures with typed errors; secret must be a valid permitted answer | Bad-length/out-of-lexicon target rejected without panic | Wordle / PLANNED |
| MM-07 | `GameSession::attempts_remaining` | Adapt | Preserve remaining-attempt calculation under a fixed six-valid-attempt policy; private state prevents counter corruption | Initial value, each valid guess, invalid no-op, terminal zero/remaining values | Wordle / PLANNED |
| MM-08 | `GameSession::submit_guess` | Adapt | Keep evaluate → history → win-before-loss sequence; validate before consuming a turn; repeated valid words consume a turn; reject all post-finish submissions | Win on sixth, no seventh evaluation, history length equals valid attempts | Wordle / PLANNED |
| MM-09 | `game.rs::GuessEvaluation` | Adapt → Wordle evaluated-row type | Retain normalized guess and attempt index; use positional feedback instead of aggregate Bulls/Cows | Each stored row has five marks and correct attempt index | Wordle / PLANNED |
| MM-10 | `game.rs::GameOutcome` | Adapt to per-game outcome/status enums | Retain explicit win/loss/quit concepts; separate CLI EOF from engine state; expose secret only through permitted completed-round summary | Terminal-state and quit/reveal tests | Wordle; shared pattern / PLANNED |
| MM-11 | `game.rs::ReplayChoice` and `from_input` | Adapt → `word-games-cli/src/replay.rs` | Keep robust menu parsing; use new puzzle / settings or launcher / exit rather than forcing every game into numeric difficulty | All selections, invalid entries, empty input, EOF and settings transition | Bee first, reused later / PLANNED |
| MM-12 | `game.rs::GuessError`, `Display`, `Error` | Design reuse | Use typed, friendly per-game errors; commands are parsed separately; remove numeric-only variants and ordinary `quit` reservation | Normal malformed input reports an error without panicking or changing state | All games / PLANNED |
| MM-13 | `game.rs::parse_guess` | Do not copy numeric parser; reuse validation structure | Wordle validates one five-letter word; Bee/Boggle reuse existing trie tokenization/normalization and specified batch policy | Word/Unicode/length/delimiter/command tests | All games / PLANNED adaptation |
| MM-14 | `generate_target_with_rng<R: Rng>` | Reuse injection pattern, replace selection logic | Bee chooses eligible dictionary anchors; Boggle samples selected distributions; Wordle chooses from checked answers | Seed/config/hash reproduction, no global RNG dependency in engine tests | All games / PLANNED |
| MM-15 | `generate_target` numeric range sampler | Do not port numeric target generation | A random A–Z sequence is not a permitted Wordle answer; independent numeric pegs are unrelated to the lexicon | Every selected answer belongs to the curated subset | Wordle / PLANNED exclusion |
| MM-16 | `config.rs::Difficulty`, `DifficultyConfig`, `BASE_POINTS`; `main.rs::M`, `N` | Do not port numeric defaults | Bee has corpus-calibrated solution-count presets; Boggle has board-size/generator configuration; Wordle stays 5/6 without numeric difficulty bands | No hidden inherited attempts/length/point multipliers | All games / PLANNED exclusion |

The existing source `submit_guess` increments attempts without retaining a finished-state flag and exposes `secret`, configuration and counters publicly. It is a useful lifecycle reference, not an unchanged safe Wordle session API. Add and test the stronger contracts instead of copying those weaknesses.

WL-F2's 26-letter frequency scorer, `WordleLexicon`, answer curation, positional keyboard state, Bee/Boggle algorithms and all game-specific scoring are **new target components**. Do not falsely attribute these to numeric Mastermind source. WL-F1 and WL-F2 share validation/types but must remain independently implemented algorithms.

## 3. Interactive driver components

| ID | Mastermind source component | Disposition and planned target | Required changes / reuse | Planned verification | Stage / status |
|---|---|---|---|---|---|
| MM-17 | `main.rs::play_round<R: BufRead,W: Write>` | Adapt structure into each `word-games-cli` game adapter | Keep injectable buffered input/output, flush prompts, error recovery and EOF; call the new game engines and render their outcomes | Scripted sessions without real stdin; writer/I/O failures; clean EOF | Bee first, then Boggle/Wordle / PLANNED |
| MM-18 | `main.rs::select_difficulty` | Adapt menu/error-loop structure | Launcher/game settings replace numeric Mastermind-only choices; Wordle needs no difficulty prompt | Invalid choice recovery and settings defaults | Bee/launcher / PLANNED |
| MM-19 | `main.rs::ask_replay` | Adapt → shared CLI replay helper | Keep retry-on-invalid and graceful EOF; new-puzzle semantics respect explicit game configuration | Replay, settings change, exit, EOF without loops | Bee first, all games / PLANNED |
| MM-20 | `main.rs::run_with_color` | Design reuse → CLI dispatcher and session coordination | Keep injectable I/O and selected color mode; remove cumulative cross-game point arithmetic (scores across different games are not comparable) | Launcher routes only implemented games; no secret/score leakage across rounds | Bee first, all games / PLANNED |
| MM-21 | `main.rs::run` and production `main()` wiring | Adapt | Keep thin real-stdin/stdout entry point; argument parsing and launcher call testable functions | Binary smoke, nonzero startup errors and successful EOF | Bee first, all games / PLANNED |

Do not extract an elaborate `Game` trait merely to mirror these loops. Small concrete adapters and truly shared terminal/replay utilities satisfy the existing granular architecture with less coupling.

## 4. Terminal rendering components

| ID | Mastermind source component | Planned target / disposition | Required adaptation and checks | Stage / status |
|---|---|---|---|
| MM-22 | `ui.rs::ColorMode`, `style`, `bold`, `dim`, color helpers | `word-games-cli/src/terminal.rs` / adapt narrow utilities | Preserve reset behavior and explicit enabled/disabled modes; extend auto detection beyond only `NO_COLOR` to actual terminal detection; add explicit color override precedence tests | Bee first, all games / PLANNED |
| MM-23 | `ui.rs::render_progress_bar` | Shared terminal helper / adapt when useful | Use an honest progress measure: found/total for Bee/Boggle, used/remaining attempts for Wordle; avoid division by zero | Bee first, reused later / PLANNED |
| MM-24 | `ui.rs::render_banner`, `render_difficulty_menu` | Per-game banners/settings / design reuse | Reuse styling/panel organization, replace titles/rules/options; fit actual labels and word lengths | Bee first, all games / PLANNED |
| MM-25 | `ui.rs::render_status_panel` | Per-game status renderers / adapt | Bee/Boggle show cached points and found words; Wordle shows attempts/rows; do not carry numeric peg range or shared cumulative score | All games / PLANNED |
| MM-26 | `ui.rs::render_feedback` | `word-games-cli/src/wordle.rs` / adapt presentation pattern | Replace only Bulls/Cows totals with five positional cells; new keyboard prioritizes correct > present > absent > unknown; text fallback identifies every mark | Wordle / PLANNED |
| MM-27 | `render_win_summary`, `render_loss_summary`, `render_quit_summary` | Per-game summary renderers / adapt | Bee Queen Bee/missed words and Boggle score/missed words; Wordle attempts and permitted answer reveal; no old multipliers | All games / PLANNED |
| MM-28 | `ui.rs::render_error` | Shared formatting + per-game errors / adapt | Friendly message and recovery, no numeric-only instructions; typed command handling separate from engine failures | Bee first, all games / PLANNED |
| MM-29 | `ui.rs::render_replay_menu` | Shared CLI replay rendering / adapt | Match actual replay choices, implemented games and configuration semantics | Bee first, all games / PLANNED |

Avoid copying fixed-width panels without checking longest rendered words, ANSI width and the required three-column Bee output. A plain-text fallback should remain useful without interpreting color. The letter keyboard is new Wordle behavior; do not present it as copied Mastermind functionality.

## 5. Test-component reuse

| ID | Source test family | Target evidence to add | Important difference | Status |
|---|---|---|---|---|
| MM-T1 | `scoring.rs` deterministic Bulls/Cows and duplicate tests | Positional Wordle arrays and adapted excess-occurrence cases | Aggregate equality cannot prove which position is yellow | PLANNED |
| MM-T2 | Seeded `reference_score_guess` differential tests | Independent multiset-removal oracle, exhaustive 59,049 pairs, seeded dictionary/full-alphabet cases | Do not implement the oracle by calling WL-F1 or WL-F2 | PLANNED |
| MM-T3 | Numeric score bands / `calculate_score` tests | No direct port | Numeric difficulty/point formulas are intentionally excluded | PLANNED exclusion |
| MM-T4 | Game parser, RNG and session tests | Word validation, allowed/answer lexicons, no attempt on invalid input, repeat consumes attempt, final-attempt win, post-finish no-op | New lexical and terminal-state invariants | PLANNED |
| MM-T5 | `main.rs` injected-I/O scripted sessions and replay tests | Per-game adapter tests and compiled-binary subprocess transcripts | Bee/Boggle batch delimiters, slash commands, launcher, EOF and prior-game regression coverage | PLANNED |
| MM-T6 | `ui.rs` ANSI/plain tests, progress bars, summary tests | Reused helper tests plus per-game output/keyboard invariants | Avoid brittle tests of every decorative space; assert required readable content and no escapes in plain mode | PLANNED |

## 6. Evidence ledger to fill during implementation

For every implemented row add actual target path/symbol, source commit/hash, disposition, the behavior copied/adapted, target tests, implementing commit and reviewer evidence. A compact appended ledger is sufficient; keep the design rows above to explain the rationale. If a component is not beneficial to reuse, record why rather than making an unnecessary abstraction just to mark a row copied.

| Map ID | Actual target symbol/path | Source version | Disposition and rationale | Actual tests | Implementing commit | Independent review |
|---|---|---|---|---|---|---|
| MM-11 | `word-games-cli/src/replay.rs::ReplayChoice` | `src/game.rs` | Adapted menu parsing for game replays | `word-games-cli/tests/spelling_bee_cli.rs::test_replay_menu_choices` | `e3013e7` | Pending subagent review |
| MM-22 | `word-games-cli/src/terminal.rs::ColorMode` | `src/ui.rs` | Adapted ANSI style helpers, auto NO_COLOR detection | `word-games-cli/tests/spelling_bee_cli.rs::test_cli_play_session_batch_and_quit` | `aa168e0` | Pending subagent review |
| MM-28 | `word-games-cli/src/spelling_bee.rs` | `src/ui.rs` | Friendly error reporting and syntax validation | `word-games-cli/tests/spelling_bee_cli.rs::test_cli_play_session_batch_syntax_error_rejects_line` | `aa168e0` | Pending subagent review |
| MM-29 | `word-games-cli/src/replay.rs::prompt_replay` | `src/ui.rs` | Adapted replay menu with clean EOF handling | `word-games-cli/tests/spelling_bee_cli.rs::test_replay_menu_choices` | `e3013e7` | Pending subagent review |
| MM-T5 | `word-games-cli/tests/spelling_bee_cli.rs` | `src/main.rs` | Subprocess CLI stdin/stdout scripted tests | `word-games-cli/tests/spelling_bee_cli.rs::test_cli_binary_subprocess_quit` | `e3013e7` | Pending subagent review |

The independent reviewer must verify representative ported components directly against the sibling source and the full map against target paths/tests. Check that no source-repository files were changed, no cross-repo build dependency exists, and required Wordle behavior was not lost while maximizing reuse. Preserve existing source attribution; do not invent an upstream license or an official vocabulary source that the snapshot does not contain.
