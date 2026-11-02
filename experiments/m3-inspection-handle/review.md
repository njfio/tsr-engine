# Independent source review: inspection-handle experiment

Reviewed 2026-10-18 against `crates/tsr-batch/src/broker_v2.rs`. Read-only implementation review; only this report was written. No compilation, candidate execution, provider call, Docker operation, or cleanup was performed.

## Assessment

No demonstrated fail-open defect was found in the proposed native resolution path. A handle is derived from a run identity incorporating canonical copy path, task, process/time/sequence, inspection call, path, and full-file digest. Resolution uses native state only, requires the latest retained handle, matching path and matching bounded source state, then applies the existing full-file digest, unique exact-span, size, and immediately-before-write checks. Edits/replacements clear the retained handle; tests that change source clear it. Wrong handles are not silently replaced with current digests. The original submitted proposal is serialized before resolution and remains the candidate hash input. Existing policy and final-oracle paths remain intact in the diff.

The Python codec forwards the supplied handle unchanged, uses action-specific fields, and preserves strict one-call/refusal/truncation/type checks. Its provider-dispatch, credential and request functions explicitly refuse execution. The five wire tests cover useful codec boundaries but do not validate native binding or mutation safety.

## Outstanding evidence and readiness requirements

1. **Initial finding, superseded by the receipt follow-up below: handle issuance was transient.** The inspection ledger observation still records the file digest, while the issued handle/path and run binding live only in native memory/request objects. Saved candidate hashes alone do not independently establish the handle-to-inspection mapping after the run. Before making replay/provenance claims, persist a bounded trusted issuance receipt/event connecting handle, run/task/path, inspection identity, full-file digest and source state, or explicitly document the missing independent verification. Never reconstruct this authority from observation text.
2. **Native behavior is uncompiled and untested.** Rustfmt parsing is not a typecheck. Before execution readiness, require separate compilation plus native cases for valid edit in both modes; null/missing/forged/cross-run/cross-task/wrong-path handle refusal; reinspection supersession; edit and test-mutation invalidation; exact-span ambiguity; source change before application; and protected-oracle rejection. Check candidate hashing against the submitted handle, not the resolved digest. No current test demonstrates these properties dynamically.
3. **This is an offline codec, not a live adapter.** Its encoder is deliberately minimal and has no model/stream/output-limit transport envelope. Do not connect it to a paid dispatcher without a separately reviewed transport/budget integration and pinned native version. The module's inherited `exact-edit@v3` header is stale descriptive text; actual protocol is `tsr-broker@inspection-handle-v1`.

No claim of improved model reliability or repaired historical defect is supported yet. Resource availability and Docker open-reference issues remain parent-managed prerequisites; this review authorizes neither a restart nor deletion. The frozen implementation remains unchanged.


## Follow-up: persistent issuance receipt

The new `inspection_receipt.rs` and broker integration address the missing persisted binding in the source design. The receipt includes run, task, call, path, full-file digest, bounded-source state, inspection event ID, and handle; all binding fields participate in handle derivation. Its exact serialized evidence source is hashed, size-limited to 16 KiB, and decoded with unknown-field rejection. Native `ctx.observe` appends it as a Rules-sourced observation **before** assigning the public handle for the next broker request. The preceding inspection event separately records the inspected file digest. No source content or canonical host copy path is placed in this receipt.

`Receipt::verify` checks the receipt's local integrity and binding, not the authenticity or correctness of its asserted facts. A complete post-run verifier must still verify the containing event/chain and trusted origin, task/run association, referenced preceding inspection event and matching file digest, issuance order, and the submitted proposal's handle. It must not accept a freshly re-sealed arbitrary receipt as independent proof. The function's comment explicitly requires containing-event verification. Thus the previous persistence gap is addressed in code, but end-to-end audit reconstruction is **not yet validated**.

The saved lightweight Rust log records four passing receipt unit tests (roundtrip, each binding field, altered evidence/handle, bounds/unknown fields). Those tests do not compile or execute the integrated driver and do not replace the pending native binding/invalidation/oracle cases. No full-native readiness claim is warranted.

Read-only cleanup diagnosis summary: 11,825 references are attributed to PID 90000, all DIR/read access; 3,359 recorded paths exist. No overlapping mounts were reported among 65 containers of any state, and four unrelated containers remain running. These observations do not establish that restarting Docker is harmless or authorize acting on unrelated services. This reviewer did not independently rerun lsof/Docker inspection, restart anything, or delete files. The runtime/resource blocker remains with the parent pending authorization and fresh checks.
