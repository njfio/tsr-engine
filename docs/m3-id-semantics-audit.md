# Proposal ID semantics audit

`/arguments/id` is a test-command selector, not bookkeeping. Replacing it with a generated request/session ID would change which command is selected and fail the allowlist. No adapter or engine behavior was changed, and no paid request was made.

## Trace and ownership

| Field | Ownership and use |
| --- | --- |
| Provider `/arguments/id` | For `test`, must equal `named-failing-test` in this scoped adapter. It becomes `Proposal::Test { id }`. For inspect/edit/stop, it must be `""` and is discarded during normalization. |
| Rust `Request.id` | Controller-derived `Hash::of(protocol, current ledger root, call index)`; not model-generated. |
| Rust `Reply.request` | Set by the trusted Python adapter from the current request ID; Rust requires exact equality before accepting a proposal. |
| `Candidate.id` | Controller-derived content identity over action kind, serialized normalized proposal digest, and parent events. The test selector is therefore committed into candidate provenance. |
| Billing reservation ID | Separate controller-generated UUID in the transport ledger, unrelated to the command selector. |

Production use sites:

1. `scripts/m3-edit-v2-broker.py:160` validates the exact test selector. Its return normalizes only active fields and supplies `reply.request` from `request['id']`.
2. `crates/tsr-batch/src/broker_v2.rs:48` defines `Proposal::Test { id }`; request correlation is checked at line 297.
3. Lines 344–347 translate the selector into `StepArgs::TestId`. `crates/tsr-batch/src/driver.rs:60` passes it unchanged to `ActionArgs::TestId`.
4. `crates/tsr-policy/src/profile.rs:137` selects a concrete trusted command by ID. Profile validation rejects empty and duplicate IDs, so the selector is unambiguous within the profile. It is intentionally reusable across test executions, not unique per request/session.
5. `crates/tsr-policy/src/gate.rs:373` requires membership in both the profile and capability grant. The broker driver also refuses unknown test IDs in both experimental modes (`broker_v2.rs:363`).
6. `broker_v2.rs:416` serializes the normalized proposal for candidate evidence. `crates/tsr-domain/src/lib.rs:235` derives candidate identity from kind, arguments and causal parents.
7. `broker_v2.rs:521` resolves the selector to argv, checks whether it is the required failing test, dispatches the candidate, and captures execution evidence and bounded feedback containing the selected test ID. Tool and wall budgets remain independent controller checks.

The request hash binds a reply to the current protocol/root/call tuple; the adapter already supplies it without model generation. This formula is not an explicit globally unique session nonce. Identical tuples reproduce an ID by design. The existing Rust test `replay_exhaustion_and_stale_reply_do_not_execute` checks that a mismatched reply causes no execution. No new ID algorithm was introduced, so collision/replay testing for a proposed metadata replacement is not applicable.

## What the failed response establishes

The saved diagnostic establishes only that `/arguments/id` was absent. It does not retain the selected action. It cannot show whether the omitted ID was semantic (`test`) or inactive (inspect/edit/stop). The known schema/instructions require it in every variant. Do not infer an inspect action merely because it was the first request.

Independent review confirmed this trace and conclusion. Thirteen offline cases passed through the current decoder: valid selector normalization, missing-field rejection in all four actions, rejection of empty/unknown/request-hash test selectors, and separation of trusted request correlation from the constant test selector. Output is in `m3-validation/m3-id-semantics-check.log`. No provider or candidate execution occurred during this audit.

## Smallest justified alternative

A new action-specific wire contract could remove redundant inactive fields while preserving semantic requirements:

- Inspect: `action`, `path`, `forecast`.
- Test: `action`, **`test_id`**, `forecast`; require the exact allowlisted selector.
- Edit: `action`, `path`, `expected_digest`, `old_text`, `new_text`, `forecast`.
- Stop: `action`, `forecast`.

Reject unknown/inactive fields rather than accepting or silently defaulting them. Normalize `test_id` into the unchanged Rust `Proposal::Test { id }`. Keep request correlation host-owned, all action/path/edit/forecast checks, protected evidence and budgets unchanged. A missing `test_id` on a test action must remain a failure, even when only one command is allowlisted.

This is a proposed interface simplification, not the requested bookkeeping fix and not proof it would have repaired the historical response. Per the instruction to report rather than change essential semantic content, it was not implemented or launched. It would require schema/decoder parity tests, supplied/missing-selector and replay tests, the full-path fixture, independent review and a distinct freeze before live use. The cumulative synthetic stage still has one request and $0.000244 spent; aggregate remains $0.229988 with zero outstanding. The larger comparison remains pending.
