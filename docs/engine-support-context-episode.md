# Prospective supporting-context continuation

This new development episode preserves the closed rejected episode documented in
[the prior outcome](engine-one-task-outcome.md). Evidence is under
`target/collection-support-episode/`. It starts from the original admitted mutant,
with independent rejection history carried through the normal API.

The editable window remains bytes 5862..6562 of `crates/rts-mcp/src/surface.rs`.
Only bytes 6562..7179 of its existing downstream validator are added as read-only
context: 617 bytes, no tests or reference repair. All other snapshot files and
outside-window bytes remain protected. The independent reviewer approved this
scope. Missing implementation context plausibly contributed to earlier semantic
mistakes, but does not establish the cause of type errors. History and starting
state also differ, so this is not a context-only causal experiment.

The prospective admission permits three local calls (240 seconds each), 55,296
model tokens, 32,768 reconstructed patch bytes, 2,400,000 tool milliseconds and
3,600,000 elapsed milliseconds, zero external charge. Normally at most two sealed
executions fit the patch budget. The command limit is 240,000 milliseconds because
the earlier cold compile used 118,836 milliseconds under a 120,000 limit. Two
four-command executions plus preparation reserve at most 2,160,000 milliseconds.
The image, installed model digest, source hashes and exact admission are preserved
in `preparation.json`, `provider-pin.json` and `admission.json`. Original requirements
remain unchanged. Full-suite success and independent semantic review are required
for acceptance; the existing optional-hook exclusion remains disclosed.

## Diagnostic correction

Named Cargo test output now has a 16 KiB capture and matching classifier bound,
instead of a 4 KiB cap. Compiler feedback retains at most 2,048 bytes, including
strict handling of lossy UTF-8 expansion. Truncated output and exit 125 still fail
closed. No arbitrary logs or extra source are sent. This fixes a mismatch found by
independent review: increasing capture alone would leave classification rejecting
complete output at the old limit.

Validation: 135 CLI library tests passed, three ignored; CLI all-target clippy with
warnings denied passed. Focused regressions cover complete output between the old
and new bounds, truncation, exit 125 and diagnostic byte limits. Independent review
found no remaining blocker. The protected cutoff tests remain unchanged; no broad
suite-clean claim is made.

## Terminal result: rejected

| Call | Input + output tokens | Result |
| --- | --- | --- |
| 1 | 1,643 + 320 | Scoped candidate; compiler E0425, exit 101 after 109,493 ms. Independent review rejected silent filtering and invalid lexical scope. No named test ran; full suite not reached. |
| 2 | 3,557 + 299 | Scoped candidate independently rejected before execution: same lexical/filtering defects plus an unterminated result expression preceding the immutable suffix. No compiler/test outcome is claimed for it. |
| 3 | 3,576 + 295 | Unchanged current source; driver stopped before execution. |

All three calls are terminal. Only one sealed execution was consumed; no useful
candidate remained for the second. No replacement implementation was authored for
the generator. The additional context did not establish recovery: repeated scope
and validation mistakes remain a demonstrated limitation of this pinned provider
on this task. A further identical retry is not justified. The next useful provider
work would require a separately bounded admission and a demonstrably different
reasoning/interface strategy; no further call is dispatched here.

Totals: 9,690 measured tokens, 62,653,542,041 ns local model duration, 115,265 tool-ms,
13,496 reconstructed patch bytes, zero external charge. Final ledger reports no
unknown usage, unsettled requests, unresolved tool requests or budget excess.
Temporary provider PID 65577 was stopped; its session returned exit 0. Integrity
checks verified all 323 frozen source files, 322 protected working files and bytes
outside the original window. Original target source was not edited.

Episode `7716e10660d97602cc1e2e8dd16ccf702e6196f4b432386a9f85e2c793792b5c`
closed rejected at **2026-10-22T07:25:41.207Z**. Earliest calendar maturity is
**2026-10-29T07:25:41.207Z**, not an eligibility assertion. Eligibility remains
`missing_independent_coverage_review`, label null. Active reviewer minutes are
unknown (`review-effort.json`); no positive complete-resource attestation was
fabricated. No full-suite pass, new independent task, mature episode, calibrated
claim or Tier2 activation is reported. Prior episode remains closed and unchanged.
The 151 qualified corpus tasks remain distinct from an unknown mature-eligible count.

Evidence includes admission/open/closure, preserved local-request journals,
`execution-1.json`, bound `review-1.json`, independent pre-execution
`independent-review-2.json`, shared `feedback-{1,2}.json`, `final-budget.json`,
`final-eligibility.json` and `integrity.json` under the evidence directory.

`cargo fmt --all -- --check` also reported 169 formatting differences across the
existing workspace; no global reformat was applied. CLI unit tests, clippy and
`git diff --check` passed; this is not a clean full-workspace validation claim.
