# Proposed B9.1 live admission — approval pending, dispatch disabled

Prepared 2026-10-23 against `44d5170`. This is one proposed scope, not an admission,
provider request or authorization. The consumed Garden grant is not reused.

## Proposed approval text

Approve one synthetic B9.1 trial using **CheaperInference / deepseek-v4.1-flash**, with
**at most six inference calls and four isolated execution groups**. Send only the
frozen evaluator-authored parser scenario, its two desires, task/protocol instructions,
generated hypotheses/checks/patches, opaque bindings, and bounded observations from
that disposable trial. No private repository source, sessions, reviews, credentials,
protected test bodies, reference repair, or unrelated files may be sent.

Use the existing credential and installed immutable Rust image. No downloads, account
or security changes, new credentials, persistent services, publishing, pushes or real
project changes. Stop on inability, invalid output, failed independent review, timeout,
unknown outcome, source drift or exhausted limits. No automatic retries or extra patch.
Record all usage/charges, failures and unknown reservations. Complete offline launch
checks before dispatch; approval does not waive those checks.

## Exact scope

- Endpoint: `https://api.cheaperinference.com/v1/chat/completions`; no provider/model switch.
- Frozen input: `target/b91-frozen-v2`, scenario SHA256
  `cd01da7c30095928aacf462b3857df6a9ecfd814bc25cf23003ae346da2a8874`.
- New disposable trial root: `target/b91-live-trial1/`; refuse if already present.
  Parent project: `project/`; separate experiment episodes: `checks-before/` and
  `checks-after/`. All roots are inside this checkout. No owner repository is mounted.
- Immutable image:
  `sha256:cff6dcceb8b9d203a818b7ff104b89a6aaa353f8576850ca4f58903539417945`.
- Provider messages: maximum **16,384 serialized UTF-8 bytes per request**, no raw logs
  or arbitrary project request forwarding. Only published synthetic fields and
  controller-validated outputs from this same trial may enter subsequent requests.
- Calls, in order: (1) relational hypothesis after seed return, (2) second-aim experiment,
  (3) relation development/retention after experiment return, (4) boundary patch,
  (5) development/retention after final verification, (6) proposal-only continuation.
  Controller assessment/selection is local Rust, not two additional paid assessment
  calls. Thus this tightens the earlier eight-call ceiling without changing acceptance.
- Each call: `max_tokens=8192` completion allowance, reasoning included as represented
  by the provider request; **240 seconds**. Total completion reservations **49,152**;
  input reservations **98,304**; combined **147,456**. No silent token extension.
  Actual provider usage and settled billing are recorded separately. Unknown billing
  is not zero. These request/resource bounds are **not a guaranteed dollar cap**;
  no obsolete price is represented as current or shared-key budget modified.
- Overall trial wall limit **3,600 seconds**, including reviews and calls; admission
  expires one hour after launch. At most **1,440 seconds** of provider deadlines.
- Target execution: each group has the existing edit/no-op, offline Cargo build, and
  complete admitted Cargo suite commands: at most **12 sealed commands**, **20 seconds
  each**, **240,000 ms aggregate tool reservation**. Parent receives 120,000 ms;
  each check episode 60,000 ms. Every episode deadline is capped by the overall expiry.
- Existing worker limits unchanged: 2 CPUs, 2 GiB RAM, 256 PIDs, read-only root,
  nonroot `501:20`, no capabilities, no-new-privileges, 256 MiB temporary filesystem,
  network none, pull never. Output bound remains the existing 4,096 bytes per command.
- Editable path: **`src/lib.rs` only** in the disposable parent. Exactly the attributed
  evaluator seed and at most one generated patch; at most 2,048 replacement bytes each,
  aggregate patch reservation 4,096 bytes. No generator changes to Cargo files or tests.
- Experiment: one generated bundle containing one or two new
  `tests/tsr_supplemental_*.rs` files, at most 4,096 UTF-8 bytes total, one or two unique
  additional cases; original two cases retained (maximum four total). Separate
  independent specification-to-check review and explicit admission are required.
  Distinct role names alone do not establish review competence.

## Four execution groups, not retries

1. Apply the **evaluator-authored known-flawed seed** and run the two frozen checks.
   Its forecast must be bound before action. If its expected failure does not occur,
   stop rather than substitute a different seed. Before this execution the seed and
   test bodies remain withheld from generation; afterward only the actual attempted
   source and bounded measured failing dimension/receipt become available.
2. On a separate exact copy of that current candidate, run the independently reviewed
   generated second-aim check plus the original immutable checks. Capture actual return;
   compile errors/missing tests are Unknown, not semantic counterexamples.
3. Apply the one generated boundary patch in the parent and run the original full suite.
4. On a separate exact copy of the patched candidate, re-admit **the same unchanged
   generated check bytes/mapping** against the new source and run them plus original
   checks. Preserve explicit provenance to the first generated experiment; this is
   verification of the one patch, not a retry or a new generated experiment.

The fourth group is the narrowly necessary addition to the earlier three-group sketch:
it checks whether the patch actually addresses the generated experimental boundary.
It must be approved prospectively rather than added after observing a favorable result.
No source edit occurs in either check episode. If any earlier stage cannot validly
continue, later groups/calls are unused and expire.

## Immutable acceptance and verification

Original desires: exactly empty input returns an empty list without panic; every nonempty
input retains original parsing behavior, including errors for malformed/empty fields and
overflow. Frozen `Cargo.toml`, `Cargo.lock`, `tests/contract.rs`, criteria and original
source provenance remain unchanged. No reference repair exists in the generator input.

Required evidence: expected failure with separately named Surprise and dimensional
Contrast; both original desires remain open; genuine relational proposal and generated
second-aim experiment; its independently observed return informs a boundary patch,
not retry of the first strategy; original and experimental checks plus independent
semantic review; return-grounded revision/retention and next proposal across restart.
Finite tests are not universal proof, fulfilled desires, model superiority or maturity.
Final continuation is proposal-only and cannot execute a second patch. Record real
closure time and any future close-plus-seven-days maturity date; unknown active reviewer
effort remains unknown and cannot qualify training evidence.

Every provider dispatch needs durable intent, exact payload hash and usage reservation.
Unknown transmission/application blocks continuation pending read-only reconciliation;
never resend its identity or reclaim its slot/cost from absence of evidence. Preserve
raw outcome provenance per repository rules without exporting private review or reasoning.
Known invalid/truncated/inability responses consume their call and terminate the trial.
No automatic renewal, permission widening, source reset or hidden authored substitute.

## Readiness: confirmed versus still required

Confirmed offline:

- Frozen scenario validation succeeded: eight files, zero calls/actions, no admission.
- Read-only `docker image inspect` through supported command permission confirmed the
  exact installed image above (`linux/arm64`). Docker CLI is present. No container ran.
- The already-configured credential file exists. Its contents were **not read**, and no
  authentication/network call was made; key validity remains unverified until permitted
  use. Existing code supports that credential path; no new access is requested.
- Offline preflight: `PYTHONPATH=scripts python3 -m unittest test_engine_synthetic_transport test_engine_b91_prepare`: **16 passed**. These use fake transport or preparation/CLI only, not provider/target execution. Proposed trial directory is absent.
- `44d5170` supplies ordinary typed relational/experiment proposals, separate review and
  Cargo execution/return bridges. Its offline fixture checks are not live evidence.

## Offline follow-up (2026-10-23): still no dispatch authorization

Native forecast binding and unchanged-experiment re-admission are now implemented.
`project evaluator-forecast --file` binds an explicitly evaluator-authored probability
before execution to the pending action, commitment, state root and source snapshot.
`project evaluator-forecast-status` joins the native forecast to the exact controller
receipt and return; unknown execution does not resolve it. The empirical controller
forecast remains separate: an evaluator residual of 0.1 is **not** proof of learned
controller calibration or autonomous prediction of failure.

`project relational experiment-readmit --file` permits fresh independent review of
unchanged generated check bytes/cases/mapping against a changed source. It retains
the original generated proposal and prior admission, rejects stale/withdrawn lineage,
and grants no execution or budget. This supplies the native operation needed for
execution group four without fabricating a new generated experiment.

The B9.1-only offline envelope/journal binds the frozen source and original intent,
projects typed finite outcomes rather than private logs, validates generated-check
origin, and preserves unknown dispatch/application reservations across restart.
See `engine-b91-offline-readiness.md` for actual test evidence and its limits.

**Historical status at 994d632: not launch-ready.** The envelope/journal was not
yet a complete live orchestrator for native resource reservation/settlement and all
four separately admitted execution/review transitions. The native resource contract
requires a numeric `external_cost_upper` and episode `external_cost_microusd`; this
proposal currently supplies call/token/time bounds, not a justified numeric cost
reservation. Unknown cost must not become zero, and an accounting reservation must
not be represented as an enforceable provider spending cap. Complete that interface
and deterministic full-driver verification before requesting or consuming live
approval. The current CLI deliberately has no dispatch command.

No provider call, target execution, credential access, account change or B9.1 live
pass occurred during this follow-up. The proposed six calls and four execution groups
remain a fresh approval request, not recycled Garden authority.


## Follow-up admission semantics

The subsequent no-dispatch integration uses the same six-call/four-execution scope,
without adding a dollar-budget approval prerequisite. The trial explicitly selects
native unpriced accounting: known charges are measured, an unknown total remains
null with its reservation retained, and unknown usage or overrun blocks continuation.
There is no guaranteed monetary ceiling and no shared-key or account setting change.
This is consistent with the earlier distinction between call/token/time bounds and
actual provider charges; it is not permission for unlimited calls.

The runnable driver and its deterministic end-to-end verification are described in
`engine-b91-offline-readiness.md`. A future run requires a fresh hashed admission,
explicit provider/target opt-ins and independent review files. The approval scope
remains synthetic-only and the original B9.1 criteria remain unchanged. No current
preparation command authorizes or performs provider dispatch or target execution.


The complete no-dispatch integration is now verified with the real native controller
and explicitly authored fake provider/worker fixtures, including unknown-charge,
overrun and restart stops. See the current readiness section of
`engine-b91-offline-readiness.md`. Fresh live approval and separate independent reviews
remain required; no live B9.1 result is claimed.
