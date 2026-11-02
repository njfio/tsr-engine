# Action-specific live integration: wire contract passed, completion blocked

The new adapter worked with the real provider for all four responses: inspect, edit, test, edit. The first edit repaired the synthetic function to return 42, and its visible named test exited 0. The fourth response proposed another edit, but the native engine rejected its source digest before proposal recording or execution. There was no explicit Stop, task closure, final oracle, or native result row. This is an incomplete failed attempt, not an accepted task.

## Change and validation

`exact-edit@v3` uses one shared action contract for schema, examples and decoder required fields:

| Action | Exact required fields |
| --- | --- |
| inspect | action, path |
| test | action, test_id, forecast |
| edit | action, path, expected_digest, old_text, new_text, forecast |
| stop | action |

No fields are optional; unknown/inactive fields and legacy `id` are rejected. `test_id` must be supplied explicitly and belong to allowed_test_ids. Only after validation does it map to unchanged internal `Proposal::Test { id }`. Forecasts remain finite numbers in [0,1] for test/edit. Exact paths, digest format/current-source comparison, nonempty unique source spans, protected tests, evidence and budgets remain enforced. No test selector is invented or defaulted. Non-action bookkeeping stays host-owned.

Content-free diagnostics now retain a finite action category on success and validation failure, alongside fixed field paths, JSON types and finite reasons. Unknown strings and arbitrary keys are not recorded. Examples and instructions identify the allowed test IDs explicitly and contain no inactive placeholders.

Five new tests passed, including 132 cases through schema/decoder and mocked transport with real temporary reservation/settlement logs. Twenty existing broker tests also passed. Tests include every action, missing and extra fields, supplied/unknown/legacy test IDs, invalid types, bounds, malformed/duplicate/deep JSON, no-op edits, multiple calls, and separation of trusted request correlation from semantic test IDs. No new request-ID scheme was introduced.

The scripted full-path fixture used the production sealed worker and strict v3 adapter in fresh state, completing inspect→failing test→edit→passing test→Stop→protected final oracle. Its exported receipt `bc6fa3e0cee173068f01462801443bc8fc81980325f7ff1106f83c024a33bfad` was independently reread and verified. Independent review confirmed all 14 local evidence hashes and all 603 frozen inputs before live launch. This local success is separate from the live outcome below.

## Live attempt

Freeze: `d5de0a72d4e52d4c0af797f282fcf11b212c736d605e21b96ed8e4a7d724a97b`.

Worker 84878 ran in record mode from 2026-10-17 12:00:56.265871 UTC to 12:01:36.358185 UTC. It exited 2; containment cleanup reported no errors. All four provider responses passed the action-specific schema/decoder and their charges settled. Three proposals were persisted: inspect, propose_patch, run_approved_test. The executed test's exit code was 0.

The process reported `stale or oversized edit source`. The retained source is a small regular file below 65536 bytes, so the failed condition is its digest comparison. The fourth edit's exact supplied digest was not retained; we cannot establish which digest it used. It is therefore incorrect to claim the model copied a particular old digest or that a specific adapter repair would resolve the failure. Validation was not weakened and no further request was sent.

The controller halted before its ordinary completed-arm export. A separate read-only terminal exporter preserved the incomplete store, CAS, candidate source, inputs, process log and bounded diagnostics. It explicitly records the absence of a result row, Stop, closure and final receipt. Independent review confirmed the diagnosis and accounting. The rules arm has no intent and remains unattempted.

## Accounting and preservation

- Current v3 attempt: 4 requests, **$0.001098**.
- Cumulative synthetic stage, including the prior failed request:5 requests, **$0.001342** of the original $5 cap.
- Aggregate settled: **$0.231086**; outstanding reservations: **$0**.
- The original stage baseline and 40-request limit were retained; neither was reset.
- At 12:02:45 UTC, the worker/process group was absent, no M3 containers remained, and 291.491 GiB was free.
- All 603 frozen inputs remained unchanged. The prior failed attempt and all previous evidence remain preserved. No provider response body or credential was logged.

The result and export hashes are in `m3-integration-v3-result.json` and `m3-integration-v3-export-sha256.json`; raw preserved artifacts are in `m3-integration-v3-execution/`. The synthetic fixture stays entirely outside qualified-corpus and performance claims.

The precise remaining blocker is completing a valid edit/test sequence with explicit submission and a protected final receipt under the real provider. No further implementation defect is demonstrated by this run. The larger comparison remains unstarted; M3 acceptance remains unmet. No push, merge, publication, deletion or fine-tuning occurred.
