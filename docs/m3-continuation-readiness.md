# Three-arm continuation: offline readiness

A separate one-shot controller now implements the narrowly scoped policy in `scripts/m3-continuation-controller.py`. `scripts/m3-continuation-broker.py` is the reviewed content-free diagnostic adapter with only its baseline path changed. No controller, adapter or state from the halted run was altered. No provider calls or credential reads occurred.

## Exact remaining execution scope

| Task | Mode | Allowed implementation file | Visible named test |
|---|---|---|---|
| `0c5a91a6aaa862c279aed1e1602a74eda7c42c4fc0f72247de513fa948f50b70` — ProtocolError::to_wire | rules | `crates/rts-daemon/src/error.rs` | `error::tests::protocol_error_round_trips_to_wire` |
| `47c787ed4f89b141de573feeeaead5a27c6e2ec399a0bccc1abe259b92911c23` — Route::to_wire | record | `crates/rts-mcp/src/roots.rs` | `roots::tests::absolute_path_outside_every_root_is_a_visible_default` |
| Same Route task | rules | Same file | Same test |

These are the three unattempted arms of the progress-v2 comparison, not three new tasks. Both tasks derive from the public `https://github.com/njfio/rs-agent-code-utility` lineage. The existing ProtocolError/record result remains unavailable and is never retried, reclassified or included as a completed pair. Other historical pilots remain separate.

Provider data is limited to the same two task definitions, allowed implementation source and bounded test observations. No unrelated sessions, credentials, private logs, hidden oracle reversal data or additional repository files are added to the provider scope. Network inference would use `https://api.cheaperinference.com/v1/chat/completions`, model `deepseek-v4.1-flash`.

Prepared inputs are in `target/m3-continuation/<full-task-id>/`; there are no new arm output directories or controller state. `docs/m3-continuation-readiness.json` freezes **225 files**, including controller/broker code, evaluator, helpers, exact input CAS/profile/context, and prior evidence. The baseline pins **$0.214469 settled, $0 outstanding**. Proposed limits, awaiting scoped approval: **$10 additional**, **$100 aggregate**, three serial arms at most once, 20 model calls and 1800 seconds per arm, 60 calls total, 6000 seconds global, zero retries. Output caps and original profile authority remain unchanged.

## Implemented lifecycle

Before dispatch, the controller validates frozen hashes, shared-ledger settlement/reservation headroom, idle runtime and disk guards. It writes and syncs a one-shot state directory and per-arm intent before spawning a worker. Existing state refuses every restart, including an interrupted attempt. A full 1800-second allowance must remain within the global deadline; the deadline is checked again after the worker returns.

After every arm, it checks process and process-group exit, container quiescence, unchanged inputs and ledger budgets. Export includes input/output CAS, task/profile/context/preparation, SQLite snapshot, raw result, sanitized diagnostics, process status and ledger. CAS digests, stored event-ID-derived roots, closure, result identity, final-oracle receipts when invoked, and export hashes are verified before a continuation decision is written and synced. Accepted/rejected outcomes must agree exactly with the boolean oracle result and receipt verification. Contradictions, non-oracle rejections and action denials halt.

For an unavailable arm, only the allowlisted `tool_shape` cases or decode-stage invalid argument JSON may continue. Every per-arm diagnostic must bind exactly to a newly reserved and settled request in order; the terminal failure must have no proposal, and preceding calls must be successful. The original unavailable result is retained. Unknown errors, choice-count errors, explicit refusal/truncation, invalid actions/scope, unresolved or duplicate billing, containment/evidence errors and global budget limits halt the cohort. Continuing means selecting the next manifest arm, never retrying or correcting the failed response.

## Offline verification

- `python3 -B scripts/test_m3_continuation.py -v`: **16 passed**. Includes durable-intent ordering, at-most-once/restart refusal, exact scope, pre/post-worker global deadlines, guard/evidence/identity failures, exact settlement binding, call bounds, proposal exclusion, budget headroom, valid negative oracle and contradictory oracle/result rejection.
- Fake-provider integration exercises the actual Python adapter encoder, mocked credential and HTTP entry points, billing, strict decoder, persisted metadata and controller decisions. Missing calls, wrong name, multiple calls and malformed argument JSON preserve unavailable and advance once; invalid action, refusal and truncation stop. Source/credential canaries are absent from every persisted test artifact. These are deterministic synthetic fixtures, not live outcome evidence.
- The production export path rereads a temporary copy of preserved evidence and verifies it, then rejects intentionally corrupted CAS. A synthetic checked negative-oracle case and contradictory-result case isolate the outcome binding rule. The contradiction regression failed before the fix; output is retained in `m3-validation/m3-continuation-review-04-before.log`.
- `/Users/n/.cargo/bin/cargo +1.90.0 test -p tsr-batch broker_v2::tests --offline`: **13 passed**, including missing forecasts/unknown fields, approved test IDs, replay/stale replies, traversal/symlinks, progress freshness and bounded deadlines. This separately tests evaluator action validation; the fake-provider lifecycle test does not claim to execute Docker or the complete Rust evaluator subprocess chain.

No paid inference or real candidate workload was used for verification. Rust tests use bounded local fixtures and fake runners. Existing evidence remains byte-for-byte preserved.

## Review and limitations before live execution

Independent review identified directory-sync durability, global deadline headroom, preservation of input evidence and exact outcome/oracle consistency. All were addressed. Final review status and the manifest hash are recorded separately.

The new lifecycle has not run live. Runtime availability, pinned image, free disk, provider price bounds and frozen identities must be checked immediately before any approved launch. The existing stored-root audit hashes event IDs in order; it does not independently recalculate every canonical event payload ID or prove candidate semantics. Receipt checks reuse the pinned reviewed verifier. A filesystem or process failure intentionally leaves the new state non-resumable for review. No cleanup or hidden retry is attempted.

A future launch requires a separate approval file bound to the exact readiness-manifest SHA256, the exact three task/mode pairs, and the $10/$100 caps; none has been created. The controller's `--approval` entry point refuses mismatched scope. The parent can use this report and frozen manifest to obtain bounded execution approval. M3 operational acceptance is still unmet; this readiness work does not supply additional real paired outcomes.
