# Malformed public-history diagnostics

## Reproduction and reachability

At a0220f1b, `BrowserProjection.test_protected_stale_cross_scope_history_and_size_fail` injects exactly `inputs.public_desires = {"private":"not admitted"}` into an authored request. The public export guard used `.get('history', [])`, so this malformed object appeared empty there. `public_history` subsequently indexed `['history']` and raised `KeyError`. Reproduction failed as expected; `/tmp/tsr-history-before.log` retains the traceback. No history was accepted, mutated, or sent to a provider.

This is an actual Python boundary diagnostic defect, not evidence of a corrupt ordinary native ledger. `crates/tsr-cli/src/engine/adaptive/coding/project/desire.rs::project_public_desires` (lines 53–159) reconstructs typed records, checks bridge source and lineage, returns `None` when history is empty, and otherwise constructs `active`, `current`, `history`, and `authority`. `relational.rs` inserts `public_desires` only for `Some`. Legacy/no-development requests omit it. No valid persisted/native record producing the injected missing-history shape was identified. A malformed injected/replayed adapter input can nevertheless reach these Python entry points. Previously it stopped with an incidental exception; it now stops with a deterministic diagnostic.

In the ordinary adapter, `verify_history` runs before `web.project` and before generation. It had the same unguarded envelope access. Restart does not repair or consume this malformed input: it must fail closed until authentic valid input exists. The change does not add a recovery disposition or clear any journal.

## Narrow correction

`engine-browser-desire.py::history_snapshot` is a shared structural check. Absent `public_desires` retains legacy behavior. A present object must have `history` and `active` arrays whose entries are objects; present null, empty/missing fields, scalars and malformed row types fail with `ValueError('public meaning history: …')`. It neither fills missing fields nor discards records. `public_history`, `verify_history`, and the public export guard use it before accessing history.

This is not a replacement semantic validator. Existing exact export hashes, owned admission/import pins, active-parent equality, original obligations, lineage and content checks remain. Native status records and richer envelope fields are not normalized. Unsupported or unowned records remain rejected by existing provenance checks. Review-note digests retain their existing export policy; no new data category is exported. The source-bound choice-summary implementation is unchanged.

## Verification

- Focused projection/reference/criterion suite: 34 tests, 33 passed and one opt-in skip, 0.072s. The previously failing broader projection test now passes. Negative cases include the exact injected object, absent arrays, null/scalar envelopes, invalid array types and invalid row types. Positive cases cover absent legacy shape, current envelope, unchanged hash-admitted history projection, and refusal of unowned history. Inputs remain byte-equivalent.
- The one skip is `test_actual_native_serialization`, requiring `TSR_BROWSER_AUTHORED_REQUEST` pointing to a two-file native request with an actual browser return. This shape-error repair does not require new browser execution. Dedicated actual native generation/review/restart and meaning-admission/import tests run separately using authored transport; final results below.
- No paid calls, live admissions, historical record edits, game changes, or altered permissions. Tests use disposable native projects and authored responses, not live model evidence.
- Independent read-only review found no blocker: legacy omission preserved, malformed present envelope rejected, native shape compatible, ownership/active-parent checks unchanged, no additional export. This is a diagnostic consistency repair, not a demonstrated paid-dispatch bypass fix.
- `git diff --check` passed. Frozen v4 packet remains `a9fe6d8325540172272f89e7891791f209ec2c4c82390ea2596d413a57d58323`; launch remains `50ee277dbcf0b5771b13d5c2d882b8fed1d27b63bea265aa990b14787d358984`. No closed trial was replayed.
- Actual native integration: **3 passed, no skips, 96.508s** with `TSR_BROWSER_ADAPTER_AUTHORED=1 PYTHONPATH=scripts python3 -m unittest scripts.test_engine_browser_adapter.BrowserAdapter.test_native_predispatch_disposition_restart_same_body scripts.test_engine_browser_adapter.BrowserAdapter.test_serialized_review_representation_after_correction_and_restart scripts.test_engine_browser_desire.BrowserDesire.test_fresh_admission_import_revision_and_restart`. This covers exact-body pre-dispatch recovery, independent review representation/replay, native meaning admission, imported history, later revision and restart. All transport responses were authored fixtures.

Conclusion: the concrete exception is closed and native compatibility verified. There is no remaining blocker for this offline repair. The skipped optional browser-return fixture limits claims about that separate browser-observation projection; no browser behavior was changed or asserted here. No new experiment is proposed.
