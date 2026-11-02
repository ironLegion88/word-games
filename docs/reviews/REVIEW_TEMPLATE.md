# Independent review — <game>

**TEMPLATE ONLY. NOT A COMPLETED REVIEW.** Replace placeholders with observed evidence; do not fill a check with a pass merely because an author supplied a successful log.

## Identity and scope

| Field | Observed value |
|---|---|
| Review date/time and timezone | <actual> |
| Reviewer subagent identifier | <actual; identify independence> |
| Repository and feature branch | <actual> |
| Base/main commit | <actual SHA> |
| Reviewed code commit | <actual SHA; report may be committed later> |
| Remediation commits reviewed | <actual or none> |
| Specs/assignments and requirement IDs | <actual files and coverage> |
| Dictionary, answer/dice and benchmark data versions | <hashes/versions> |
| Working-tree state during review | <clean/known generated artifacts; author edits paused> |

## Independent execution

| Exact command / probe | Environment and input | Exit/result | What it verifies |
|---|---|---|---|
| <command> | <fixture/seed/toolchain> | <PASS/FAIL/BLOCKED; actual counts> | <requirement> |

Include formatting, all-target compilation, strict Clippy, debug/release tests, release build, CLI transcripts, README examples, benchmark smoke and representative comparative reproduction. Record blockers and skipped coverage, without silently treating them as passes.

## Requirement and algorithm audit

| Requirement ID / version | Source evidence (path:line) | Independently inspected or executed evidence | Status / gap |
|---|---|---|---|
| <ID> | <actual> | <actual test/probe> | <result> |

Check algorithm independence, complete sets/positional feedback, ownership, duplicates, terminal state, scoring and overflow, input handling, data provenance, benchmark workload fairness/failures, review scope, untouched sibling source and accurate docs.

## Findings

For each finding provide:

1. ID and severity (P0 blocker, P1 serious correctness, P2 actionable defect/gap, P3 polish).
2. File and line; violated requirement or contract.
3. Minimal reproduction, expected behavior and observed behavior.
4. Proposed remediation and focused regression test.
5. Author fix commit, reviewer recheck and disposition.

If no findings remain, state what was independently checked and the practical limits. A bare “looks good” is not a review.

## Final disposition

- <READY FOR LOCAL MERGE / CHANGES REQUIRED / BLOCKED>
- <resolved findings and re-review evidence>
- <remaining documented limitations and any non-actionable observations>
- <whether all active requirement IDs, versioned algorithms, comparative evidence and documentation are covered>

The reviewer does not merge. The author performs the local merge only after this gate passes, then records post-merge checks and the real merge SHA in the final report/testing evidence. Do not fabricate a report's own future commit hash.
