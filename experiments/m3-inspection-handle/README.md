# Inspection-handle experiment: offline validation complete

This separately built experimental implementation preserves the frozen broker, engine and fixed12 evidence. It is not connected to a live provider. No model-reliability improvement or M3 real-corpus completion is established.

## Interface and evidence

`broker_v2.rs` replaces only its corresponding module in the separate source tree created by `materialize.py`; `inspection_receipt.rs` is added beside it. Protocol: `tsr-broker@inspection-handle-v1`. A successful native inspection issues structured `inspection: {handle,path}`. Private native state binds the handle to run, task, inspection call/event, path, full-file digest and workspace source state. Exact edit resolves this reference before the existing full-file digest, unique-span and prewrite checks. Edits/replacements and source-mutating tests invalidate it. Existing containment, policy, scope, budgets and oracle logic remain in place.

Before exposing a handle, the controller persists a bounded16KiB receipt in a Rules-authored Evidence event. Its digest hashes the exact serialized receipt. Receipts contain no raw source, host path or credentials. They identify evidence; they grant no policy authority.

`broker.py` is an offline codec with strict one-action validation. Credential access and provider dispatch refuse execution. `native_check.py` explicitly binds its fake provider to the existing synthetic known-answer task. The wrong-path subcase intentionally bypasses the codec path filter to test the native rejection boundary.

## Results

Seven groups, eight arms:

| Group | Verified result |
|---|---|
| Record repair → test → Stop | Accepted; final oracle receipts verified |
| Rules repair → test → Stop | Accepted; final oracle receipts verified |
| Multiple tool calls | Unavailable before proposal/edit; source unchanged |
| Truncated output | Unavailable before proposal/edit; source unchanged |
| Stale handle after edit | Native rejection; no second write |
| Foreign-arm handle and wrong path | Both native rejections; source unchanged |
| Protected-test alteration | Exact `tamper:changed full-suite test resource tests/answer.rs` closure; rejected before oracle invocation |

The eight final arms used fresh directories under `target/m3-handle-native-v3`. Initial v1/v2 harness attempts stopped before any candidate action because fake-broker initialization/synthetic allowlist wiring was incomplete. Their logs remain saved. No provider calls or paid-ledger changes occurred. Running container IDs were unchanged after each final arm. The200GiB guard remained enforced.

`audit.rs` verifies canonical event identities and parent chains, trusted issuance receipt integrity, referenced inspection digest, submitted-proposal/candidate argument bindings, result roots and final oracle receipts. `finish_audit.py` exports evidence and checks exact negative outcomes. Deliberately changed issuance evidence fails audit. The audit does **not** independently replay source freshness or invalidation; the native negative cases support those properties separately.

## Validation and deliverables

- Offline native engine build: passed (Homebrew Rust1.90; isolated target).
- Receipt unit tests:4passed. Codec tests:5passed.
- Seven native groups/eight arms:passed.
- Eight-arm persisted-evidence audit and corrupted-receipt rejection:passed.
- Rust formatting, Python syntax checks, and separate metadata typecheck:passed.
- Independent review: `final-review.md`.
- Exported evidence: `../../docs/m3-inspection-handle-native-evidence/` (147 hashed files).
- Cleanup proof: `../../docs/m3-fixed12-approved-cleanup/subset14/after.json`.

Logs retain failed manual compiler selection (rustup vs Homebrew metadata; syn feature selection) followed by successful checks. No frozen dependency artifacts were rebuilt or overwritten. The new engine is `target/m3-handle-build/debug/tsr`; the original pinned engine remains unchanged.

A live pilot requires separately reviewed transport/budget integration and explicit authorization. This codec deliberately lacks a live transport envelope. No push, PR, merge or deployment is part of this work.
