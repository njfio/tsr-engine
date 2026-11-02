# Explicit support-gated Tier 2 plumbing

Default operation remains the existing bounded decision process and shadow diagnostics.
No operational policy was enabled, no provider ran, and no dataset acquired production
eligibility in this implementation. An opt-in requests validation on each decision;
it does not certify a model or expand any action permission.

## Gates and operational choices

`engine/tier2.rs` stores versioned operator policy in the main append-only history.
Unknown versions, non-operator provenance, stale updates and changed CAS are rejected.
Absent/rolled-back policy reads no model artifacts and creates no selection records.
An enabled policy calls the shared shadow extractor with freshly frozen inputs; it
never trusts a cached shadow result's status or probability.

Only the project receipt schema and `task_accepted_in_budget@v1` are compatible.
Notebook coverage and generic adaptive actions have no matching operational common-
event semantics, so their ordinary fallback remains in effect. Project gates require:

- Prospectively admitted operational episode; matching family/lineage; current source
  snapshot; no open execution; complete known resource accounting.
- Already admitted, positive-score candidates with a shared conserved-obligation
  assessment and no unknown obligations. No model prediction can add an action.
- Fresh scope/patch checks, prepared dependency availability, and conservative remaining
  patch/tool/wall bounds. Execution retains its independent durable reservation and
  original scope checks; a score is never an execution grant.
- Pinned bundle, schema, dataset, maturity policy/cutoff and content integrity; original
  operational eligibility reconstructed for every labeled row. Synthetic/replay and
  legacy unknown evidence cannot acquire an operational label.
- No query task, lineage, clone group or original-task overlap with the training or
  evaluation evidence; all existing leakage probes rerun on the same cutoff view.
- At least 150 distinct mature fit/calibration tasks, 30 family tasks, and matching
  feature-range support at the bundle floor (minimum 30).
- Recomputed held-out paired controller-versus-generator validation, pinned by report
  digest and fixed report seed zero. Both overall and query-family paired differences
  must have at least two lineage clusters and a finite 90% bootstrap interval entirely
  below zero. This conservative **v1 engineering gate** is stronger than merely
  observing a lower Brier mean. It is not a TSR theorem, causal superiority claim,
  guarantee of future accuracy or permission to tune on the holdout.

All gates must pass for the full considered candidate set. Missing evidence yields
an inspectable fallback receipt, never a zero probability or fabricated calibration.
Passing gates yields `p * conserved_net_gain - effort - continuity_charge`, then the
existing margin/exploration selector with its recorded propensity. Net gain already
contains standing-want losses; they are not added back. The factual continuity cost
is preserved. Nonpositive choices are removed; if none remain or a near-tie abstains,
no commitment is made. Common-event task acceptance probability is an operational
weight on the existing obligation value, not a calibrated forecast for each individual
obligation. Proxy-history counterfactuals are not mislabeled as interventions on an
activated model. The gate audit is linked from the commitment evidence.

## Forecast collection prerequisite now supported

`tsr-proposal@v1` optionally accepts `common_event_forecasts`, mapping canonical
proposal indexes (JSON strings `"0"` through `"3"`) to actual supplied probabilities.
Missing entries stay absent; invalid indexes/probabilities and ambiguous duplicate
action forecasts are rejected. A recorded provider response precedes selection. Only
the chosen action's supplied probability is frozen as a Generator common-event forecast
immediately before commitment. Unchosen predictions remain in the proposal receipt,
not negative outcomes. No probability is inferred from rationale or test success.

The installed local transport `scripts/engine-local-model.py` accepts optional
`common_event_probability` alongside its existing edits/rationale and transports it
unchanged to proposal index `"0"`. The schema does not require it. No new call was made.
The v2 project lifecycle projection includes these common-event forecasts and validates
source/candidate/chronology. Saved v1 closures retain v1 reconstruction; unknown versions
are refused. The selected forecast survives the shared dataset pipeline, so future
honestly collected episodes can provide the paired baseline. Historical missing
forecasts cannot be reconstructed after returns.

## Operator interface (not executed on an operational run)

Generate and independently inspect a report from an eligible bundle/dataset:

```sh
target/debug/tsr train report BUNDLE --dataset DATASET --seed 0 --json > REPORT.json
target/debug/tsr train report-pin --report REPORT.json
```

`report-pin` returns a canonical content digest and `validated:false`; it does not
approve the report. Validation is recomputed from evidence at selection time.
An explicit update file has this shape:

```json
{
  "state_root": "<current engine root>",
  "reason": "<operator's scoped activation rationale>",
  "enabled": {
    "version": "supported-common-event-selection@v1",
    "prediction": {
      "bundle_directory": "<local pinned bundle>",
      "bundle_id": "<bundle id>",
      "dataset_directory": "<local evidence dataset>",
      "event": "task_accepted_in_budget@v1",
      "family": "<current admitted family>",
      "lineage": "<current admitted lineage>",
      "training_as_of": "<actual RFC3339 cutoff>"
    },
    "validation_report": "<report_digest>",
    "selection": {"margin": 0.1, "epsilon": 0.0, "seed": 0}
  }
}
```

Margin/epsilon/seed here illustrate explicit operator inputs, not recommended learned
values. Configuration pins the current episode metadata. A subsequent episode with
changed metadata requires a fresh operator update; nothing silently broadens scope.

```sh
target/debug/tsr engine --directory RUN tier2-configure --update UPDATE.json
target/debug/tsr engine --directory RUN tier2-view
```

Rollback uses the same command with current `state_root`, a reason, and `"enabled":null`.
It does not delete prior configuration, receipts or commitments. It affects future
selection, not already authorized or dispatched actions. No always-on service is added.

## Evidence and remaining limits

Deterministic checks cover disabled noninterference, unsupported adapters, restart,
rollback, forged policy provenance, version/state rejection, conservative rescoring,
post-gate selected probability/EV/propensity/audit linkage, and optional forecast
transport. The actual adapter regression uses two explicitly synthetic forecasts,
executes a fixture action, independently records fixture review, exports/rebuilds and
checks the chosen generator forecast. Synthetic origin still excludes every label.
This is mechanics evidence, not a successful production activation or evidence that
any tested generator is calibrated.

No qualifying operational receipt bundle exists here. An end-to-end operational
activation result remains unverified until genuinely eligible data, independent
reviews, elapsed maturity, matched baseline forecasts and held-out validation exist.
The 151 qualified corpus tasks do not establish that population. Provider composition
limitations, manual project semantic review and notebook semantic limits are unchanged.
The two protected legacy fit-without-cutoff tests remain unchanged and explicitly fail
for the documented contract conflict in `engine-maturity.md`.

Final checks: CLI library 125 passed/1 ignored; M4 training integration 87 passed;
learner 137 unit + 1 integration passed; offline Python transport 18 passed.
Independent review found no remaining scoped blocker. These are affected checks,
not a claim that the protected cutoff conflicts or runtime-dependent broad failures
are resolved.

Logs: `target/tier2-final-{cli,learn,clippy,build}.log`,
`target/tier2-adapter-tests.log`, `target/tier2-python-{local,forecast}.log`.
