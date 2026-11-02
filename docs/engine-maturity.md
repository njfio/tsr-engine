# Common-event maturity and evidence cutoff

New dataset construction and production fitting enforce PRD B7's seven-day
horizon. This is an evidence-correctness change, not a calibration result.

```sh
cargo +1.90.0 build -p tsr-cli
# Supply an actual declared evidence cutoff, including its timezone.
target/debug/tsr train dataset --events "$EVENTS" --tasks "$TASKS" \
  --labels "$REVIEWS" --as-of "$AS_OF" --by time --out "$NEW_DATASET"
target/debug/tsr train fit "$NEW_DATASET" --out "$NEW_BUNDLE"
target/debug/tsr train verify "$NEW_BUNDLE" --rebuild "$NEW_DATASET"
```

Inputs remain the event log, task/family/lineage/clone index, and explicit operator
review records. `--as-of` is required for maturity validation and subsequent fitting;
no ambient clock silently changes the snapshot. With a cutoff, construction prints
eligibility counts and writes `maturity.json`,
original `events.jsonl`, `tasks.jsonl`, and `labels.jsonl`. The manifest binds their
canonical digests and the cutoff/policy record. These are operator-provided
provenance assertions, not authenticated timestamps or independently verified
semantic acceptance.

## Versioned policy

`task-close-plus-seven-days@v1` applies to `task_accepted_in_budget@v1`, not notebook
progress, qualification checks, or standalone final-oracle results.

- Recorded events after `as_of` do not supply features, closure or outcomes.
  Sequence remains causal order; duplicate/non-increasing sequences, changed
  event identities and commitments after task closure are rejected.
- A reviewed task remains `pending_horizon` until exactly closure + 604800 seconds.
  RFC3339 offsets and fractional seconds are compared as instants. Invalid clocks,
  unknown offsets and unsupported leap-second notation are refused, not guessed.
- Review must be recorded by both the cutoff and the fixed horizon. A late review
  remains preserved but does not retroactively reopen this horizon's censored
  outcome. That explicit operational choice would require a new policy version to
  change; the PRD does not define late-review reopening.
- Unreviewed or externally stopped outcomes remain censored; open tasks remain
  open. Budget breaches within the horizon still affect the common event.
- Features and leakage probes use the same available-event view. Fitting reruns
  probes, then reconstructs eligibility, labels, row identities, statistics and
  provenance from retained inputs before counting support or fitting a model.
  Full inputs remain preserved even when future/late evidence was excluded.
- Fit/calibrate retain distinct-task counting, existing split/clone boundaries,
  the 150-task gate and 30-per-family population gate. Those counts do not certify
  independence or absence of prior exposure. Real-corpus mappings and review
  provenance are still required.

The maturity digest is part of new bundle identity. Legacy datasets and bundles
remain readable/hash-verifiable; CLI historical application/reporting warns that
maturity is unknown. New fitting and `verify --rebuild` from legacy datasets refuse
until rebuilt into a new directory with original evidence and an explicit cutoff.
No historical fixtures or receipts were rewritten. Existing numerical unit tests
explicitly exercise the private legacy numerical fitter; the production entry
point always enforces maturity. Rebuild tests use the unchanged synthetic M4 corpus
with a declared simulated cutoff, and retain their original assertions.

## Verification

Results: 128 learning-crate tests, 87 CLI pipeline tests and six inventory tests
passed. Clippy passed for both crates and all targets. `bash -n` passed for the
updated synthetic evidence script; the full historical evidence campaign was not
rerun. No protected-test assertions or historical fixture bytes changed.

Commands and local logs:

- `cargo +1.90.0 test -p tsr-learn --lib` — `target/maturity-learn-tests.log`.
- `cargo +1.90.0 test -p tsr-cli --test m4_train` — `target/maturity-cli-tests.log`.
- `cargo +1.90.0 clippy -p tsr-learn -p tsr-cli --all-targets -- -D warnings`
  — `target/maturity-clippy.log`.
- `python3 -m unittest discover -s scripts -p test_engine_evidence_inventory.py`
  — `target/maturity-inventory-tests.log`.

Focused tests cover just before/exactly at the horizon, offset equivalence, leap
dates, future closure/review/observations, late review/budget evidence, operator
censoring, restart/reload equivalence, legacy fitting refusal, premature-label
forgery despite updated digests, malformed clocks/identity, and ambiguous sequence.
Independent review found and prompted fixes for probe-snapshot mismatch,
post-close outcome credit and duplicate sequences.

## Local generation batch reconciled

`target/support-live/run/local-directions.jsonl` contains nine starts and nine
terminal records: four returned and five rejected. Calls 1/3 performed a genuine
inspection and composition; calls 2/4 proposed refinements. Calls 5–9 did not
establish further progress. Call 8 repeated mechanically unsupported progress;
call 9 again supplied an invalid prerequisite, leaving fewer than two valid
alternatives. No additional call was dispatched. Driver session 7913 exited 0;
temporary Ollama PID 29836 was terminated and port 11435 had no listener.

The last accepted state remained
`3d736f20dfa9683eb7b1d6b0c7d1cf2589fdb4ccc16bf4950e01188f1bf51c4f`.
A fresh `direction-request` read confirmed that root, status `deliberate`, and
observation `[0.5, 0.8, 0.0]` in `target/support-live/final-state.json`. Accepted
direction returns occurred under the earlier v7 policy; no accepted live v8
tradeoff was demonstrated in this batch. Original coverage/brevity obligations
remained unmet rather than being weakened.
This supports rejection integrity and a bounded partial live cycle, not successful
coverage recovery or live verification of every standing-loss case. Deterministic
standing-loss cases were verified separately in the preceding fallback change.
Coverage and brevity are mechanical predicates, not truth or understanding.

The inventory still contains **151 qualified corpus tasks**. Fixed12's three audited
final-oracle outcomes do not establish 150 mature independent common-event episodes.
The eligible real episode count remains unknown. No paid inference, source export,
new model download, original-target mutation or publication occurred in this pass.

Next core work: inspect why the local proposal/revision path repeats structurally
invalid prerequisites after precise rejection feedback, and improve recoverable
contract handling without authoring an answer or relaxing standing wants. New live
verification needs another explicit bounded local batch; this batch is exhausted.
Mature calibration additionally needs the explicit episode/review/lineage mapping
listed in `engine-evidence-reconciliation.md`, not another invented success label.

## Legacy CLI compatibility repair

Omitting `--as-of` now preserves the original dataset-inspection interface. It
builds a diagnostic snapshot from exactly the supplied content, runs the original
leakage probes and preserves events, task index and review records. It prints
`maturity=unknown as_of=unspecified fit_eligible=false`, warns on stderr, and writes
`snapshot-status.json` with source digests. There is no implicit current time,
maximum-event-time guess, added seven-day interval, or maturity proof. Historical
resolved labels remain review assertions for inspection/probes, not mature support.
Production fitting still refuses the snapshot; changing its explanatory status
file cannot grant eligibility. Supply an actual explicit cutoff in a new output
directory to enter the existing maturity-validation path.

The required-argument change in 6d1ce76 did introduce an avoidable CLI compatibility
regression for inspection. Six formerly failing assertions now pass unchanged:

- `m4_corpus::two_real_sessions_pair_each_commitment_with_its_own_generator_forecast`
- `m4_corpus::two_real_forecasts_pair_both_with_the_commitment`
- `m4_probes::a12_dataset_refuses_a_leaking_split_before_writing`
- `m4_probes::a6_fit_refuses_a_forged_split_json`
- `m4_probes::b3_fit_refuses_a_dataset_missing_events_jsonl`
- `m4_probes::b3_fit_refuses_a_dataset_missing_tasks_jsonl`

Two original assertions remain incompatible with the evidence contract:
`m4_corpus::b7_train_apply_prints_raw_and_calibrated_p_and_support` (line 445)
and `m4_corpus::by_repo_seeds_1_to_8_give_three_non_empty_sides` (line 570).
Both now build the dataset successfully, but require **fit success without any
cutoff**. PRD B7 fixes the common event horizon at task close plus seven days;
recorded resolved labels alone do not establish availability at that horizon.
No implementation can preserve both unconditional fitting of this input and
mandatory reproducible maturity evidence. No test, fixture or fitting guard was
changed to hide the conflict.

The precise decision requested is permission to migrate only those two synthetic
test dataset invocations to an explicit simulated cutoff (the existing `m4_train`
harness already declares `2028-01-01T00:00:00Z`), preserving all assertions and
fixture bytes. This verifies their original numerical/split behavior under the new
contract, not production eligibility of synthetic evidence. Without that permission,
the two failures remain explicit intentional incompatibilities. Disabling maturity
or silently inventing a cutoff is not proposed.

Evidence: `target/compat-legacy-tests.log` (10 passed, two fit refusals across these
two test targets), `target/compat-learn-tests.log`, `target/compat-cli-tests.log`,
`target/compat-clippy.log`, `target/compat-build.log`. The new deterministic test
checks byte-identical diagnostic exports/reload, refusal before model creation,
status tampering, and refusal to overwrite existing output. Independent review
found no fitting bypass and confirmed the two semantic conflicts.

Affected aggregate results after repair: learning 137 unit tests and one integration
test passed; CLI library 120 passed/one existing ignored; M4 training pipeline 87
passed. The unchanged corpus/probe targets total 10 passed and the two documented
fit-without-cutoff failures. All-target clippy with warnings denied, build and diff
whitespace checks passed. No claim of a fully green broad suite is made.
