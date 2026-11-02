# Local information forecast bridge

This opt-in bridge freezes an operator-supplied, uncalibrated probability before a
bounded information read and resolves it from its authenticated native receipt.
The outcome is `validated_information_receipt@v1`: the selected bytes were accessed
and validated for this dispatch. It does not predict truth, answerability, desire
fulfillment, usefulness or a nonphysical quantity.

## Reused path and exact limits

`qualitative/information.rs` reuses domain `Forecast`, `Horizon::ThisDispatch`,
`Probability`, `Surprise`, and `tsr-forecast` resolution/scoring types. The original
native reader and all path, digest, inquiry, policy and authority checks remain.
Existing APIs and projections remain unchanged in stores without information
attempts. The new path checks known eligibility before a durable intent, then
rechecks sensitive state under the read lock. Invalid caller input does not create
an uncertain operation. A successful receipt is bound to the exact attempt,
inquiry, corpus, IDs and pre-read relevance; no later receipt can substitute.

The intent commits before file I/O. The read/receipt transaction can fail or be
interrupted; intent remains unresolved. All reader entry points then refuse a
repeat. If the receipt committed before settlement was interrupted, the explicit
resolver can settle that receipt without rereading. Duplicate settlement fails.
No failed attempt is guessed false and no missing receipt is treated as success.
There is intentionally no automatic retry or reset command.

Resolution writes a generic Surprise residual, with **no Contrast**. Native next
request and the existing public attachment expose attributed access-forecast
history, receipt identity, residual and Brier contribution. No new weights, action
scores, accepted meanings, permissions or preferences are created. Predictions
remain attributed to the supplying operator; no model forecast is impersonated.

This is a narrow typed/scoring bridge, **not full integration into the generic
TaskFacts resolver**: the forecast lives in the source-bound native attempt, and a
domain-specific resolver authenticates the receipt before using generic scoring.
The generic named command/task registry is unchanged; it cannot infer these events
from command exits. No calibrated or empirical next-read probability is produced:
success-only known outcomes plus unresolved failures would bias such an estimate.
Thus later generation can see frozen expectations and discrepancies, but learned
selection influence remains unproved. Known-failure/censoring evidence and a typed
information-outcome registry are still needed before a reliability estimator.
Question usefulness needs a separately defined independent judgment with unknown
allowed; it must not be inferred from file-access success.

## Local entry points (not live authorization)

`tsr-qualitative DIRECTORY information-expectation` is read-only.
`forecasted-retrieve INPUT.json` accepts `{choice, probability}`; `choice` is the
existing exact `InformationChoice` with current root, admitted corpus/IDs, reviewed
inquiry and bounded pre-read relevance. It performs a real read and therefore needs
its own applicable action admission; this change does not authorize using it on the
campaign corpus. `resolve-information INPUT.json` takes `{attempt, receipt}` and
only reconciles a matching already-durable receipt.

The existing paid export/runner remains pinned to its old profile and does not
silently enable this new path. Frozen live5 and continuation artifacts are unchanged.
No provider request, campaign inspection, live adoption or model call occurred.
All newly executed reads were deterministic tests of temporary authored files.

## Verification and remaining proof

New controls cover forecast mismatch (0.2 followed by actual success gives -0.8),
unchanged parent/authority, stale input and cross-root rejection, invalid probability,
invalid relevance/IDs/corpus with no state mutation, restart persistence, altered
bytes leaving an unscored unknown, blocked repeat reads, wrong receipt, duplicate
settlement, and crash after receipt before settlement. A second authored cycle
retains the first forecast/outcome. Existing path/symlink, review, withdrawal and
source-provenance controls remain in the affected suite. These are local wiring
checks, not live evidence or reliable prediction. Verification: 24 qualitative tests and five public-attachment tests passed. A final
six-test `information_` run passed after adding the attachment assertion; CLI
`cargo check --offline -p tsr-cli --bin tsr-qualitative` passed. No full workspace
suite is claimed. Two development tests initially
exposed missing fixture review and new-entry source classification; both were fixed
before the affected run passed.

## Prospective relational evaluation (design only)

Use the paired held-out design in `engine-two-cycle-fidelity-audit.md`: three new
non-private tasks, two cycles per arm, frozen external acceptance, same model,
budget, raw evidence and ordinary possibility memory, blinded assessors and
randomized arm order. Full native relational guidance is compared with the same
text, parent links, provenance and review judgment supplied as plain memory.
Only operational native treatment is removed. At a divergence each arm receives
its own actual returns; no outcome is fabricated to maintain symmetry.

Before any launch, demonstrate a real operational intervention in local request/
selection traces. Current public attachment mainly supplies advisory context:
if the only difference is wrapper/label wording, label the evaluation as a framing
comparison. Do not manufacture new weights or a selector to make the arms differ.
No relationship/no difference/abstention are valid results. Information access scores
cannot serve as the acceptance oracle for relationship development or fulfillment.
The tasks and numeric live envelope are not frozen or launch-ready; no live authority
or cap expansion is requested. Campaign stays 218/220 calls, 60/60 operations,
$0.176124/$5; operations are exhausted.

Independent source/documentation review by `/root/inquiry_review` passed after the
pre-intent validation repair. No remaining material blocker was identified for this
explicitly limited bridge.
