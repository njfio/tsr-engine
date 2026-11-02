# Runtime review bridge for real project stages

`scripts/engine-project-runtime-review.py` supplies a callable reviewer for the existing B9.1 `Driver.reviewer` boundary. It reuses the real project controller and source-bound projection, shared campaign ledger, structured transport validation and bounded rejection diagnostics. It does not add a new executor or change the default `FileReviewer` workflow.

The optional adapter can review initial/return relational guidance, the generated supplemental experiment, proposed patch scope, and readmission of the unchanged experiment against the new source. It requires a separate admission bound to the exact generator admission, campaign manifest, implementation and criteria. Its maximum is six reviewer calls, each 8,192 completion tokens, 240 seconds and 32 KiB. These calls are additional to the generator's six-call allowance and must count against the same campaign. No live reviewer calls were authorized or made for this implementation check.

The reviewer sees frozen requirements, editable synthetic source, the generated candidate, parent/relational context, case identities and typed actual test evidence. It does not receive protected test bodies, raw logs, credentials or private operator reports. The existing projector rejects content outside the frozen B9.1 source lineage. This is not arbitrary repository export support.

Review findings distinguish static source analysis, observed test-receipt citations and unknowns. Accept requires complete met coverage; reject, unknown, abstain, malformed output, source drift, STOP, expiry or uncertain usage/dispatch stops. A receipt citation establishes evidence identity, not the truth of every natural-language claim. Separate calls to the same model may share mistakes. Stored review provenance and exact request/outcome bindings are checked on restart; an unresolved dispatch is never repeated automatically.

Final `semantic_coverage` delegates directly to the existing independent `FileReviewer`, without an extra model gate. A weak optional final proposal therefore cannot veto a sound repair. The adapter does not grant execution permissions, modify protected criteria or make a model verdict into formal native acceptance. The B9.1 driver's phase order also remains fixed; this work does not create autonomous planning over arbitrary coding tasks.

## Integration boundary

After separately reviewing a concrete admission, a caller can supply the adapter through the existing callback:

```python
review_admission = project_review.admission(driver.admission, ledger.manifest)
driver.reviewer = project_review.RuntimeReviewer(
    driver, review_admission, ledger, separate_review_transport, final_file_reviewer
)
```

Here `project_review` is the loaded module, `driver` is the existing B9.1 driver, `ledger` is the existing approved campaign, and `separate_review_transport` is an explicitly supplied bounded transport. This is an API sketch, not permission or a live launch command. The module constructs no provider connection. The default production CLI behavior remains unchanged. `engine-b91-campaign.py run` enables this adapter only when both `--runtime-review-admission PATH` and `--runtime-review-admission-sha256 HASH` are supplied, alongside the existing exact generator/campaign admission and effect opt-ins. It validates the review admission before metadata lookup or enrollment. These flags are rejected outside `run`; a missing or mismatched pair fails closed. Final formal review still uses the existing `--reviews` directory.

Prepare the separate review admission without dispatch using:

```sh
python3 scripts/engine-project-runtime-review.py \
  --driver-admission PATH_TO_DRIVER_ADMISSION \
  --campaign-manifest PATH_TO_EXISTING_CAMPAIGN_MANIFEST \
  --output PATH_TO_NEW_REVIEW_ADMISSION
```

Preparation emits the exact review-admission hash; it does not grant permission or start a provider connection. Live use requires review of that concrete admission and sufficient shared campaign headroom for up to six generator calls plus six reviewer calls and four execution groups. Do not retrofit activation into an already-dispatched historical trial.

`scripts/test_engine_project_runtime_review.py` is the runnable offline example. It uses the real native controller, authored provider/reviewer fixtures and the existing cfg(test)-only worker. It exercises the actual callback sequence, persistent state and restart without provider calls, compilation/execution of generated target code, or claims of live semantic performance. The hand-authored repair in that test is fixture machinery only and never replaces a live engine proposal.

Verified on this checkout with the already-installed test worker:

```sh
TSR_B91_TEST_WORKER_BIN="$PWD/target/debug/deps/tsr_cli-62904bb7a7cdafee" \
  PYTHONPATH=scripts python3 -m unittest test_engine_project_runtime_review -v
```

All **11 tests passed in 87.182 seconds**, with no skips. They cover the full six-generator/four-test-worker sequence, six automatic review callbacks and separate final review; persistent replay without duplicate review calls; rejection/unknown, stale or ungenerated source, malformed reports, wrong receipt citations, missing/invalid usage, budget/campaign mismatch and interrupted review reservations. Earlier tests exposed an exception-class import mismatch and missing review context; the final run includes their repairs. No full workspace pass or live project-review competence is claimed. If the test-worker binary is absent, the test reports a skip rather than silently running target code; the recorded result above used the actual existing worker.

[Independent source review](../specs/lanes/reviews/engine-project-runtime-review.md) found no remaining material blocker in this opt-in frozen-scenario scope. It is a source review, not evidence of model competence or a new live admission.

## Remaining boundary

Trial 17's third generator response failed evidence binding before it could reach any semantic review. This bridge does not excuse or silently repair that failure. It removes the need for operator-written intermediate review files when explicitly integrated, but its live review competence is untested. Final formal acceptance remains independently supplied. The next meaningful live proof would test this new intermediate-review path on the same bounded real repair, rather than repeat the old operator-assisted sequence; no such extra run was made here.
