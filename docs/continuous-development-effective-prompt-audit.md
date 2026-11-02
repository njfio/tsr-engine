# Effective-prompt audit after b1f735df

Offline audit only. All fourteen retained request bodies were authenticated against
their recorded SHA-256 digests. The completed test, accounting and results are
unchanged. Exact request inventory/excerpts are in
`target/wanting-presentation-audit/effective-request-audit.json`. Excerpts below
contain only generic instructions; no credentials or private material are included.

## Findings

**One actual rubric inconsistency, not an observed rejected development.** Both
artifact-review requests (`ee99730aad18…`, `93e7dbe90d69…`) contained:

> Reject invented user testimony, empirical claims, an unrelated task suggestion
> masquerading as wanting, or new action authority.

That blanket phrase in `engine-project-runtime-review.py::CRITERIA.parent_fidelity`
conflicts with the same rubric's `epistemic_grounding`, which permits faithful
source/receipt-bound empirical statements. No provisional proposal was generated
or reviewed, so this cannot be described as a demonstrated rejection of wanting.

**Artifact-maintenance emphasis is a possible presentation bias, not a proven
causal explanation.** Every proposal system message began:

> Decide what, if anything, the supplied experience warrants for this public browser project.

The shared `proposal_responsibilities.generator` explained an untested initial edit;
`review_stage` explained whether “a complete incremental edit is useful and safe.”
The supplied context nevertheless explicitly said provisional wanting required no
prior aspiration, successful return or unmet Contrast. Its actual contract stated:

> Optional genuinely generated wanting grounded in one or more current contributors,
> including generated children. No prior aspiration, successful return or unmet Contrast
> is required. Explain the connection honestly; uncertainty and no novelty are valid.

Thus the path was available and described, not unreachable. The overarching role
description was incomplete relative to that path. Grounding a possible wanting in
contributors is different from proving an empirical outcome or corrective need.

**The old prohibition was not active.** `engine-pursuit-reflection.py` retains v1/v2
strings saying “Do not invent desires” for immutable earlier capture contracts.
This run's actual `e24786f8a303…` interpretation used v3:

> Do not invent user preferences, observations, fulfillment or authority. Optional
> possibilities may suggest generated wanting with explicit uncertainty; only the
> separate ordinary proposal and independent admission path can make it a native desire.

Its reviewer described possibilities as optional hypotheses and permitted later
ordinary admission. Neither legacy string was silently rewritten in this repair.

## Actual timing and opportunities

| Choice episode | Proposal request prefix | Context and result |
|---|---|---|
| Initial construction | `a5d43cbe8c6c`, `c4ce00c7c05a` | First invalid shape, then format correction; one choice episode |
| Revision after independent artifact rejection | `745c636987fd` | Corrected crossing-counter proposal |
| After artifact execution and qualitative interpretation | `a8af38038df7` | Retention; render/source evidence, not gameplay controls |
| Crossing-first continuation | `800a0d2d4a0a` | Retention after six unavailable observer attempts and migrated valid Unknown |
| Growth-first continuation | `11b5e76800c8` | Retention after twelve actual control traces plus unavailable history |
| Mixed-pursuit continuation | `241b904bfbde` | Retention after twenty-four cumulative actual traces plus unavailable history |

Seven proposal calls therefore represent **six choice episodes**, not statistically
independent samples. Only two followed actual gameplay controls. The remaining seven
model calls were two artifact reviews, one qualitative interpretation and its
independent assessment, and three criterion assessments. Interpretation could suggest
possibilities but could not itself admit a new identity. Formatting, validation,
accounting and evidence callbacks are not extra development opportunities.

Bare retention ends the current checkpoint (`engine-project-cycle.py::Cycle.run`),
preserving open desires rather than acceptance. It does not trigger another thought
automatically after retaining both artifact and meaning. A deliberation can instead
retain the artifact with separately reviewed development. The next frozen external
input created the later opportunities in this assisted test. No stopping semantics
were changed or disguised as an extra opportunity.

The latest choice explicitly said no sufficiently supported artifact change was
apparent. That is evidence of artifact-oriented rationale, not proof that all
potential wanting was evaluated, or that the wording caused omission. There was
no recorded provisional proposal, admission, or later influence to evaluate.

## Narrow prospective correction

* `engine-browser-adapter.py::proposal_body`: lead with current wants, relationships,
  possibilities and experience; explicitly distinguish uncertain generated wanting
  from asserted facts. Corrective evidence/success is not a prerequisite. Clarify
  that bare retention retains both artifact and meaning, while deliberation can keep
  the artifact and propose development. No option is ranked or required.
* `engine-browser-project.py::RESPONSIBILITIES` v2: include provisional wanting and
  its own review basis alongside incremental edits; retain evidence/authority limits.
* `engine-project-runtime-review.py::CRITERIA`: reject **fabricated or unsupported**
  empirical claims, not all empirical statements. Assess wanting by contributor
  connection and honest uncertainty rather than whether it fixes an artifact defect.

Native schemas, evidence validation, lineage, admission, review independence,
permissions and terminal behavior are unchanged. No example wanting, desired
synthesis or novelty reward was supplied. Authored native fixtures are tests, not
instructions to a live model. Independent semantic/code review passed.

## Verification and limits

Nine offline checks passed: five serialized presentation/guardrail checks plus four
real native authored cases for recursive provisional admission and inert restart,
stale/forged source and Stop, retention without novelty, and rejected development
not authorizing a dependent artifact. Initial test-only wording typo and four
missing-opt-in skips are retained; the final explicit opt-in run passed all nine
with zero skips in 37.602 seconds. No provider calls or live admissions occurred.

These tests show consistent instructions and unchanged mechanics, not better model
creativity. The remaining uncertainty is whether this model would choose development
on an opportunity with these clarified instructions. This audit neither establishes
a causal diagnosis of prior retention nor justifies ritual repetition of the task.

Separately, the earlier optimized runtime comparison remains limited: six-bundle
debug/optimized request JSON matched exactly; eighteen-bundle debug timed out while
optimized completed. **Full-state debug/optimized equivalence is unproven.**
