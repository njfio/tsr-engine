# Persisted freshness replay and proposed live test

This separate version closes the specific audit gap identified at c555b64. No native handle-validation defect was demonstrated. The original experimental sources, binaries and exports remain unchanged.

The native controller now persists a hashed Rules-authored source checkpoint after each completed non-Stop action: call, candidate identity, action, source state before and after. Checkpoints contain hashes and metadata, not source text. The Rust auditor verifies their event-chain identity and binding to the submitted candidate/request. Only after that canonical audit, `replay.py` independently folds latest inspection and invalidation: edits/replacements always invalidate, source-mutating tests invalidate, unchanged tests preserve, and a new inspection supersedes the prior one. It validates advertised references and independently rejects the three undispatched stale/foreign/wrong-path proposals.

Validation completed:

- Separate offline engine build succeeded.
- Eight native arms across the existing seven groups passed.
- Both successful modes reached independently verified final oracle acceptance.
- Persisted freshness replay passed for all eight arms, including one independently invalid handle each in stale/foreign/wrong-path cases.
- Six fold-unit tests, four full-stream negative fold tests, and five codec tests passed.
- Corrupted issuance evidence remains rejected by canonical audit.
- Independent review: `review.md`.

Replay trusts the controller's recorded before/after state hashes; it does not independently reexecute commands or detect transient changes reverted between snapshots. Python replay alone is not a cryptographic audit; the Rust audit must precede it. No claim of model reliability, repair superiority or TSR gate quality follows from these scripted tests.

Evidence: `docs/m3-inspection-handle-replay-evidence/`; separate binary: `target/m3-handle-replay-build/debug/tsr`. Old c555b64 evidence is retained. No cleanup, Docker restart, unrelated-container changes, paid inference or source transmission occurred in this follow-up.

The prepared live proposal is `docs/m3-handle-live-validation-plan.md`: six matched old/new Record-mode pairs on the frozen11,893-byte reconciler task, up to240calls/2.4Mgenerator tokens/$5additional spend. Cumulative settled spend is reconciled at$0.544194 against the$100aggregate ceiling. The experiment needs new batch-specific approval and a narrowly scoped, offline-verified live wrapper before dispatch; no launch is authorized or claimed ready now.
