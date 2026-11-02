# Native choice connected to isolated project execution

Later local extension: [bounded relational reconsideration](engine-project-reconsideration.md) connects a new return to one advisory update of an existing aspiration, including when mechanical checks pass. Its explicit fixture-only profile is not covered by the historical live admissions described below.

This local vertical slice replaces the prescribed B9.1 stage sequence with a state-driven boundary around existing native capabilities. It does not replace the selector, executor, preference ledger or acceptance rules. PRD B2/B2.1/B8 and the preserved user correction require relationships and returns to shape possibilities, while original desires and action authority remain traceable.

The ordinary native request contains current attention, standing desires, applicable relational guidance, observations and expectation history. A proposer may offer competing exact edits or already-admitted verification actions. A separate review covers the exact full response. Native policy chooses among the offered eligible alternatives and freezes its commitment; the wrapper does not select a winner. The selected commitment can use the existing sealed executor. Its return is recorded before requesting the next possibilities.

The cycle can request proposals, resume a reviewed unattempted commitment, wait for independent acceptance review, idle, stop or require reconciliation. It does not require a seed failure, experiment or patch in a particular order. Relational/experiment proposals remain review-only: their appearance does not admit a new test command or new preference authority. Advance returns `review_required` for such a batch, and the local wrapper records a durable stop rather than regenerating it. Native request alone does not implement a queue for separate relational/experiment admission. Final goal acceptance remains separate from runtime proposal review.

## Local interface

On an already initialized, explicitly admitted disposable project:

```sh
target/debug/tsr engine --directory "$RUN" project cycle request
target/debug/tsr engine --directory "$RUN" project cycle preview --response RESPONSE.json
target/debug/tsr engine --directory "$RUN" project cycle advance --file REVIEWED_RESPONSE.json
```

Preview validates the response without committing and returns its native digest. The advance input contains `binding`, the unchanged native `response`, and `review` with `binding`, `response_digest`, `state_root`, `source_snapshot`, `reviewer`, `generator`, and `decision: "approve"`. Obtain bindings/digests from current native request/preview; do not invent them or rewrite a generated response to match a later state. Advance records the review attribution and native selection atomically but does not execute target commands.

Only when the returned branch is `resume`, and the caller has admitted this actual execution, use:

```sh
target/debug/tsr engine --directory "$RUN" project cycle resume --binding CURRENT_BINDING
```

This invokes the existing sealed runner for the exact pending native plan. It grants no new command, path or network authority. An uncertain attempted operation requires reconciliation, not another resume. A pending commitment made through an unreviewed legacy path cannot enter this reviewed resume path.

The small `scripts/engine-project-cycle.py` helper supplies durable orchestration with injected native/proposal/review callbacks. It contains no provider transport, credentials or external export. It is an integration interface, not a new approved paid launcher. Every external callback still needs its own reviewed admission, source-limited payload and campaign reservation. Different reviewer/generator strings document attribution; they do not prove independent cognition or correctness.

## Evidence and boundaries

Deterministic fixtures test mechanism behavior, not generated insight or real-world success. Changed guidance can change supplied possibilities without adding an arbitrary aspiration score. Existing native ranking tests isolate observed semantic return, unknown evidence, standing obligations and withdrawal; equivalent inputs must not be described as adaptive improvement merely because a request was repeated.

The completed historical repair trials and their failures remain unchanged. No paid call or isolated target execution is authorized merely by this document. Current campaign usage before this implementation is 181/200 calls, 56/60 execution groups and $0.157617/$5, with no campaign unknowns; the older Garden unknown reservation remains separate. A meaningful live demonstration must fit the remaining 19 calls/four groups, use reviewed open-choice payloads and count generator/reviewer calls together. The B9.1 six-stage admission is not silently reused for this different orchestration.

This slice does not implement arbitrary experiment selection, automatic formal acceptance, new desire weights or mature calibrated learning. Relational guidance remains qualitative, and its semantic usefulness requires assessment beyond deterministic plumbing.

## Live integration boundary at commit 2d44933

The existing paid `RuntimeReviewer` depends on the B9.1 journal's stage identity. It cannot be silently attached to ordinary open-choice requests, and the ordinary native request must not be exported wholesale: it can contain private operator context. A live adapter needs a frozen-suite allowlist for source, original requirements, applicable guidance and attributed finite receipts; a stage-independent review payload; and existing campaign/native reservation and settlement around both callbacks. This is a technical integration boundary, not evidence that current authority or the remaining numerical budget is insufficient. No new live outcome is claimed here.

## Verified results (2026-10-27)

- Six new native cycle tests passed. They exercise alternatives through the existing selector, authored failed/passed worker returns, restart, exact review/source/resource binding, unreviewed pending refusal, withdrawal/closure, unknown execution, and mixed qualitative/executable batch rejection.
- Fifteen pure Python orchestration tests passed. The additional cross-language test passed in 34.773 seconds: actual native CLI request/preview/advance, two authored responses with cfg(test) worker returns, restart between them, exact independent-review-required stop, and unchanged zero-repeat terminal replay. No model or actual target program ran. Production CLI `cycle resume` was source-reviewed, not launched in this test; native `project_cycle_plan` is tested directly.
- The complete CLI library initially had 236 passes, five local-socket `Operation not permitted` failures and eight ignored tests. Supported per-command permission allowed the full rerun: **240 passed, one failed, eight ignored**. The existing `local_transport_isolated_metered_quarantined_and_not_replayable` verifier stub actually took 2,185 ms against its 2,000 ms allowance and was quarantined. The cause of that elapsed-time overrun is unproven; the suite is not claimed clean. No deadline/assertion was weakened and no rerun-until-green was performed.
- CLI binary build, Python compilation and diff whitespace checks passed. Independent review is in `specs/lanes/reviews/engine-native-project-cycle.md`.

The changed-offer cycle test proves wiring, not that feedback caused a better proposal: the offers are authored. Separately, the existing shared-kernel paired test holds alternatives/source/cost fixed and shows negative versus unknown review changing rank, withdrawal removing pressure, restoration recovering it, and native selection consuming that evidence. Existing relational tests cover guidance propagation, parent preservation and stale/withdrawn applicability. These complementary controls do not establish live semantic emergence, causal TSR advantage or mature calibration.

The local tests can be rerun without any provider:

```sh
python3 -m unittest discover -s scripts -p test_engine_project_cycle.py
PATH=/Users/n/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:$PATH CARGO_BUILD_JOBS=2 /Users/n/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin/cargo test --offline -p tsr-cli --lib engine::adaptive::coding::project::cycle::tests
```

The Python native-boundary case intentionally skips unless the existing explicitly built cfg(test) worker is supplied as `TSR_B91_TEST_WORKER_BIN`; that skip is not a pass. It must never point at an arbitrary executable or a production runner. All campaign counters remain unchanged by this implementation turn.

## Fidelity controls and the missing proof

| Claim | Concrete control | Evidence limit |
| --- | --- | --- |
| Attributed return can change native selection | `project.rs::shared_kernel_uses_independent_review_and_preserves_unknown_on_restart` holds source, alternatives and costs fixed; rejected versus unresolved review changes ranking, then native `next` selects the unaffected option. Removing only the learned projection restores the tie. | Existing fixed-alternative control; not new generated possibilities or raw execution-return causality. |
| Active desires affect that choice | The same test withdraws/out-of-scopes the semantic desire, checks bias disappears, restores it and checks identical original options return. Restart with unchanged state preserves options exactly. | Status/obligation control, not evidence that accumulated rich relationships generate useful new possibilities. |
| Relationships and parents survive feedback | `project/relational.rs::relational_reconsiders_independent_semantic_return_without_new_execution` and guidance/attention tests preserve lineage, bind reconsideration to evidence and exclude withdrawn/stale guidance. | Existing context/persistence mechanisms; no causal quality claim. |
| Native choice reaches an execution boundary and returns to choice | New native cycle and cross-language tests use two authored offers and fake worker returns through actual selection, commitment, return and restart. | Second offer is authored. The return does not independently cause it. Production sealed execution is not exercised this turn. |
| Equivalent guidance does not create a fabricated difference | Python `test_equal_guidance_no_manufactured_choice_change` repeats the same deterministic fixture callback. Native shared-kernel controls check identical restart/restored option state. | The Python check is only fixture consistency, not a model or integrated causal control. |

The requested end-to-end evolving-desire behavior remains unproven. The next smallest implementation is the allowlisted open-choice provider/reviewer callback adapter described above, with native/campaign accounting. Then a paired initial-state control and one retained live prefix can test generated possibilities before and after an actual return without scripting the selected trajectory. Any live envelope must reserve generator plus reviewer calls within the remaining 19 calls/four execution groups. No live admission was launched here because that export/review adapter is not yet implemented; there is no newly identified permission denial or numerical-cap blocker.

### Isolated verifier follow-up

At the parent's request, the failing verifier test was executed once in isolation with the same two-second resource limit: **one passed, 248 filtered out**, test process 2.11 seconds (the process duration includes work outside the timed transport). Log: `target/native-project-cycle/verifier-isolated.log`. A first launcher attempt using Homebrew Cargo with `+1.90.0` failed before any test ran; its error is separately retained. The corrected invocation used the installed pinned Cargo directly. The aggregate 2,185 ms transport failure remains unchanged; contention or a harness race is not established by this isolated pass. No further suite rerun was performed.

## Completed transport integration (follow-up)

`scripts/engine-project-cycle-adapter.py` now implements the narrow synthetic-only proposal and independent review callbacks, using the existing Luna transport and campaign ledger plus native reservation/settlement. It pins the transitive helpers, campaign code and binary. It exports only the frozen public requirements, approved editable source, available verification identity, native scoped case evidence and generated relational lineage. Generic initialization contrasts are desire-admission provenance, not test results, and are not exported as observed failures.

A single separately reviewed qualitative proposal can be admitted through the existing native relational API and enter the next request. It does not authorize an experiment, executable command or acceptance. Pure experiment proposals still stop for separate admission. Mixed executable/deferred batches are explicitly forbidden; inability can stop without manufacturing an action. Exact generated responses remain unchanged, and separate reviewer invocations may reject or abstain. Same-model errors can correlate.

Supported explicit entry point (requires already reviewed, source-bound admission files):

```sh
python3 scripts/engine-native-cycle.py check \
  --admission ADAPTER_ADMISSION.json --expected-sha256 ADAPTER_SHA256 \
  --cycle-admission CYCLE_ADMISSION.json \
  --campaign-manifest CAMPAIGN_MANIFEST.json --campaign-upgrade REVIEWED_UPGRADE.json
# Only within the already-authorized live scope, replace check with run.
```

`check` performs local validation and creates only local journal headers; no inference or target command. `run` verifies current public pricing, reserves the complete finite envelope prospectively, and delegates each actual effect to the existing native/campaign accounting. A terminal replay prints the saved result without network/provider or target dispatch. An uncertain intent stops for reconciliation. It does not create credentials or authorize new fixtures/private export.

Five actual-native adapter tests passed with fake provider/worker receipts, plus 18 pure checks. Their fake provider responses and zero target commands are not live performance. New native controls demonstrate that identical offered patch/source with passing versus failing fixture receipts routes to review versus continued proposal; repeated failure and restart remain identical. For a fixed alternative pair, reject versus unknown evidence changed the leading score from 0.34997 to 0.25830333333333333, still above the alternative's -0.4: the winner did **not** change. Withdrawal removes the relevant guidance and returns to review. Do not reinterpret estimate sensitivity as a superior selected action.

The prospective live design is in `engine-native-cycle-live-plan.md`. Initial scope preparation at `target/native-open-choice-1` stopped locally because origin freezing must precede verification-action admission; no provider call or target command occurred. A fresh correctly ordered setup preserved that failed preparation. All earlier experimental scores and histories remain unchanged.
