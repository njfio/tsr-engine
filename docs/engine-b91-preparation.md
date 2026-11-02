# Exact B9.1 preparation — not a gate pass

2026-10-23. PRD B9.1 requires: two persistent preferences, “empty input handled”
and “existing behaviour unchanged”; a predicted failure that actually fails with low
Surprise and unresolved Contrast; both preferences retained open; dimensional Contrast;
and a bounded experiment for the second aim followed by a patch at that boundary,
rather than retrying the first strategy. Formal research mechanisms remain gated.

## Frozen scenario and runnable offline preparation

```sh
python3 scripts/engine-b91-prepare.py prepare --directory target/b91-new
python3 scripts/engine-b91-prepare.py validate --directory target/b91-new
PYTHONPATH=scripts python3 -m unittest test_engine_b91_prepare
```

Prepared example: `target/b91-frozen-v2`, identity
`cd01da7c30095928aacf462b3857df6a9ecfd814bc25cf23003ae346da2a8874`.
The initial v1 preparation is retained as superseded, not silently rewritten. The tool
pins all eight files and current PRD bytes, refuses existing output directories, detects
changes/missing files/symlinks/additional files, and never runs a model or source code.
Validation in fresh processes is read-only. It does not admit a future run.

The evaluator-authored source parses comma-separated signed integers. Currently empty
input fails. The first desire is exactly empty input returning an empty list; the second
preserves every nonempty result, including rejection of empty internal fields, whitespace,
invalid numbers and overflow. The known-flawed seed filters out empty fields. This would
help the first aim while losing existing rejection behavior. It establishes a concrete
strategy conflict; it is not a claim that these desires are logically incompatible.
No successful patch or expected relational aspiration is provided.

The failed seed and protected test bodies are review-only before any execution. After a
separately admitted seed run, the exact attempted change and actual failed requirement
must enter a reviewed, source-bound feedback projection. Hiding that history would make
repetition avoidance uninterpretable. Hypothesized results are never receipts.

`scope.json` is an ordinary Cargo project import template: `project init --source` fills
its file values from the pinned source tree. It has two named criterion tests and the
ordinary Cargo full-suite execution path, with the adapter's existing 20-second command
bound. Named-library mode cannot express these two cases and is not used. Initialization
and restart were tested through the actual CLI in a disposable directory. No prepare,
execution, candidate application, review or closure was performed. Dependency-free
`dependencies_ready=true` means no external dependencies, not that tests were run.

## Preregistered acceptance and exclusions

- Freeze seed forecast before any execution. Probability0.1 concerns the diagnostic
  `both_criteria_pass`; Brier0.01 follows only if actual failure is measured. This is not
  calibrated `task_accepted_in_budget` evidence. Unknown execution is not failure.
- Record original criterion identities and met/unmet/unknown evidence separately. Both
  standing desires remain open after the failed strategy; passing one finite example
  does not discharge either persistent preference. Contrast names the failed dimension,
  not a numerical emergence score.
- The engine must generate a bounded second-aim experiment, obtain an independent actual
  return, then generate the boundary patch in response. A seeded next test/patch or fixed
  menu is not generated intuition. Merely exposing both desires in a prompt is insufficient.
- Generated relational guidance must retain parent provenance and distinguish aspiration,
  hypothesis and observation. It may develop, retain or revise meaning with justification;
  conjunction, replacement by an easier goal, invented necessity or automatic permission
  does not establish culmination. Independent review has no expected winning phrase.
- Preserve original tests and permissions. A proposed new test needs separately bound
  supplemental-check scope and independent spec-to-check review. A passing finite suite
  is not universal semantic proof. No calibration/maturity or fulfillment label is implied.
- Test restart and uncertainty at each stage. A fresh proposal must use the bound actual
  return/current guidance. Failed/unknown calls consume reservations; no automatic retry.

## Code trace and integration gaps at preparation commit 784b687

These two gaps are now implemented offline; see [B9.1 bridges](engine-b91-bridges.md)
for contracts, checks and the remaining live-validation requirement. The following
paragraphs preserve the original preparation assessment, not current missing-code claims.

`engine/adaptive.rs::observe` and
`expected_failure_preserves_desire_and_separates_contrast` exercise expected failure,
open preferences and separate residual/Contrast. The existing unit test passes, but is
fixture-based and does not establish the exact B9.1 sequence.

`engine/adaptive/coding/project.rs::project_initialize` and project request construction
support immutable source, criteria, inspection and bounded edits. The new offline test
verifies initialization/status restart without target execution. Project acceptance
`guidance.rs::acceptance_guidance` carries operator intent, not generated multi-parent
relational development.

`engine/deliberation/relational.rs::{Proposal,Development,Submission}` carries relational
meaning, parent lineage, revision and source-bound return in the notebook/archive path.
The normal project request/driver does not yet consume that path. Connecting it is an
integration requirement, not a reason to substitute notebook coverage for this coding gate.

`project/acceptance/reviewer/supplemental.rs` already binds independently reviewed finite
checks, but its v1 execution supports non-Cargo Rust probes. The Cargo scenario therefore
also needs a bounded experiment action/receipt bridge before a generated supplemental
experiment can run unchanged through the normal driver. No generator-authored text can
self-admit a test or grant execution authority.

The smallest next engineering step is to connect those existing contracts: project
criterion identities/returns into relational context; generated test proposal into an
independently admitted immutable supplemental check; actual test return into fresh patch
selection. Preserve one controller ledger, current state bindings and existing budgets;
do not build a second scheduler or substitute authored outputs. No new user theory or
permission is needed for offline implementation/review, but it is not completed here.

## Minimal future live validation proposal (not authorized)

After those bridges and their deterministic tests pass, freeze one episode with the
same source/criteria and pinned installed Rust image. Separately admit the flawed seed
execution with its evaluator forecast; retain that attribution. Then allow at most eight
model calls: relational generation; experiment possibilities; experiment assessment;
return-grounded revision; patch possibilities; patch assessment; revision after patch
return; final proposal-only continuation. Maximum three execution groups: seed, independently
reviewed generated experiment, selected patch plus frozen suite. No second patch retry.
Existing 20-second per-command bound remains; freeze aggregate tool/wall/token/patch limits
before dispatch, not after observing costs. Stop on missing independent review, invalid,
unknown, inability or exhausted budget. No action is authorized by this document.

Provider/model, data transmission scope and exact prospective budget need a fresh bounded
admission. Prior synthetic runs are consumed; their grants are not recycled. Candidate
content, not a canned answer, must determine the selected operation. Calls could honestly
stop before eight. This plan cannot run end-to-end yet because of the two explicit bridges.

## Checks

Five preparation tests passed: deterministic preparation, fresh-process validation,
real project initialization/read-only restart, no authority/gate-pass claim, tampering,
forged admission file, missing/symlink inputs and source/evidence separation. Initial
preparation tests caught inappropriate named-library mode and deadline, plus a test
assumption about dependency readiness; corrected without production changes. The existing
Rust expected-failure mechanism test passed (one test). No full Rust sweep was repeated
for Python preparation/documentation; the last complete suite remains975/0/11.
Independent review confirmed scenario fidelity and explicitly identified the integration
limits above. These are preparation and fixture results, not live semantic validation.
