# Existing pursuit loop: executable local trace

This follows `e3224414` with executable checks, not another proposed subsystem.
No production wiring change was needed. The project-native path already connects
reviewed relational lineage, native forecast/commitment, a worker return,
reconsideration and the next ordinary proposal context. Model behavior at that
connection remains unproved.

## Entry points and dataflow

On an initialized, resource-admitted disposable project (not a current live grant):

1. `tsr engine --directory DIR project cycle request` calls
   `project_cycle_request`; its ordinary request includes current parents, reviewed
   guidance, prior outcomes and frozen expectations. `project_context` supplies
   `project_relational_context` in `coding.relational`.
2. `project cycle preview --response FILE` validates exact root/source and current
   admissible kinds. `project cycle advance --file FILE` records the separately
   supplied review; native `evaluate_proposal`/`next` selects from eligible offered
   alternatives and freezes the forecast before commitment. The wrapper does not
   choose a winning patch.
3. `project cycle resume --binding BINDING`, only for an admitted unattempted
   commitment, obtains `project_cycle_plan` and invokes the existing sealed runner.
   Return ingestion binds the actual result to that commitment. Tests below use
   the existing `Fake` Runner, not this live command or a target process.
4. A later `project_cycle_request` reads actual project-return identities.
   `cycle_reconsideration` detects an existing lineage with new evidence, rather
   than an arbitrary stage counter. An uncertain effect instead routes to
   reconciliation; accepted/closed tasks remain stopped.
5. A separately reviewed `Suggestion::Relational` revision must name the current
   binding and ancestor. `project_relational_admit(root, proposal_id, index)` is the
   native admission API; proposal/review alone does not perform it implicitly.
   It preserves original lineage and acceptance boundaries. The CLI exposure is
   documented in the existing project command interface; do not infer a new grant.
6. `proposal_request` after restart includes the admitted revision in
   `coding.relational.guidance`, with prior outcomes and frozen expectations.
   This is available to generation, not an automatic new action or utility weight.

Sources: `engine/adaptive/provider.rs`; `engine/adaptive.rs`; and
`engine/adaptive/coding/project/{cycle,relational}.rs`. The shared source-bound
qualitative attachment remains advisory. Its `Basis` supports initial context and
retrieval observations, **not arbitrary project outcomes**; do not fabricate a
retrieval basis to import a task return. The existing task-native route avoids
needing that translation or a new schema.

## Reusable offline checks and what they falsify

`public_attachment::tests::attached_relation_and_return_trace_separates_context_from_selection`
runs a 2×2 control: identical authored alternatives with/without reviewed relational
context, crossed with authored met/unmet feedback. Both alternatives claim the same
two fixture obligations. These claims are setup, not independently proven semantics.

The test observes the actual generic engine: forecast precedes commitment; unmet
feedback changes the next choice, met feedback retains it; restart retains outcomes
and frozen forecasts. It checks immutable criteria and unchanged source meanings.
Identical offers yield identical selections with/without the prose attachment.
This explicitly exposes the limit: the relationship does not affect fixed-offer
ranking merely by being stored. A model would have to use it in generating offers.
There is no `Executed` event; the observer input is clearly authored, not a claimed
real outcome. It is return-sensitive selection wiring only.

The existing reusable
`project::cycle::tests::reconsider_revision_propagates_only_after_separate_admission`
now also asserts: no reconsideration before return; reconsideration afterward;
matching candidate forecast precedes commitment; acceptance criteria remain
unchanged; the admitted revision, prior outcomes and forecasts reach the ordinary
request after restart; no action is committed merely by admitting the revision.
The prior checks for wrong ancestry, quarantine before admission and original
lineage retention remain unchanged. The fake worker makes temporary fixture writes
and produces authored receipts; it runs no target executable or model. This is the
complete **local lifecycle** trace, not generated adaptation or real repair evidence.

A regression that ignores discrepancy would fail the first control. A regression
that only stores revision but omits it from subsequent generation context would fail
the second. Neither proves semantic use by a model; a future result that merely
repeats the relationship in prose, without affecting justified options/pursuit,
would fail the behavioral target even if all these tests pass.

## Exact remaining dependency and next live requirements

The missing proof is generated behavior through this same project-native path:
a genuinely proposed joint possibility, a pursuit-relevant frozen expectation,
an authenticated action outcome, reviewed retention/development, and a later freely
generated pursuit. Relationship-only development is allowed; novelty is not forced.
The original parents and external acceptance remain independently assessed. A valid
stop is safe but is not evidence of successful revised pursuit.

No new mechanism is justified by these traces. Use an existing permitted real-action
adapter with multiple possible means; do not add a map-specific action or substitute
read-success for a desire-serving outcome. Freeze held-out inputs and acceptance,
review exact export/action scope and source pins, and reserve the complete run before
launch. Both proposal and semantic-review calls count. A planning envelope for two
choices is eight ordinary generation/review calls (initial relation, first choice,
return-informed revision, next choice, each reviewed), plus explicitly bounded
corrections, and up to two actual execution groups if both choices are actualized.
This is not an admission or a verified launcher budget; final independent outcome
assessment and the actual runner's stopping/review branches must be included before
pinning an envelope. The existing campaign has only two call slots and no operations,
so no such live test fits and none was requested or launched.

For attribution, use the same facts, wants, candidate memory, relational text,
parent links, review judgments and budgets in a plain-memory control. Remove only
native operational treatment. If that amounts only to a wrapper change, call it a
framing comparison. Equal performance leaves TSR-specific advantage unproved.

Campaign remains 218/220 calls, 60/60 groups, $0.176124/$5. Frozen live artifacts and
historical negative results were not modified. No paid call, corpus inspection,
real target execution, credential change or publication occurred.

## Verification

Both focused native tests passed: the four-arm attachment/return control (2.51s)
and the extended project reconsideration trace (20.00s after the final matching-
candidate forecast-order assertion). Installed Rust 1.90, offline, `cargo test
-p tsr-cli --lib TEST_NAME -- --nocapture`. No full-suite pass is claimed.
Independent review by `/root/inquiry_review` found no material blocker and agreed
that no production wiring change was justified. It specifically distinguished
return-sensitive fixed-offer selection, revision propagation, and the still-unproved
semantic effect on generated alternatives. `git diff --check` passed. Existing
assertions and unrelated files were preserved.
