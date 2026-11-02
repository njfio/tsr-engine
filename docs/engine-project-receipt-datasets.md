# Receipt dataset construction and validation

`project-receipt-dataset@v1` is a portable evidence inventory, not a native training
dataset. It contains saved pre-action project rows and enough original CAS evidence
to reconstruct `project-receipt-features@v1`. It does not create labels, maturity,
independent task counts or splits from repeated turns. The numerical bridge below consumes this inventory plus separate prospective
eligibility evidence; the inventory itself never acquires labels.

## Runnable interface

```
target/debug/tsr engine --directory RUN project-dataset --as-of RFC3339 --out NEW_FILE.json
target/debug/tsr engine --directory RUN project-dataset-validate --artifact NEW_FILE.json
```

Export requires a new file and a nonfuture explicit cutoff. No inference runs.
Rows require a previously recorded project shadow prediction. Missing legacy
prediction features or causal side-prefix binding cause an explicit refusal;
export does not reconstruct a forecast after observing its return. An episode with
no saved prediction rows produces an honest empty inventory.

The runtime and validator call the same pure extractor in
`tsr-learn::project_receipts`. Each row binds original main prefix, captured side
prefix, candidate, source version, factual state, declared family and lineage,
schema digest and prediction receipt witness. Feature evidence retains original
CAS bytes and verifies their hashes and event identities. Post-action/review data
cannot supply feature values. Prediction event availability and embedded feature
cutoff are checked separately. Later side records cannot enter an earlier row,
even within the same clock millisecond. Missing context remains null; unknown test
outcomes remain unknown counts. Issued context still does not prove inspection.

`export_time_lifecycle_inventory` is explicitly current export-time metadata,
separate from the as-of row evidence. It preserves closure/mapping/resource facts
for inspection, never as features or labels. Original histories are untouched.
Artifact digests establish content consistency, not authenticated origin or honest
operator statements. Preserve these local artifacts outside candidate workspaces;
they contain scoped source/receipt data and are not publication artifacts.

## Machine-enforced boundaries

- Schema and content binding, CAS integrity, prediction provenance, duplicate row
  identity and bounded family/lineage metadata are checked on load/restart.
- `eligible_rows` must remain zero. Current project closures have no admitted
  native common-event label. An accepted review cannot silently become one.
- `require_fit_eligible` refuses these inventories. They cannot be passed off as
  existing `rows.jsonl`/maturity datasets. No synthetic fixture meets a support floor.
- Shared schema dispatch validates native and receipt specifications. Renaming a
  native artifact to the receipt schema cannot enable prediction. Receipt models
  require reconstructible maturity and feature evidence. Shadow predictions remain
  isolated from selection and authority.
- The numerical bridge applies maturity/as-of, clone/lineage splitting, overlap,
  leakage probes, support and artifact checks to receipt-derived rows.

## Verification

The existing project initialize→proposal→execution→independent-review lifecycle
integration test now exports, reconstructs and reloads real adapter records with a
deterministic fixture runner. It tests restart, unchanged factual roots, missing
CAS, modified counts, incompatible schema, causal side-prefix extension and
historical receipt unavailability. These are mechanics tests, not production data.
Pure extractor tests preserve legacy missingness, reject altered CAS/future
receipts and exclude review outcomes. Independent review found the side-prefix and
receipt-time defects; both were fixed before handoff.

Checks passed: CLI library 119 passed/one existing ignored; tsr-learn 135 unit
tests plus one integration test; clippy (CLI/learn, all targets, warnings denied),
CLI build and diff whitespace checks. The preserved RTS history copy exports zero
saved prediction rows and zero eligible rows; validation succeeds without inventing
historical forecasts. Evidence: `target/project-dataset-history-{export,validate}.log`.

Logs: `target/project-dataset-cli.log`, `target/project-dataset-learn.log`,
`target/project-dataset-clippy.log`, `target/project-dataset-build.log`.

## Numerical bridge and prospective eligibility

Shared `tsr-learn::schema` dispatch now serves dataset construction, reconstruction,
leakage probes and pre-action shadow application. Original artifact rows and closure
labels remain unchanged. A separate evidence envelope binds the original admission,
source/side events and CAS, reviewed closure, resource accounting and review cutoff.
Unknown numeric context cannot become zero. Unknown test outcomes retain a separate
count. Unchosen alternatives remain evidence, never negative training examples.

For a **new** episode, `project-lifecycle-open --admission FILE` accepts optional
`training` metadata alongside existing state root/tool/non-tool budgets:

```json
{
  "origin": "operational",
  "original_task": "<original task content hash>",
  "family": "<named event family>",
  "lineage": "<source lineage>",
  "clone_group": "<independent clone grouping>",
  "reviewer": "operator",
  "no_inference": false
}
```

This object belongs under `training`; it is not a standalone lifecycle admission.
Use `synthetic` or `replay` origin for those sources: they are permanently excluded
from operational labels, even after the maturity horizon. `no_inference` is an
explicit claim checked against the ledger, not a way to hide unmeasured calls.
Legacy episodes cannot receive a retrospective prospective admission.

After the usual actual execution, independent semantic review and closure:

```sh
target/debug/tsr engine --directory RUN project-eligibility-view
# Write REVIEW.json from independent inspection: closure digest from the view,
# reviewer, independence explanation, complete_instrumented_coverage=true.
target/debug/tsr engine --directory RUN project-eligibility-review --review REVIEW.json
target/debug/tsr engine --directory RUN project-training-evidence --as-of RFC3339 --out NEW_EVIDENCE.json
target/debug/tsr train receipt-dataset --artifacts EVIDENCE_A.json EVIDENCE_B.json EVIDENCE_C.json EVIDENCE_D.json --as-of RFC3339 --out NEW_DATASET
```

The builder uses repo/clone-group splits (25% calibration, 25% holdout, seed zero),
requires nonempty sides, runs all leakage probes before writing, and writes the
normal dataset layout plus digest-bound `feature-evidence.json`. Existing `train
fit`, `verify`, `report` and `apply` consume that layout. Fitting still requires the
existing 150 mature fit/calibration episodes and family/range support gates; the
four-file example illustrates file syntax, not sufficient production support.

Eligibility checks original source-to-projection mapping, frozen precommit prefixes,
all commitments executed, review/source/snapshot bindings, full measured resource
coverage, limits, unique original tasks, declared clone groups and close-plus-seven-
day maturity. Missing/unknown/contradictory evidence withholds a label or refuses
malformed evidence. Review must exist within the allowed horizon, not be backdated.
Operator statements and content hashes are not authenticated proof of independence
or completeness; an accountable independent reviewer remains required.

The real-adapter regression supplies two alternatives in each of four synthetic
fixture episodes, executes one, independently records fixture review, closes,
exports and rebuilds the dataset after restart. Eight prediction receipts produce
four unlabeled training rows. Synthetic origin stays ineligible at a distant future
cutoff; fitting refuses. Tampered evidence, clone metadata, duplicate proofs,
missing extraction input and unresolved commitments are rejected. This verifies
plumbing, not production calibration. No model inference was used.

## Remaining data dependency

The preserved RTS history lacks prospective admission/saved prediction rows and
remains ineligible; its past success is not rewritten into a forecast. The 151
qualified corpus tasks remain distinct from independently reviewed mature episodes.
Their mature eligible count is unknown. To fit a usable receipt model, collect
prospectively admitted operational episodes with original identity/clone grouping,
pre-action receipt features, complete measured budgets, actual independent returns
and timely independent reviews, then wait for the actual seven-day horizon and
construct disjoint data at an explicit cutoff. No operational model was fitted or
calibration claim made here. Useful bounded decision operation continues with its
explicit weak-support fallback; provider generation limits remain separate.

Current checks/evidence: `target/schema-{cli-tests,learn-tests,clippy,build}.log`,
`target/schema-cli-doc.log`, `target/schema-cli-full-tests.log`, and
`target/schema-history-{view,export}.log`. Full CLI integration includes existing
M0 runtime-sensitive failures and legacy M4 tests omitting the required `--as-of`;
these are not a clean full-suite pass. See the current handoff for exact totals.

Verification for the numerical bridge: CLI library 120 passed/one existing ignored;
learn 136 unit tests plus one integration test passed; CLI doctests passed; affected
all-target clippy with warnings denied and CLI build passed. The broad CLI run
completed 356 passing and 10 failing tests: two documented M0 runtime-dependent
failures and eight M4 tests omitting `--as-of`, already required at base 07583cd.
Those four test files are unchanged. The broad doctest invocation lost an rlib during
concurrent rebuild; a serialized doctest rerun passed. The 87-test M4 training suite
passed. No protected tests were edited to change those results. Independent review
caught and verified fixes for projection/clone/review bindings, unfinished execution
and unchosen alternatives; its final review found no remaining blocker.
