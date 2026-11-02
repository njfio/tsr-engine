# Shared bounded decision kernel

[Ongoing-episode observation revisions](engine-observation-revisions.md) now reopen
coverage after admitted source content changes, preserving wants and historical
evidence. A genuine local continuation restored current coverage in three calls.

[Latest live evidence after the capacity fix](engine-capacity-live-evidence.md):
five local calls, original wants preserved, complete operational satisfaction and
identical restart state. This remains bounded coverage/brevity evidence.

[Post-migration live verification](engine-shared-live-verification.md) now confirms
the generated decision/return/restart cycle. The bounded run exhausted ten local calls
with one original desire incomplete; live operation is not reliable goal satisfaction.
The extraction-only evidence below remains a historical record.

The notebook and scoped-project paths now call the same pure assessment in
`tsr-domain::decision`. The kernel does not execute actions, admit preferences,
verify source truth, or manufacture outcomes. Existing `PreferenceId`, `EventId`,
`Candidate`, forecast/commitment records, independent reviews and executor boundaries
remain in use. No second event store, supervisor, task queue, or provider protocol was
introduced. `Domain`/`Council` remain the domain and generation contracts; the new
module is reusable decision arithmetic, not another competing adapter trait.

## What is shared

- Explicit obligation identity (`PreferenceId`) and separately declared refinement
  relationships. Only controller-admitted acyclic chains participate. A generated
  subset inherits its original's weight; it does not become a new independent value.
- Balanced prospective progress, losses across all standing obligations, set-based
  commitment continuity, and explicit effort. The numeric rule is the existing
  engineering proxy, not a claim to derive quantitative laws from TSR.
- Evidence-weighted expectation over actual return receipts. Unknown observations
  remain `None`; duplicate evidence is rejected; unobserved results are not failures.
- Inspectable assessment with original observations, selected obligations, projections,
  probability, effort, losses, continuity cost, and unresolved obligations.

Unknown current satisfaction uses an **optimistic zero planning baseline**. This is
explicitly recorded as an assumption, not neutral uncertainty handling or evidence
of failure. Positive planning value cannot establish satisfaction. A domain needing
robust worst-case decisions must supply appropriate independent bounds or decline the
action; the kernel does not infer that risk policy from a generated explanation.

The module accepts controller-provided authorization for assessment. That means only
that the proposal is in admitted planning scope. Existing execution gates and accepted
export checks remain mandatory. A kernel result is never an approval token.

## Adapter responsibilities

The notebook adapter reconstructs the actual ledger preference identities, supplies
measured note coverage/word budget, bounds known action effects, and transfers return
residuals using operation/topic/context similarity. Generated subsets remain hints for
subsequent direction generation, not duplicate reward. Its satisfaction remains
coverage and brevity; copying notes establishes neither understanding nor truth.

The existing project adapter supplies original test preferences plus independently
admitted semantic requirements. Current test results are usable only when their
receipt binds the current source snapshot. Semantic observations require the current
review binding. Stale or unresolved reviews are unknown. Omission of a test from a
proposal's support list preserves its known observation; omission alone is not evidence
of damage. Real execution can still contradict that forecast and reopen desires.

Semantic expectations fold independent reviews, not proposer claims. Exact actions
have full relatedness; new patch identities transfer evidence by edited-file Jaccard
overlap. Unrelated local files receive no transferred evidence. Imported review history
without action context receives weak family evidence (weight 0.25). These are coarse,
explicit adapter assumptions, not a claim of semantic similarity. Superseding reviews
replace their receipt's label in the fold; disputed/unresolved reviews do not multiply
or fabricate observations. Detailed kernel observations stay in operator-facing state; provider requests retain
ranking estimates but omit the new per-obligation decision trace. Private requirement text and private review explanations
are not added to model requests by this change.

Project option ranking now uses the shared kernel. Recorded commitment utility uses
that kernel's gain and cost, rather than the old criterion count. Test forecasts and
independent acceptance forecasts remain separate. The existing generic configured
menu adapter is unchanged; it is not evidence of this extraction's endpoint.

## Run and inspect

From the repository root, with the already installed toolchain:

```sh
/Users/n/.cargo/bin/rustup run 1.90.0 cargo build --offline -p tsr-cli --bin tsr
# Choose fresh output directories; no model or execution is invoked here.
target/debug/tsr engine --directory target/shared-notebook-example direction-init \
  --spec examples/engine-directions/standing-wants.json
target/debug/tsr engine --directory target/shared-notebook-example direction-request

target/debug/tsr engine --directory target/shared-project-example project init \
  --spec examples/engine-invoice/project.json
target/debug/tsr engine --directory target/shared-project-example state
```

Notebook `deliberate --response FILE` records `.turn.offered[].shared` assessments.
Project `state` records `.options[].decision`; `project request` produces the existing
bounded proposal contract. `project run --response FILE` assesses alternatives and
uses the existing scoped executor. `project acceptance review` supplies independently
reviewed evidence, and accepted-only `project export` remains guarded. See the existing
[operator quickstart](engine-local-quickstart.md) for review, correction, restart and
export commands. Do not run its provider/execution steps without the applicable local
resources and authority. No live inference was performed for this extraction.

User inputs remain: standing desires and their operational criteria; admitted evidence
or repository scope; protected tests and execution limits; independent semantic review
where machine observations cannot decide correctness; and separately approved action
and inference budgets. Generated possibilities can decompose these obligations.
New independent values or expanded authority remain unresolved without admission.

## Deterministic evidence and boundaries

`cargo test --offline -p tsr-cli -p tsr-domain --lib` exercises both adapters and shared
invariants. The project integration test uses the existing bounded fixture runner and
a separate review record; it does not claim newly performed model discovery or a real
production repair. It shows a rejection changing scores for a new related patch,
changing selection relative to an unrelated alternative, and reaching the next actual
commitment. A counterfactual reruns the same kernel with identical observations/costs
and only the learned semantic estimate removed. Restart and unchanged controls agree;
planning never marks the project accepted.

Notebook controls preserve decomposition/permutation invariance, contrary-return
learning and prospective loss. Shared tests reject cyclic/unadmitted refinements and
unauthorized actions, preserve unknown observations, reject duplicate receipts and
check contradictory evidence. Independent review found an invented-loss bug for
unserved tests; its new regression test failed before the fix and passed afterward.

This delivers a reusable bounded decision component connected to two existing
operational adapters. It does not demonstrate autonomous admission of fundamental
values, propositional world belief, general semantic verification, or an unattended
cross-domain application. Direction generation is still supplied through existing
provider interfaces; this change tests replayed alternatives, not new live generation.

## Verification record

- Final `cargo test --offline -p tsr-cli -p tsr-domain --lib`: **77 CLI + 12 domain
  tests passed**, including the final receipt/criterion-binding and provider-redaction
  corrections. Log: `target/shared-final-checks.log`.
- Final workspace/all-target Clippy with `-D warnings`: passed.
  Log: `target/shared-final-clippy.log`. Changed-file formatting and diff checks passed.
- The broader workspace sweep (before the final recovery/admission/privacy refinements)
  produced **796 passed, 4 failed, 5 ignored**. Two are the documented M0 runtime-dependent
  failures. Two untouched corpus-mutant fixture tests failed to apply their patches.
  The isolated serial corpus-mutant suite then passed **29 tests, 1 ignored**, including
  both failures. Their broad-run cause is not established. Logs:
  `target/shared-workspace.log`, `target/shared-mutants-recheck.log`.
- Both documented fresh initialization/read commands completed at
  `target/shared-notebook-example` and `target/shared-project-example`; JSON receipts
  are `target/shared-notebook-request.json` and `target/shared-project-state.json`.
- Independent read-only review cleared the corrected identity/admission, source/receipt/
  commitment/criterion binding, and shared scoring. No live or paid inference, external
  transmission, push, deployment, credentials, or original target-repository edits.

The full workspace result is not reported as green. The final affected library and
lint checks cover the final code; the broad sweep's limitations remain visible.

[Follow-up differential and concurrency investigation](engine-mutant-failure-review.md)
qualifies the two corpus-mutant failures against untouched baseline `31859c2`.
Three eight-thread runs passed on each revision; the original broad-run cause remains
unresolved and is not labeled harmless or environmental.
