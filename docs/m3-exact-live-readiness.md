# Exact-edit paired live validation: offline freeze

**Offline ready and independently reviewed; not launched.** This scope supersedes the earlier 10-request-per-arm proposal. The frozen scope is **20 model requests per arm, 40 total**, with zero retries.

Freeze: `docs/m3-exact-live-freeze.json`

SHA256: `bdd85d2a556b3154da57aeb1f5c92af70ee5fd9a5a877de457f8a88edc10545d`

All **535** frozen input hashes were rechecked locally and by the independent reviewer. The separate engine is `target/m3-exact-build/debug/tsr`, SHA256 `589b5466cfc6d2462990c5007ef64214d8ee1aeced0317e454f9a28c60cd4451`. The previous engine, evaluator and recorded evidence remain preserved. No old manifest has been refreshed to authorize this new baseline.

## Exact scope

| Item | Frozen choice |
| --- | --- |
| Task | `0c5a91a6aaa862c279aed1e1602a74eda7c42c4fc0f72247de513fa948f50b70` |
| Function | `ProtocolError::to_wire` |
| Source file | `crates/rts-daemon/src/error.rs` |
| Named test | `error::tests::protocol_error_round_trips_to_wire` |
| Selection | Lexicographically smallest task ID among the two previously qualified tasks in the prior plan; selected before new outcomes |
| Qualification | Existing accepted `baseline-full-green/named-red/full-red/exact-reversal/full-green-v1` admission, pinned by SHA256 |
| Order | Record, then rules; serial; no replacement task |
| Per-arm budgets | 20 requests, 1,800 seconds, 200,000 generator tokens, 15 tool minutes, 131,072 patch bytes |
| Pair request budget | 40 total; zero retries or resume |
| Money | At most $5 additional; $100 aggregate including the settled $0.229582 baseline; $0 outstanding |
| Output cap | 16,384 tokens per request |
| Source identity | Complete inspected file digest plus one unique exact text match; preserve untouched bytes |
| Initial state | Identical frozen inputs; independent empty per-arm output stores and startup SQLite paths; actual starting roots must match |

Both modes use the same engine, unchanged evaluator and oracle receipt verifier, exact-edit adapter, task/CAS/profile/preparation, and pinned sandbox image. Only the record/rules mode differs. Candidate commands stay in the existing sealed, non-networked sandbox as user `65534:65534`. No evaluated code receives host tools or credentials.

The controller uses a 4,200-second **dispatch/acceptance deadline**, starting before preflight, with at least 1,800 seconds plus a 300-second evidence margin available before starting an arm. Hash subprocesses are limited to 15 seconds and the receipt verifier to 60 seconds. Synchronous filesystem export is not hard-interruptible; this is not a guaranteed hard 4,200-second ceiling for all I/O. Each worker retains its 1,800-second process deadline and existing termination/reaping checks.

## Disclosure and pricing

Only the chosen public task ID, named test, source path, bounded source observations (including inline tests in that source file), bounded named-test stdout/stderr, action metadata, forecasts and controller progress may enter the model request. The hidden reversal/reference diff is not disclosed. Neither credentials, unrelated sessions/files, private logs nor host environment content enters the prompt. The existing credential is used only for API authentication. Request and response bodies are not persisted in diagnostics.

The fixed destination is `https://api.cheaperinference.com/v1/chat/completions`, model `deepseek-v4.1-flash`. Each inference is preceded by a fresh price-catalog check. Finite, nonnegative list prices must stay at or below **$0.30 input / $1.20 output per million tokens**. The wire limit and output cap feed the conservative request-cost bound. A $2 reservation must fit both the incremental and aggregate limits before dispatch, rechecked under the shared ledger lock. Successful responses must provide exact settled USD billing; unknown charges remain reserved and stop the pair. The new wrapper pins the correct $0.229582 baseline and $5 cap, replacing the inherited old budget binding without editing the historical transport.

Public documentation previously inspected does not establish `parallel_tool_calls` support for this model on Chat Completions. No such setting is assumed. Exactly-one validation remains authoritative regardless of instructions or provider behavior.

## Terminal evidence and failure policy

The controller writes a durable at-most-once intent before each dispatch. A completed arm requires process status and quiescence, exact reservation/settlement/diagnostic attribution, an explicit Stop proposal, exported task/profile/preparation/CAS/SQLite/results, verified CAS and event-derived roots, consistent closure, and an independently verified final-oracle receipt. Exported files receive a hash manifest. Visible tests are never substituted for final-oracle evidence.

Any malformed/multiple-call response, provider error, uncertain billing, process/precondition error, protected-test rejection, denial, missing evidence or export failure halts the whole pair. A legitimate final-oracle accepted or rejected result may proceed to the second arm. There are no retries, implicit repair, task substitution or deletion. Raw outputs and partial exports are retained; a partial export is not represented as verified. It needs independent salvage after quiescence before another experiment. The absence of a second-arm intent means it was unattempted.

## Validation actually performed

- Built the new engine offline with Rust 1.90.0 in a separate target directory; old binaries were not overwritten.
- **8** lifecycle/budget/pricing tests passed: $5 and 40-call bounds, unresolved reservations, exact baseline, one-shot state, failure halts, root mismatch, deadline headroom, helper timeouts, scope, and mocked settlement/protocol failures without retry.
- The **actual live wrapper and inherited transport** were exercised with only HTTP and credential lookup mocked. Real local reservation, settlement and diagnostics code fed the actual decoder; Rust then applied the edit, submitted Stop and passed through the unchanged evaluator to verified final-oracle receipts. This integration test passed **seven scenarios in each mode**: valid repair, stale digest, overlapping match, duplicate match, protected-test edit, multiple calls, malformed JSON. It ran from the same separate build directory as the frozen engine, with the comparable 20-request profile budget. All candidate source and test results were inert/synthetic; no candidate code ran.
- The prior implementation's 91 Rust tests, 20 Python tests, CLI check and clippy results remain documented in `m3-exact-edits.md`.
- All 106 prior export files remain identical in both file set and hashes. Of 225 prior readiness inputs, only the deliberate exact-edit driver source differs; the historical evaluator, adapter, controller, helper binaries and engine are intact.
- Read-only preflight passed at **2026-10-17 01:13:52 UTC**: freeze, Python interpreter, historical stopped-coordinator identity, idle workers/containers, locally present pinned image, disk reserve, exact settled baseline and fresh outputs. The initial sandbox could not run `ps`; an approved read-only escalation completed those checks. Nothing was started or resumed.
- Independent review confirmed the final freeze and found no remaining implementation blocker. Its initial receipt-helper timeout finding was fixed and tested before this freeze.

Evidence: `docs/m3-validation/m3-exact-freeze-{build,tests,integration}.log`, `docs/m3-exact-live-offline-preflight.json`, and `docs/m3-exact-live-review.json`.

## Remaining validation before paid execution

No known offline implementation blocker remains. **Paid execution is still pending scoped approval bound to the exact freeze hash.** No approval file has been created, no controller launched, and no provider calls made. Immediately before dispatch, recheck the frozen files, current ledger, fresh outputs, idle runtime, disk/image and live prices. The runtime checks here are a timestamped preparation check, not a guarantee of later availability.

Real provider compatibility, single-action compliance, real repair success and paired outcomes remain unmeasured. A two-arm pilot can establish usable behavior and receipts for this one task, not representative M3 acceptance or a performance advantage. Do not pool it with previous failures or call synthetic receipts real-corpus outcomes.
