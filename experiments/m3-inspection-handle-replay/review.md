# Independent persisted-freshness replay review

Reviewed 2026-10-18. No containers, candidates, inference, source transmission, or implementation edits were performed. Only this review document was written.

**The previously identified persisted freshness/invalidation replay gap is addressed for the eight saved synthetic fixtures.** Native checkpoints record call, candidate, action, before-state and after-state in hashed Rules-sourced events. The Rust auditor verifies canonical event identities and parents, checkpoint/proposal/call bindings, issuance receipts, prior inspection digest references, candidate arguments and available final-oracle receipts. The subsequent Python fold independently derives latest issuance and invalidation from actions/state transitions, rather than consuming the driver's validity flag.

All 146 files in the exported hash manifest match. Record and Rules each show four source checkpoints, one fresh edit, and successful verified oracle evidence. Stale, foreign and wrong-path cases each show one independently invalid undispatched handle. Multiple/truncation have no issuance or edit. Protected-test mutation has the exact protected-resource rejection and no final-oracle invocation. This is consistent with the intended eight-arm matrix.

No concrete fail-open defect was found in this bounded source review. Important scope limits remain:

- Run the canonical Rust audit before the Python replay; `replay.py` alone does not verify cryptographic event identities or authenticity.
- Replay trusts controller-observed bounded source-state hashes. It checks recorded freshness semantics, not independent filesystem reconstruction or candidate-command reexecution. Transient changes that return to the same before/after state are not detected by those snapshots.
- The six unit tests exercise the Freshness fold primitives, including supersession, edit/revert and mutating tests. They are not a comprehensive full-event-stream corruption suite. The native evidence and canonical auditor supply separate integration support.
- The offline native harness uses scripted proposals and disabled provider access. Its success establishes neither provider contract compliance nor improved model repair yield.

The proposed live-plan file was not present at review time. Six repeated old/new pairs on one previously exposed reconciler task could measure that task's bounded interface reliability, but are not six independent repository/task samples and cannot establish general repair superiority or gate quality. Any comparison should disclose the entire changed interface/prompt/checkpoint treatment, preserve all twelve assignments and stopping causes, and avoid describing a small stochastic experiment as decisively establishing general reliability. No live execution or source transmission is authorized by this review.

## Final plan follow-up

Reviewed `docs/m3-handle-live-validation-plan.md`. The plan explicitly withholds launch authorization and source transmission, fixes twelve assignments/six counterbalanced pairs on one previously exposed task, retains all failures/missing slots, and separates an operational go/no-go from general reliability or TSR superiority. The arithmetic is consistent: twelve times 20 calls / 200,000 tokens / 1,800 seconds gives 240 calls / 2.4 million tokens / six hours of nominal arm budgets; $0.544194 plus $5 gives $5.544194, including unresolved reservations. Ordinary failures may advance only within the fixed schedule; uncertainty and integrity failures halt globally.

No planning blocker found. Before launch, the six-hour global ceiling must include setup/export time and require sufficient headroom before dispatch; twelve maximum-duration arms cannot all fit plus overhead, so censoring remains possible. The described outbound redaction/credential filtering, task-scoped transport wrapper, common instructions, paired controller, immutable freeze and explicit approval remain prerequisites, not completed capabilities. The new engine/checkpoint/interface treatment is broader than replacing one token field; the plan appropriately avoids a pure-field causal claim. The multi-edit opportunity criterion also correctly allows an inconclusive result rather than forcing extra edits.

Four additional full-stream negative replay fixtures are reported passing (stale edit, missing checkpoint, request-state mismatch and old advertised handle). They test the fold separately from canonical verification; they supplement rather than replace the ordered Rust-audit-then-replay requirement. No tests were rerun by this reviewer.
