# Remaining gap after Trial 11 and its linked continuation

Read-only source/evidence audit at `1fcdd84`, 2026-10-25. No provider calls, target
execution, adoption of the generated final proposal, or historical-state changes.
The only executed checks were existing native unit tests with authored Fake runners.

## What the original criterion actually requires

PRD §B9.1 (`docs/PRD.md:463`) defines its two-desires gate as: both aims preserved
open after a predicted failure, dimensional Contrast, then a bounded experiment
for the second aim followed by a boundary patch rather than retrying the first.
It does **not** require another experiment, native accepted-task closure, richer
emergence, mature calibration data, or a statistically significant ablation.

Trial 11 demonstrates this literal gate: both desires remained open after the
expected failed seed; the generated second-aim experiment failed on the seed; a
generated patch then passed the original checks and unchanged experiment. The
main native log's event sequence 37 records the second-aim criterion
`62d8ae4037e94dfb60cf0f2e326f79fdba11d2cfd2981fcb8428ece7ae6360b2`
as `wrong_behaviour`, separate from forecast residuals 0.5 and 0.1. The 0.1 forecast
was explicitly evaluator-authored, not learned model calibration. Sequence 63
records the subsequent passing return with no unmet test criteria and residual
-0.5. A scalar surprise was not substituted for dimensional Contrast.

The continuation additionally demonstrated return-informed guidance revision and
a next generated, unexecuted experiment. Earlier reports' wording about an
incomplete “full B9.1 cycle” referred to the expanded six-stage driver/acceptance
protocol; it must not be read as adding those stages to the literal PRD gate.
The original stopped trial remains stopped; the later continuation is separately
linked evidence, not a retroactively uninterrupted success.

The driver fixes this benchmark's phase order and requires a second-aim experiment;
the model genuinely supplies its test and patch content. This is not evidence of
unprompted selection among arbitrary goals. Broader bounded relational generation
has separate workshop/garden evidence; neither passing parser tests nor this audit
proves the entire general TSR goal.

## Exact current gaps

| Item | Current evidence and classification | Smallest next step |
|---|---|---|
| Original desires | Both remain open because no native independent acceptance decision was adopted. Test passes alone intentionally do not close them. | Independently assess both original criteria against the unchanged tested candidate; record a source-bound accept/reject/unresolved decision only with authority. Never force acceptance. |
| Belief and expectation | Tool-return residuals and source-informed revisions exist. Semantic acceptance history has zero reviewed outcomes; its active pre-closure estimates were 0.5. Closed continuation hides guidance/belief views. | An authorized independent acceptance record feeds the existing native fold and resolves the matching frozen expectation. This would be one bounded observation, not calibrated accuracy. |
| Native accepted-task status | Final status is `review_required`, `accepted=false`, `decision_record=null`, despite the source-bound passing receipt. This is an intentional advisory boundary, not a demonstrated defect. | Use the existing `project acceptance review` contract, with evidence for every original requirement and exact source/receipt/criteria identities. |
| Accepted resource episode/common-event label | The continuation deliberately had zero executions and closed advisory-only. Its reused verification receipt predates its own admission. | Do not turn it into a newly executed accepted episode. Keep both historical episodes and costs immutable; a late candidate review is distinct from an in-budget training episode. |
| Final proposed checks | Additional endpoint/field-error examples are relevant but optional; several cases repeat prior checks. They are not original acceptance requirements and have no execution authority. | No execution is needed merely to satisfy the existing gate. Assess the existing repair first. |
| Empirical claims | Reliable repeatability, causal benefit of relational guidance, calibrated common-event probabilities and mature independent data remain unproven. | These are future evaluation/data dependencies, not reasons to invent more requirements for this bounded task. No campaign is implied. |

## Existing implementation, not missing wiring

`acceptance.rs:195–242` checks current snapshot, requirements revision, passing
receipt, complete per-requirement evidence and absence of a current supplemental
counterexample. The special-case-empty branch plus the unchanged original
nonempty expression is available for independent source-equivalence review; finite
test counts alone are not the argument for all nonempty inputs.

`acceptance/cycle.rs:512–616` records the independent return, resolves a matching
frozen forecast once, records dimensional Contrast, and updates the scoped desire
statuses without overwriting their original meanings or overriding withdrawal.
`cycle.rs:224–291` deduplicates/supersedes review evidence by receipt and preserves
unresolved judgments as unknown. `cycle.rs:293–383` feeds reviewed observations
into shared-kernel expectations with explicit action applicability. `cycle.rs:385–484`
exposes review-derived rates; these are engineering folds, not empirical calibrated
certainty. Later source/criteria changes invalidate satisfaction/review applicability.

`lifecycle.rs:378–381` separately requires review and execution receipt to belong
to the admitted episode before reviewed lifecycle closure. The zero-execution
continuation cannot satisfy that check with an old receipt. Weakening it, changing
timestamps or rerunning successful tests solely to manufacture a fresh episode
would obscure provenance. Native candidate acceptance and mature/common-event
resource-episode eligibility must not be conflated.

No implementation change is justified by these facts. The smallest remaining
proof point is **a separately authorized independent review/adoption of the already
tested repair**, using existing evidence and no model call or target execution.
Use a new linked review copy so Trial 11 and the advisory continuation remain
immutable. Bind the decision to:

- snapshot `cf21151fc9de0c2f452474c9bca71b38a41c2414383a65652ec7162c8cc3f1e1`;
- requirements `2816bb065de25c3b697eb349134323a2085f7f4970e2ea15292431dbf1a85fe9`;
- protected passing receipt `0a47acfbdaf4da8fd2677e4cdc1e4acf31087103d731d7b223c07040402fcaf9`;
- supporting unchanged-experiment receipt `322471c35c939a13cf623c76b544c60777691fbb595401619c18930582cc8e18`.

After adoption, inspect per-desire status, receipt-deduplicated evidence and frozen
expectation resolution across restart. Rejection or unresolved evidence must remain
possible. This authorizes neither applying the final proposed experiment nor
relabelling an advisory lifecycle, manufacturing mature evidence, or declaring all
future standing desires satisfied. The necessary decision is whether to permit
recording this existing-candidate judgment beyond the prior advisory-only scope;
there is no reason to request more model calls or parser executions for that step.

## Checks and review

Three existing compiled native tests passed, using Fake runners and temporary
fixture projects only (`target/b91-completion-gap-fixtures.log`):

- `independent_acceptance_survives_restart_and_invalidates_on_change`;
- `shared_kernel_uses_independent_review_and_preserves_unknown_on_restart`;
- `native_lifecycle_tracks_actual_adapter_returns_review_budget_and_restart`.

These verify that green tests alone leave desires open, stale/missing review is
rejected, independent rejection changes support/ranking, acceptance changes scoped
status, restart preserves the fold, and lifecycle budgeting remains separate.
They are not a live acceptance record for the parser repair. No protected tests,
fixtures or implementation files were edited. Independent source review by
`readiness_review` reached the same conclusion: acceptance adoption is the current
gate; extra boundary tests and cross-episode relabeling are not prerequisites.
