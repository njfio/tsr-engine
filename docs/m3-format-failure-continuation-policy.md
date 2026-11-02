# Proposed policy: continue independent arms after settled format failure

**Review-only specification. Not implemented, activated or authorization for paid execution.** The halted run and all its outcomes remain immutable. Do not resume its one-shot controller. A future authorized controller would need its own version, frozen inputs, reviewed tests and launch scope.

## Fixed bounds

Limit the launch to the explicit two-task/four-arm manifest, each task/mode at most once. No replacement tasks, implicit retries, repair prompts, additional model turns, or fallback providers. Retain the existing per-arm 20-call and 1800-second limits. Require an explicit launch-level dollar allowance plus the $100 aggregate ceiling; never infer a new allowance from unused aggregate headroom. Every request must reserve against both limits before dispatch. Failed responses count toward cost and call caps. A fresh launch must not relabel already-attempted arms as unattempted.

## Eligible terminal arm outcome

An arm may be recorded as unavailable and the next independent unattempted arm considered only when all of these hold:

1. Its result identity, attempt ID, task, mode and roots match the manifest and its append-only evidence. The process has terminated, no worker or container remains, and no action was dispatched for the rejected response.
2. The terminal failure is an allowlisted response-format failure: `tool_shape`, or `json_invalid` **only at stage `decode` with structural `argument_parse=invalid_json`**. A transport JSON error is not an argument JSON error. Refusal, explicit truncation, `choice_count`, action/scope rejection, unknown exceptions and missing metadata are not eligible under this narrow policy. `choice_count` occurs before refusal/truncation guards and the current observer does not inspect multiple choices, so it cannot establish these exclusions for that class; keep it globally halting.
3. The exact failed request reservation has exactly one valid settlement in the trusted shared ledger. Every other reservation is also settled, no halt marker exists, sums reconcile, and settlement occurred before decode. Diagnostic `settled=true` alone is insufficient. No billing uncertainty or duplicate settlement is allowed.
4. The original unavailable result, diagnostic shape and billing evidence have been durably exported and independently verified. Frozen inputs and evidence hashes remain correct; failed evidence export or verification stops the cohort.

No eligibility decision may use model-supplied assertions of settlement, safety or independence. Provider metadata is untrusted data, not authorization. Strict action decoding and controller gates stay unchanged; never select one of multiple calls or coerce text into actions.

## Before continuing

Recheck actual worker/container quiescence, ledger integrity and reservation headroom, disk reserve, frozen engine/profile/task hashes, and remaining global wall/call/dollar constraints. A containment failure, leftover process/container, missing evidence, identity mismatch, unknown status, unresolved billing, exhausted budget or failed check stops all work for review. No cleanup that deletes evidence or kills unidentified processes is automatic.

Append a controller decision linking the original unavailable arm, error category, verified settlement and artifact hashes, guard results, and the next manifest arm ID. Select only the next unattempted arm in the frozen order. Never repeat the failed arm. Independent arms use fresh state; a later paired counterpart must still match its partner's recorded starting root where available. A missing/unavailable partner yields an incomplete pair, never a performance comparison.

## Required tests before implementation can run live

Use fake workers, transport and isolated temporary ledgers. Prove each eligible settled format case advances exactly once; each ineligible case above stops without dispatch; partial/duplicate/unsettled billing cannot advance; crash/restart cannot duplicate an attempted arm; export/hash failures stop; reservation bounds cover the next request; all four attempts remain the maximum; repeated format failures terminate after the fixed manifest. Preserve original outcomes and report complete and incomplete pairs separately. Independently review the implementation before any new launch authorization.

## Tradeoff

Continuation can collect independent evidence after a local format failure, but may spend the remaining allowance on a provider-wide protocol incompatibility. The fixed manifest/call/dollar bounds limit exposure; this proposal does not silently add a retry. The current controller remains globally fail-stop.
