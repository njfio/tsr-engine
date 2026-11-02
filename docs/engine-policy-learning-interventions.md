# Saved-draw learning interventions

`decision-policy-view` now includes `learning_interventions` alongside factual
selection traces. New configured-policy decisions record the intervention before
commitment, including abstention. No model calls or alternate actions are run.

## What is held fixed

The primary comparison uses the factual policy, pre-offer state, candidate
identities, **factual eligibility set** and recorded 53-bit draw. It recomputes
only the declared learning inputs:

- Notebook: remove probability/residual return-history learning while preserving
  observations, active desires, offered actions, commitment and policy.
- Adaptive/project: reset empirical dispatch-success probability to the prior
  0.5. Preserve observed returns, independent semantic review and semantic
  projection learning, costs, desires, and commitment. This is a deliberately
  narrow intervention, not removal of every learned quantity. Project ranking
  uses the full option set before restricting to offered actions, so omission of
  the prior action by a provider cannot accidentally remove Belief continuity.

`ranking_changed` compares stable best candidates; `same_draw_choice_changed`
compares resulting selections. `factual_sampling_departure` identifies when the
factual draw selected something other than the stable best. Probability vectors
and factual propensity remain separately visible. A changed ranking need not
change the sampled choice, and a changed choice is not proof of improved outcome.

## Changed eligibility and missing evidence

`changed_support_scenario` separately evaluates the counterfactual eligible set,
with explicit added/removed IDs. It never silently replaces the primary support
set. Fixed-support replay can include an action that would no longer be eligible
under the intervention; it is a hypothetical isolation control, **not permission
to dispatch it**. A missing score for a factual candidate makes fixed-support
comparison unavailable and lists the missing IDs.

Version-1 deterministic, abstaining, empty and single-candidate traces did not
save an unused random draw. If the counterfactual now requires sampling, the
report retains its probability distribution but marks sampled selection and
propensity unknown (`missing_saved_draw_for_sampling`). It does not generate a
new draw and call it historical evidence. Deterministic counterfactuals remain
computable. Unsupported historical policy versions are reported explicitly;
factual v1 traces must replay exactly before comparison. Old traces without an
intervention are not backfilled or rewritten.

## Integrity and inspection

Each `selection-learning-intervention@v1` artifact binds the factual trace digest,
embeds that trace, states its intervention and records the comparison. The view
requires controller provenance, a preceding recorded controller trace, matching
content hashes and matching embedded factual copies. No outcome or propensity
record is overwritten. Restart returns identical stored comparisons.

```
target/debug/tsr engine --directory RUN decision-policy-view
```

The older notebook `learning_audit` remains suppressed when policy is configured:
its argmax-only interpretation is incompatible with exploration. The stored
policy-aware intervention is the replacement, including pre-action abstentions.

## Verification

- Four pure selection tests cover deterministic and exploratory comparisons,
  near-tie abstention, empty/one-action sets, changed/missing eligibility, fixed
  draw and unchanged-state controls, unsupported historical versions, tampered
  propensity, quantized distributions and invalid candidates.
- 109 CLI library tests pass; one existing diagnostic remains ignored. New
  controls include notebook learning after an observed return, adaptive ranking
  change after observed failure, identical saved draw, restart, omitted previous
  project commitment, and a forged factual trace reference.
- Clippy with warnings denied, CLI build and whitespace checks pass. Independent
  read-only review found and verified the commitment/provenance corrections.

Logs: `target/intervention-domain-tests.log`, `target/intervention-cli-tests.log`,
`target/intervention-clippy.log`, `target/intervention-build.log`.

These are **controlled model-state interventions**, not empirical causal
identification, calibrated Tier 2 performance, or observed alternative outcomes.
No inference slots were used. One remains unused; request29 inability and
request13 unknown/abandoned remain intact. No provider was started.

## Remaining operational boundaries

The preserved generation task remains capability-limited under the tested
installed model/contracts; no new useful direction or live recovery is claimed.
Validated common-event calibration remains dependent on independently reviewed,
deduplicated mature episodes. The 151 qualified tasks remain recorded; mature
eligible episode count remains unknown.

A distinct original PRD integration gap remains: the operational notebook/project
selectors use proxy progress estimates, while `tsr-learn::bundle::load_bundle` and
`head::predict_checked` exist separately. A next implementable step is a read-only,
version-bound **shadow** adapter for validated common-event bundle predictions,
with feature/support checks and explicit unavailable status. It must not replace
the incumbent or claim calibration when qualifying bundles/data are absent.
Shadow feature extraction must remain pre-action; outcome/reflection evidence
cannot become candidate features. This is separate from fixing generator ability,
and does not require inventing new values or authorizing external execution.
