# Browser pre-dispatch recovery and safe observation

This repairs the operator-induced lock contention recorded in
`docs/relational-observation-live-v3-result.md`. That trial remains incomplete;
its opening retention is model behavior, its observer-lock failure and failed
manual restart are operator interventions. No new model result is implied.

## Contract

Campaign status reads take a verified read-only point-in-time snapshot without
locking the writer. They reject incomplete/corrupt chains. Status is never
spending authority: every writer still locks and revalidates the authoritative
ledger before appending. A changing snapshot may require another observation.

A native `NotDispatched` accounting record is separate from measured settlement.
It requires the exact existing unpriced reservation, current source root and
reviewed transport-proof digest; it cannot replace any settlement (including
unknown), reopen a closed episode or accept later usage. The reserved call and
token upper bound remain charged. It records no provider tokens, duration or
zero-charge receipt. Exact duplicate proof replay is idempotent.

The ordinary browser controller can create that proof only while holding its
launch lock, with unchanged saved body/source, no dispatch intent, no campaign
request identity, an empty preserved output directory, and no blocking unknown
campaign effects. Empty output alone is not proof. Adapter, session and checkpoint
Stop markers are checked, including immediately before the disposition. Only the
exact retained proposal body may follow; the next request has a new index and
output directory. Independent semantic review remains required. Unknown/partial
dispatch is never recovered by this route.

Existing checkpoint journals resume their saved callback before preparing a new
return assessment. This avoids the additional criterion input written by the
failed historical restart. Original failed request bodies, reservations and empty
output directories are retained.

## Boundaries

This is a transport attestation by trusted local code, not independent native
proof that HTTP never happened. The validator's source hash identifies the
reviewed rule implementation; it is not fabricated human or provider testimony.
The source-bound route currently supports proposal callbacks only. Review-stage
interruptions, source drift, unknown effects, and a crash between the pure
`predispatch_input` intent and result fail closed with retained evidence. They do
not trigger a guessed retry. A stopped project remains stopped.

No live continuation is authorized by this repair. Frozen launch bindings remain
unchanged and cannot run the modified implementation without separate prospective
admission. Historical accounting resolution, if performed, is an accounting-only
append under separately authenticated archived evidence, not reuse of expired or
changed launch authority.

## Verification

Authored fixtures only; none are live provider evidence. Detailed final results
and the historical reservation disposition are appended after verification.

## Final verified result

The historical native reservation `fdff1c6e…1b820` now has one append-only
`NotDispatched` disposition. Its **112,927-token reservation** is still counted:
**7 native reserved calls / 199,598 charged tokens** before and after. Native
unsettled and unknown usage/compute/cost counts went from one to zero; no tool
uncertainty existed. The six measured calls and **$0.010332** trial cost are
unchanged. No zero-token provider receipt was invented.

Campaign remains **469 calls / 149 operation groups / $0.861270 known**, plus the
older quarantined **$0.05 uncertain estimate** (not a verified maximum). This
repair made **zero provider calls and zero target executions**, and created no
new admission. Accounting is no longer blocked, but the trial is still incomplete
and was not resumed. Original frozen manifests, adapter/cycle journals, failure
logs and result report remain byte-identical. The appended proof/result is under
`target/relational-observation-live-v3-repair`; hashes are in
`docs/browser-predispatch-recovery-result.json`.

Checks on the final code:

- Python concurrency, proof, Stop, interrupted disposition, session ordering,
  transient-recovery and replay regressions: **65 passed / 1 skipped**. The skip
  is the pre-existing full B9.1 fake-worker test requiring an explicitly supplied
  `TSR_B91_TEST_WORKER_BIN`; it is not the new recovery test.
- Native unpriced accounting regression group: **13 passed**.
- Real native reservation → pre-dispatch interruption → disposition → restart →
  exact Unicode-containing body through authored transport: **1 passed**. Two
  native reservations remain counted; only one authored provider invocation.
- Actual-native unknown-bill/no-redispatch and honest-abstention controls:
  **2 passed**.
- `git diff --check`: passed. Build succeeded. No full-workspace/fmt/Clippy claim.

An initial closed-episode negative fixture failed because it tried to close an
unsettled episode, which native lifecycle correctly prohibits. The fixture now
settles its authored request, closes legitimately, and verifies the exact
closed-episode rejection. Both logs are retained under
`docs/validation/browser-predispatch`.

Independent `binding_review` verified the source fixes, request identity, native
reservation fields and current-source/no-dispatch proof before disposition. It
also caught two archival-helper defects before use (Unicode canonicalization and
elapsed-time equality); both were corrected and reviewed. These are offline
engineering interventions, not Luna behavior or additional live trial progress.

Safe monitoring going forward: use the verified read-only status snapshot and
read existing logs/receipts. Never use writer-locking admission helpers as polling
observers. Snapshot conflicts/incomplete writes are observation errors, not
permission to settle or retry effects. This repair does not grant a new live run.
