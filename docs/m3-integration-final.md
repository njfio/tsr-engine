# Consolidated integration stage: blocked at provider contract

The first live synthetic request failed before any candidate action. The strict adapter reported `proposal_fields`: `/arguments/id` was missing, with expected type `string`. The response contained one `propose` call and parseable JSON, but did not satisfy the required-fields contract. This is a demonstrated response-contract violation; the retained evidence does not establish its upstream cause.

No decoder/schema mismatch was found: the emitted schema requires all seven fields, the instructions explicitly prohibit omission, and examples include every field. Independent review confirmed this assessment. Under the approved rule to stop when no repairable integration defect is demonstrated, **no second live request was made**. Validation was not weakened. The rules arm has no intent and was not attempted. The 12-task comparison has not been frozen or launched because the successful live integration prerequisite remains unmet.

## Scope and implementation

The approved stage uses DeepSeek v4.1 Flash at `api.cheaperinference.com`, only synthetic material and bounded feedback, with a $5 incremental cap within $100 aggregate, 40 total requests and 1800 seconds per attempt. The stage baseline was $0.229744 settled. No unrelated private data, credentials, qualified source, or hidden reference diff was sent as model input.

The isolated task `b7b1fee58ff04e6456d81b023cc8754aac6602069743a7d6b5125d3fc1aeadf3` has a dependency-free Rust function documented to return 42, initially returning 0. Tests reside in a protected external file. Its Git source, controller-only reference diff, CAS, preparation identity, profile and empty initial state are separate from the qualified corpus. No synthetic task was added to the 151 qualified tasks.

New files provide the scoped broker, preparation generator, bounded controller, local preflight and terminal exporter. The production engine, strict exact-edit adapter, protected-test grading and receipt verifier are unchanged. The prelaunch freeze is `5ced86a5542c46bd2606bb16493560240f63adbd3fb338b071935d7ff1c5822f`; all 587 pinned inputs remained byte-identical through the attempt. The initial pre-review manifest is also preserved.

## Local checks versus live outcome

The first scripted preflight used a raw file hash where exact edits require the inspection's length-prefixed digest. It failed with `stale or oversized edit source`; its state remains in `target/m3-integration-local-check`. The scripted fixture was corrected to copy the emitted inspection digest. Independent review also found and corrected the initial disk threshold to use 240 GiB rather than the 200 GiB ongoing guard, before live launch.

The fresh scripted run in `target/m3-integration-local-check-v2` completed the real networkless worker path in rules mode: inspect → visibly failing test → exact edit → visibly passing test → Stop → protected final oracle. Exported immutable events end with `oracle:true`; the independent receipt checker reread CAS and verified manifest `489c3fff356b04a237d846fcc001968cb7cb565324b39192597fbe3bab6f1494`. This proves the local sealed pipeline works with valid proposals, not that the real provider path succeeds.

Six adapter tests passed, including the 266-case matrix. The new broker accepted the exact synthetic scope and rejected four deviations (task, path, test, request limit). Independent review confirmed both preflight fixes, all 587 hashes and local receipt evidence before the live launch. No full Rust rebuild was needed; the exact frozen executable was used.

The live record worker PID 66238 ran from 2026-10-17 11:50:20.666001 UTC to 11:50:30.808368 UTC, exited 0 with no containment errors, and recorded an unavailable outcome. One settled completion was rejected at decode with this content-free detail:

```json
{"version":1,"path":"/arguments/id","expected_type":"string","actual_type":"missing","reason":"required"}
```

It produced zero proposals, edits, tests, submissions or final-oracle invocations. The controller preserved the unavailable export and halted on missing explicit submission. The result is neither an accepted task nor a successful live integration.

## Accounting and terminal evidence

- Requests: 1 of the 40 maximum; 1578 prompt and 58 completion tokens.
- Incremental charge: **$0.000244**.
- Aggregate settled: **$0.229988**.
- Outstanding reservations: **$0**.
- At 11:52:14 UTC, the worker and process group were absent, no M3 containers remained, and free space was 291.593 GiB.
- `m3-integration-result.json` contains the reconciled outcome and diagnostic. `m3-integration-execution/` preserves 35 evidence files, covered by `m3-integration-export-sha256.json`.
- `m3-integration-local-evidence/` contains the separate scripted-run result, immutable store, CAS and independently verified receipt.

The provider's [parameter documentation](https://www.cheaperinference.com/docs#parameters) says tool/structured-output support and interpretation vary by model and serving provider. It does not establish `oneOf` or strict schema enforcement for this route. The [OpenAPI document](https://www.cheaperinference.com/api/openapi.json) describes optional model-lock and technique-disabling headers; this frozen transport did not set them. That is a future comparison-control concern, not evidence that routing or optimization caused this missing-field failure, and it was not used as a justification to repeat the request.

M3 remains incomplete. The remaining blocker is demonstrated reliable model-to-strict-adapter behavior through final receipt, before spending on the fixed comparison. No push, merge, publication, deletion, fine-tuning, or extra repository scope occurred.
