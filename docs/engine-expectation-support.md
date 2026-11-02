# Expectation support and bounded fallback

The notebook adapter now reports support beside each frozen assessment. This is
descriptive evidence, not a fitted calibrator. `expectation_support` names the
progress event, compatible dispatch count, effective similarity weight, weighted
receipt identities, applicability limits, uncertainty, and calibration status.
Existing `prediction_evidence` and `calibration_evidence` retain their distinct
provenance. A plan's estimate concerns its proposed terminal action; the first-step
forecast remains separate in `plan`.

The available receipts come from correlated turns in one run. They establish zero
independent mature episodes. Their smoothed progress rate is unvalidated; neither
an empirical confidence interval nor validated feature-range coverage is claimed.
Changing the canonical active-original scope removes incompatible success labels;
observation revisions independently restrict residual transfer. Historical records
are not backfilled or rewritten.

## Declared fallback

`bounded-mechanical-evidence@v1` is a conservative rules policy within existing
notebook authority. It requires all of:

- The sealed first action cannot lower an active original standing predicate.
- It either makes independently bounded progress on selected active originals, or
  gathers new information needed for a dependent terminal action that independently
  offers such progress without standing loss.
- Its estimated benefit clears the existing action and continuity cost.

The terminal action is neither executed nor authorized by this check. It requires
fresh generation and selection after the first return. An unrelated inspection
does not qualify merely because the generator attaches a plan. Among eligible
actions, the existing shared-kernel scores still determine selection. Weak
statistical support therefore does not universally prevent useful local inspection.

Every assessment records eligibility and its reason. With none eligible, the
command returns an explicit abstention reason and preserves state, desires and
contrast; it does not fabricate a failed return. The positive-score condition is
an operational conservative choice, not a calibrated confidence threshold. It can
abstain from mechanically useful work when the unvalidated estimate is pessimistic.

Adapter version `shared-obligation-kernel@v7` isolates counterfactual audits from
historical v6 selection behavior. No new authority, filesystem source, model call,
or domain action was added.

## Inspect and verify

Use the ordinary direction interface:

```sh
target/debug/tsr engine --directory RUN direction-request
target/debug/tsr engine --directory RUN deliberate --response offer.json
```

The result's `turn.offered` exposes the support and fallback records; subsequent
requests expose them in `recent_returns`. Legacy receipts without these fields
remain readable and are not relabeled. `expectation_support_policy` states the
current request policy even before any return exists.

Deterministic verification: 94 CLI library tests passed, one existing ignored;
26 Python driver tests passed. Existing withdrawal, restart, unchanged-state,
scope-change, stale-offer, authority and no-progress tests remain intact. Added
cases cover zero history, correlated receipts without episode inflation, safe
information gathering, standing regression versus withdrawal, and an unrelated
zero-capacity continuation. Independent review found and corrected the initial
plan-presence loophole and required policy versioning. Logs are
`target/support-tests.log`, `support-python.log`, `support-clippy.log`, and
`support-build.log`. A fresh ordinary-command fixture selected bounded inspection
with zero mature-episode support and preserved its result across restart; evidence
is `target/support-smoke/result.json` (authored deterministic fixture, not model
generation). No live inference was needed; nine local slots remain unused.

## Further calibrated policy: exact evidence requirement

PRD B8 requires at least 150 mature episodes per named event and at least 30 per
family, with validated Tier 1 calibration before Tier 2 support-floor enforcement.
These are necessary counts, not a sufficient quality guarantee. Outcomes need
independent acceptance evidence, maturity/censoring labels, repository lineage and
timestamps; splits must avoid near-clone and time/repository leakage. Every green
in held-out evaluation needs independent review. The named acceptance event is
not interchangeable with notebook coverage progress.

A safe collection path is the existing isolated project execution/acceptance
pipeline using explicitly admitted sanitized tasks and protected independent tests,
with operator semantic review where no domain verifier exists. Preserve rejected,
disputed and censored outcomes. Deduplicate task lineages and group all turns from
one episode before fitting or evaluating. Import only owner-provided authorized
data; do not mine private sessions or create synthetic positives to meet counts.
Training, held-out validation and feature-range support must precede claiming a
calibrated commitment policy. This pass does not perform that data collection or
claim that the statistical gate has been met.

## Bounded tradeoff correction (v8)

The subsequent policy `bounded-mechanical-evidence@v2` also permits a sealed,
reversible local composition when the shared kernel, evaluated with exact independent
predicate effects, has positive net value after standing loss and cost, and the
existing estimated score is positive. This avoids making every regression an
absolute prohibition. It does not extend to unknown effects or external actions.
`exact_first_step_net` and `exact_standing_loss` expose this basis. Regressed wants
remain unmet and the no-regression progress event still resolves false. This is a
policy tradeoff under the existing numerical proxy, not proof of semantic utility.
Old receipts omit these new optional fields; v8 audits do not reinterpret v7 choices.

The negative-net regression fixture now uses a dominating length loss, while a
separate exact-value fixture verifies positive-net tradeoff eligibility and unchanged
controls. No protected test or historical receipt was changed.
