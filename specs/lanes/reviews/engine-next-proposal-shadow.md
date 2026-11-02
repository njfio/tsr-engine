# Independent review: synthetic next-proposal shadow preparation

Reviewed working implementation from `2fb165d`, 2026-10-26. No provider calls,
target execution or output adoption by this reviewer. The frozen plan was read,
not edited.

## Scope and experiment interpretation

Both bodies derive from one native rejected fixture state. Their only experimental
difference is `current_guidance`; source, parent meanings, original aspiration,
semantic evidence, attention, lineage IDs, settings and response contracts are shared.
The projection checks exact evaluator-authored hypothesis/development text, excludes
the developed proposal from shared history and does not serialize private acceptance
prose. The authored Boolean fixture and Fake outcomes are explicitly distinguished
from executed B9 parser evidence and genuine model-generated culmination.

The semantic rubric permits useful identical proposals, uncertainty or inability;
it does not score wording overlap or require a preferred assertion. Two outputs
cannot establish reliable benefit. Fixed order and possible reviewer unblinding
remain disclosed limitations. This is a narrow composition probe, not an autonomous
learning-cycle demonstration or a new empirical benchmark claim.

## Native and transport boundaries

`validate_next` is a distinct cfg(test) mode. The historical validator remains
revision-only. New responses are parsed into native types and passed to
`evaluate_proposal(false)`, with state-root invariance checked. Exact edits remain
source/inspection-bound; experiments retain native path, identity and mapping
validation. Structural success is not semantic correctness or permission to run.
No generated proposal is applied, executed or admitted.

The profile reuses the existing durable accounting loop: an exclusive journal,
intent before dispatch, measured usage/charge records, terminal stop on unknown or
invalid outcomes, and no automatic redispatch. Manifest checks bind implementation,
worker, binary, rubric, state, directory, model/settings and both bodies. Runway is
rechecked after native controls. `live_authorized:false` plus explicit opt-in keeps
offline preparation from granting live authority.

No blocking source-level finding identified. The final focused changes address the
coverage recommendations: a formatting-only authored exact edit reaches native
preview and is eligible but unexecuted; stale inspection and source digests are
rejected; an explicit inability passes through the full accounting loop before the
second independent arm. That inability is retained as no proposal and does not
become an execution or adoption. Export now explicitly validates both source and
inspection identities as digests. These additions introduce no winning behavioral
assertion or target execution. Their test commands were still running when reviewed;
source review is not a terminal test result.

Tests and aggregate results are reported separately when terminal; no running or
uninspected suite is claimed to pass here. Existing v1 external guard and prior
trial records remain outside this change.

The inspected aggregate Rust log (`target/next-proposal-cli-tests.log`) reports
234 passed, 1 failed and 8 ignored. The failure is the existing local verifier stub:
a complete, metered response was quarantined at 2112 ms against its 2000 ms
admission. Preserve this aggregate failure and any later isolated result separately;
do not weaken the deadline or call the whole suite clean from an isolated rerun.

## Verification follow-up

The five final native profile tests passed, including exact edits, stale identities,
inability, fresh-process drift and unknown reservation controls. The unchanged
shared-accounting profile passed all eleven native tests. The aggregate Rust
result remains 234 passed, one failed, eight ignored; its single failed verifier
stub passed once in isolation (1.45 seconds). This does not erase the aggregate
deadline failure or establish its scheduling cause. No deadline/assertion changed.

Full Python aggregate subsequently finished: 663 tests, 623 passed and 40 skipped.
The sixteen opt-in native tests above were separately exercised successfully.
Fresh-process preparation/preflight/inspection confirmed only current guidance
differs, zero dispatch history and an unauthorized live manifest.

## Independent deadline diagnosis (2026-10-26)

Read-only inspection of `acceptance/reviewer.rs`, `reviewer/local.rs` and
`reviewer/local_tests.rs` confirms that the 2000 ms admission covers the entire
runner invocation. It includes freshness checks, resource reservation, durable
progress, both model-identity checks, response transport, settlement and terminal
persistence. The provider's reported 1 ms duration is not this wall-time budget.
The outer reviewer correctly quarantines a complete response whose whole invocation
exceeds the admission. The inner HTTP deadline starts after initial checks; it does
not override that outer admission check. No unsafe acceptance or cancellation race
was demonstrated by the historical 2112 ms failure.

Parent-run temporary cfg(test) instrumentation was inspected independently. The
instrumented aggregate passed 235 tests, with 8 ignored, in 55.70 seconds
(`target/verifier-deadline-diagnostic-aggregate.log`). An isolated successful
invocation reached terminal persistence in about 465 ms. Aggregate success-path
phase measurements reached roughly 1766 ms, with time distributed across host-side
checks, persistence and transport phases. Four concurrent exact-test controls and
an isolated control all exited successfully (`target/verifier-deadline-comparison.json`).
These measurements support sensitivity to concurrent host work but do not identify
the precise cause of the earlier 2112 ms event. They do not reproduce a deterministic
implementation or synchronization defect. A passing diagnostic run does not erase
the original aggregate failure; temporary instrumentation also changes timing.
No production fix, deadline increase or assertion weakening is justified by this
evidence. Final uninstrumented aggregate verification remains separately reportable.

The proposed next-proposal comparison uses the Python external transport and its
shared `run_profile` accounting, not this local-verifier HTTP runner. Its native
preview performs validation without executing a verifier. This failure is therefore
a disclosed aggregate reliability limitation, not evidence that the shadow request
projection or its transport is defective. It grants no live comparison authority.

Final uninstrumented default-parallel aggregate completed: 235 passed, zero failed,
eight ignored, 55.44 seconds. Original failure remains retained separately.
See `docs/engine-verifier-deadline-diagnosis.md` for phase timings and limits.
