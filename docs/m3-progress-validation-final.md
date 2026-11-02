# Corrected M3 smoke: terminal report

The approved repeat **halted after its first arm became unavailable**. Launch was **2026-10-16 21:32:34.257636 UTC**; terminal controller status was recorded at **21:39:15.005333 UTC**. The remaining three arms were not started. No retries or resumption occurred.

**Incremental settled cost: $0.005842. Aggregate settled cost: $0.214469. Outstanding: $0.** All six reservations have settlements, including the response rejected for protocol shape. The previous $0.208627 ledger prefix is unchanged. These figures are provider-reported billing receipts, not an account invoice.

| Task | Mode | Terminal disposition | Paid calls | Final oracle |
|---|---|---|---:|---|
| ProtocolError::to_wire | record | unavailable: broker response shape | 6 | not invoked |
| ProtocolError::to_wire | rules | not attempted | 0 | not invoked |
| Route::to_wire | record | not attempted | 0 | not invoked |
| Route::to_wire | rules | not attempted | 0 | not invoked |

## What stopped the run

The first five model responses produced committed actions: **inspect → visible test → replace → replace → inspect**. The visible test exited 101 before either replacement, taking 199454 ms. The final inspection saw changed source bytes. There was no post-patch test and no valid Stop proposal.

The sixth response was billed and settled, then rejected by the adapter with **`tool_shape`**. This code means the response did not contain exactly one tool call named `propose`. The retained diagnostics do not distinguish zero calls, multiple calls, or a differently named tool. No provider response body was retained, so a narrower cause cannot be reconstructed. It is not recorded as a safety refusal, policy denial, transport timeout, or ambiguous charge. The Rust driver returned unavailable and the frozen controller stopped at its `arm unavailable; preserve and stop` guard.

This was not exhaustion of the 20-call budget. Six paid requests produced five committed actions; the rejected sixth response was not a committed action. The first process exited 0 after recording its unavailable outcome; that clean process exit is not task acceptance. The 400.382295-second arm duration includes setup; its raw driver wall clock is 255320 ms.

The exact action that remains unperformed is launching the other three queued provider-backed arms. They were withheld by the one-shot controller's fail-stop rule after the unavailable result. No automatic approval rejection occurred and no financial outcome remains uncertain.

## Roots, receipts, and interpretation

The only attempted arm started at the expected empty root:
`af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262`

Its terminal root is:
`f14bb2f6aec04524b63c255a3ee3fd4841edbeec84d12c99a5655f9f2c463e36`

The other arms have no observed starting or ending roots. Their prepared inputs match the repeated task scope, but **there are zero completed pairs**, so no observed paired-root equality or record-versus-rules performance result is claimed.

Final-oracle evaluation was never invoked. Consequently there are no final-oracle receipts from this live attempt; this is distinct from losing receipts after an executed oracle. Raw, closure-consistent, and receipt-verified accepted counts are all zero. The live run does not validate successful final submission or receipt capture and does not establish whether the progress change prevents repetition. Offline deterministic validation of those paths remains separate evidence.

M3 operational acceptance remains unmet. This run supplies no representative paired outcome comparison and no operational reviewed-tension evidence; earlier results are not pooled or reclassified.

## Evidence audit and preservation

The [terminal audit](m3-progress-validation-terminal-audit.json) passes for **68 exported files**, **52 BLAKE3 CAS objects**, **91 unchanged frozen inputs**, SQLite integrity, stored event-ID-derived roots, exact result-file bindings and ledger reconciliation. Root recomputation checks event IDs in sequence; it is not a new canonical validation of every event payload. The receipt verifier was prepared and independently reviewed, but no final oracle was reached in this run.

The prior run remains byte-for-byte preserved: its **72 frozen inputs** and exact **98-file export set/hashes** still match. The new executable remains `target/m3-evaluation-engine-progress-v2`, SHA256 `e1d52d0b836d560741fb2e16b3287ee6a963803b857f2f5fce49e8b84ca8d75f`, built from source commit `187ea213da70fa67f7ad9416f22a7e2ec746a45e`.

Runtime checks at **21:41:01.703547 UTC** found no active M3 workers or containers; historical coordinator PID29246 remained stopped with its expected identity. Free disk was 310.270 GiB. No existing artifacts were deleted.

## Reporting corrections and tests

Report-side corrections were independent of the frozen evaluator. The exporter accommodates the added oracle evidence field, exports each arm's CAS for receipt verification, and reports raw, closure-consistent and receipt-verified success separately. The audit now refuses to pass when an invoked oracle has missing or failed receipt verification; a valid checked negative result remains distinct. The read-only receipt checker and build/library provenance are pinned separately in [report-tool manifest](m3-progress-validation-report-tools.json).

A final display correction separates **unstarted arms** from **process failures**. The original export's legacy `process_failures` counted both; the corrected summary reports zero actual process failures, one unstarted record arm and two unstarted rules arms. The former count is retained as `unclean_or_unstarted_arms`, and [the original summary](m3-progress-validation-result-original-export.json) is preserved. Raw `missing` result counts describe absent records for these unstarted arms, not lost outputs from executed arms. No raw result, closure, cost, success count or acceptance criterion changed.

Validation completed for this launch/report work: three offline lifecycle/budget-load tests, four exporter tests and four audit tests passed. The receipt-audit defect has a retained fail-first regression. Independent preparation, reporting and terminal reviews found no remaining evidence-reporting blocker. The prior evaluator validation remains 122 Rust tests and two offline adapter tests, with one preexisting ignored test. Lefthook was unavailable; no hook pass is claimed.

## Deliverables and remaining work

Branch/workspace: `codex/m3-evidence-pipeline`, `/Users/n/Documents/Codex/2026-10-14/task/tsr-engine-m3`.

- [Frozen approved scope](m3-progress-validation-plan.json) and [authorization provenance](m3-progress-validation-authorization.json).
- [Results](m3-progress-validation-result.json), [per-arm accounting](m3-progress-validation-accounting.json), [export manifest](m3-progress-validation-export-sha256.json), and exported evidence in `docs/m3-progress-validation-execution/`.
- [Independent terminal review](m3-progress-validation-terminal-review.json).

The next diagnostic change, if pursued, should be offline and limited to non-content response-shape metadata (tool-call count, expected-name match, finish reason) and deterministic adapter tests. The current retained `tool_shape` code cannot identify the narrower defect. No additional cohort, retry, paid call after the halt, push, merge, or publication was performed.
