# One-task local episode: rejected, with evidence preserved

The prepared RTS task was run under its frozen admission on 2026-10-22. It did
**not** produce an accepted repair. No source, test, budget or desired criterion
was weakened. This is one development episode, not a reliability benchmark.

Evidence directory: `target/collection-one-ready-v4/`.
Episode: `e92a77c06b47e83fb4910b917d8422d5d26fccc8224c39d154d53a1f8943073c`.
Original task: `39385564a640e614fb7a8958eef24266728525a5364238611d2ab52bc345a2d7`.
Provider/model/image/scope remained as pinned in [the admission](engine-one-task-readiness.md).

| Call | Measured tokens (input + output) | Result |
| --- | --- | --- |
| 1 | 944 + 295 | Valid scoped proposal; sealed compilation returned 101 with E0308. Independent review rejected profile-only filtering that discarded unknown and observability names. |
| 2 | 3,096 + 351 | Returned unchanged source despite feedback; rejected before execution. |
| 3 | 3,102 + 381 | Different scoped proposal, but retained the defects and added validation after filtering. Independent review rejected it. Second execution emitted E0308 and hit diagnostic output cap, returning 125/unavailable. |

Neither named test executed. Consequently neither execution reached the full
workspace oracle. The optional-hook exclusion remains disclosed; no suite pass is
claimed. Generator rationales were not treated as proof. Bound independent reviews
and explicitly published feedback are `review-{1,2}.json` and `feedback-{1,2}.json`;
no replacement answer was supplied. Model-authored proposals and all terminal
attempt/resource journals remain under `episode/`.

Measured cumulative resources: **3 calls, 8,169 tokens, 59,999,383,291ns local model
duration, 234,276 tool-ms, 27,472 patch bytes, zero external charge**. The final
ledger has zero unknown usages, unsettled requests or unresolved tool requests;
no frozen budget was exceeded. The third generation slot and two normal
executions were consumed; no further calls were made. The temporary loopback
provider PID 30927 was stopped and its session completed. Protected 322 files,
bytes outside the edit window and original repository status were checked unchanged.

Closure was **2026-10-22T06:59:02.759Z**, independently reviewed rejected.
Earliest possible calendar maturity is **2026-10-29T06:59:02.759Z**. This is not an
eligibility claim: `final-eligibility.json` reports
`missing_independent_coverage_review`, label null. Both generator common-event
forecasts were omitted; frozen controller expectations and evidence-only shadow
records do not substitute for missing paired generator probabilities.

Active reviewer minutes are **unknown**, recorded explicitly in `review-effort.json`.
Wall-clock turnaround is not active effort. A false complete-coverage submission
was attempted and refused by the current API, which accepts positive attestations
only; the attempted file is preserved as `coverage-review.json`. No positive
attestation was fabricated or retried. Missing review continues to block training
eligibility. Full-suite coverage is also absent. The 151 qualified corpus tasks
remain distinct from independently mature episodes; no new mature count, model
fit or Tier2 activation is claimed.

## Concrete follow-up implemented without inference

Independent audit found a restricted-context limitation: the generator could see
only its editable span, not the downstream validator it had been told to preserve.
Feedback described that validator but did not expose its implementation. This does
not prove that context caused every failure or that more context guarantees repair.

`ProjectSpec.supporting_spans` now optionally admits bounded read-only implementation
observations for **future** configurations. It maps already-admitted paths to ordered
initial UTF-8 byte ranges (`start`, `end`). Defaults remain empty and legacy
serialization is unchanged. At most eight spans, 2,048 bytes each and 4,096 bytes
combined are allowed, within the existing 8,192-byte total source context bound.
Spans must lie in protected files or outside a file's edit window. Immutable suffix
spans anchor to EOF so variable-length repairs do not shift their meaning. Each
request binds current file and span digests and labels the content noneditable;
no patch handle or additional action authority is supplied. The local transport
forwards these observations while its response schema still allows edits only to
original editable paths. Source remains untrusted data, not instructions.

Operator admission must inspect supporting ranges to exclude tests, reference
answers and unrelated material. Path/test-marker checks are a coarse guard, not a
proof that an arbitrary span is not inside an inline test. Context telemetry records
file hashes, not precise span exposure or whole-file reading; request content and
span digests describe the actual offered observations. No understanding is inferred.

The closed episode, 323-file snapshot, 700-byte edit window and frozen budgets were
not altered or reopened. No extra source was transmitted to a model. The next
operational step is a separately admitted continuation with reviewed supporting
implementation spans and preserved rejection history, only after a new bounded
local-call allowance. Repeating the unchanged restricted prompt is not the next
step. No external provider or publication is needed.

Verification: 133 CLI library tests passed, 3 ignored; 18 offline local-transport
Python tests passed; CLI all-target clippy with warnings denied and diff check
passed. Tests cover resizing/restart, unchanged control, stale file binding,
forbidden edits, unknown/overlapping/test/invalid UTF-8/oversized spans and transport
authority separation. Independent review found no blocking defect and supplied the
limitations above. Logs: `target/collection-support-{all-tests,python,clippy}.log`.
The two protected cutoff tests remain unchanged; no broad-suite-clean claim is made.
