# Proposed fixed live validation: six matched interface pairs

Status: prepared for decision; NOT authorized to launch. No source has been sent and no paid call made for this plan.

Financial baseline: cumulative settled spend **$0.544194**; existing aggregate ceiling **$100**. Proposed additional hard ceiling **$5**, including outstanding reservations and uncertain charges. This cohort must stop before cumulative spend/reservations exceed **$5.544194** (and never exceed$100). The previous approval covered the terminal24-run comparison and does not authorize this cohort.

## Question and task

Can a real model reliably complete the native inspect/test/edit/reinspect/Stop interface on a previously problematic real source task, and is the new interface operationally better than the old interface on matched attempts?

Use exactly one frozen constructed mutation task from `njfio/rs-agent-code-utility`: `8eb7db184391a66ae727fdfeca4197e7274592687274549d722f3a47a590feca`, `crates/rts-daemon/src/reconciler.rs`. The initial mutated source is11,893bytes, SHA256 `5e49fb5cd5e47230f8e4c28aee0cebdf5e1f4156ffc3adf3d8d8eccb9e043c07`. Reconstructed locally from the frozen base and mutation patch and matched to the prior freeze; exact metadata in `m3-handle-live-payload-inventory.json`.

Why this task: its old Rules attempt stopped on an edit precondition after an earlier edit; its old Record attempt reached the oracle but regressed the full suite. This exercises realistic source length, repair and evidence handling. The74-byte synthetic function is useful for safety checks but cannot represent that operational failure. This task is previously exposed, not a holdout or historical bug-fix task. Do not disclose reference/reversal patches or oracle-only material.

## Fixed allocation and bounds

- Twelve fresh attempts: six old-interface / new-interface pairs, all in **Record mode**. This isolates interface reliability before testing TSR gating.
- Order fixed before launch: O,N / N,O / O,N / N,O / O,N / N,O. Each attempt starts from the same frozen mutant with empty model context. No trajectory reuse, repair retries, automatic replacements, task substitutions or adaptive prompt edits.
- Proposed provider: **CheaperInference**, endpoint `https://api.cheaperinference.com/v1/chat/completions`; model identifier **`deepseek-v4.1-flash`**, matching the prior comparison. Availability, pricing and routing must be revalidated by the existing provider preflight after authorization; these are not claims about current provider availability.
- Per attempt:20model calls,200,000 billed generator tokens,16,384maximum output tokens per call,15tool-minutes,1,800seconds wall time,131,072patch bytes. Cohort ceilings:240calls,2,400,000generator tokens,6hours wall time, and$5additional cost/reservations. No retries after uncertain billing. The6hour cohort clock includes preparation, oracle and audit overhead; require sufficient remaining wall-time headroom before each dispatch, so the ceiling may censor later slots rather than guaranteeing twelve maximum-duration runs.
- Preserve the conservative pre-dispatch input-byte/token bound, strict single-call schema, refusal/truncation rejection, native exact-edit/path checks, sealed networkless worker, independent oracle and200GiB live disk floor. Require240GiB at initial launch; there is currently enough headroom, but recheck then. No cleanup or Docker restart is included.
- Old arm uses the pinned digest engine/codec; new arm uses the checkpointed handle engine and reviewed live wrapper. Use a shared instruction template except the digest/handle-specific instructions and schema, with identical model settings, output allowance, observations and named-test configuration. Freeze final serialized sample bodies and all executable/script/profile hashes before any dispatch. Do not claim a pure causal effect of a token field if prompts/routing differ.

## Exact proposed outbound inventory

Only these allowlisted task-local data may reach the endpoint:

1. The above source file as observed from the fresh mutant and the model's subsequent edits. Initially11,893bytes; native inspection cap65,536bytes and displayed source excerpt cap57,344bytes. Never send another repository file. The file includes its existing embedded test text; that text is part of this explicitly inventoried source file.
2. Task ID; two failing-test identifiers: `reconciler::tests::throttle_passes_through_under_cap` and `reconciler::tests::throttle_sleeps_when_bucket_drained`; approved selector `named-failing-test`, selecting only the first named test for visible feedback.
3. Bounded visible stdout/stderr (up to2,048bytes each), exit status, duration/budget counters and output digests. Redact host absolute paths and reject credential-shaped material before outbound serialization. These checks apply equally to both arms; never send unrestricted logs.
4. Native request ID, state root/version, source-state digest, freshness flags, remaining calls/tokens/time, allowed path/test lists and submission status. New arm additionally sends the current opaque inspection handle and allowlisted relative path; old arm exposes the inspected full-file digest. No host path or run nonce preimage.
5. Fixed reviewed system instructions, action schema and model settings. Model responses remain bounded to inspect/test/edit/Stop proposals plus forecasts; no arbitrary shell tools.

Excluded: hidden mutation/reversal/reference diff, protected oracle workspace/tests outside the inventoried file, private session histories, unrelated logs/files, credentials, environment values and network tools. Authentication uses the existing credential only as the authorization header; no credential creation or model-visible secret. A wrapper must refuse any request outside this exact task/path inventory.

## Objective outcomes and stop rule

Completion means all12scheduled attempts have a terminal classification and independently checked evidence, or a declared global guard stops the cohort. Retain every failure/censored/missing slot in the denominator; no resampling.

For each arm report: provider/schema failures, edit-precondition failures, stale-handle refusals, multi-edit opportunities, successful reinspections/edits, budget stops, oracle reached and verified final acceptance. An eligible freshness opportunity is a second or later edit after a prior edit; no instruction should force unnecessary changes simply to create one.

A **go to a separately approved TSR comparison** requires all of:

- New interface reaches final oracle in at least5/6attempts and has at least3/6verified acceptances.
- Zero unexplained handle-binding/replay/receipt failures and zero new-interface stale-reference or malformed-response endings.
- At least three new-interface attempts contain an eligible multi-edit opportunity, all with fresh independently replayed references. Otherwise the prior failure mechanism remains insufficiently exercised: classify that question inconclusive, even if one-edit repairs succeed.
- New interface has no lower oracle-completion or acceptance count than the contemporaneous old interface. Ties show viability, not demonstrated superiority. Six pairs cannot establish broad statistical reliability.

A go/no-go outcome is an operational decision, not proof that TSR improves repairs. Only after a go should a separate fixed, repeated Record-versus-Rules real-task plan be proposed; it is not included in this authorization.

Stop immediately on unexpected disclosure, evidence mismatch, containment failure, uncertain billing, unavailable model, changed pins, disk below200GiB, or cost/call/token/wall bounds. Ordinary model/repair failure is recorded and the next preassigned arm may run; do not stop early merely for favorable results. A global stop ends the cohort without automatic restart.

## Remaining launch work and authorization

Offline freshness replay and safety checks are complete in the companion evidence. The experimental codec deliberately refuses live transport. Before launch, a narrow task-scoped wrapper must connect that codec to the existing reservation/settlement transport, validate both complete wire bodies offline, and freeze the paired controller. This is bounded integration, not another corpus/harness exploration; it must not weaken any guard.

The exact new approval required is: **authorize the12attempt, six-pair Record-mode validation above, sending only this inventoried reconciler source and bounded observations to api.cheaperinference.com using deepseek-v4.1-flash, with a$5additional cap within the existing$100aggregate limit.** Approval does not authorize a TSR cohort, extra tasks, cleanup, restart, push, publication, PR, merge or deployment. No such approval has been requested or received for this batch.
