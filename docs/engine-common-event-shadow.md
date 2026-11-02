# Common-event shadow evidence

This increment adds pre-return shadow receipts without changing factual selection,
propensity, authority, or learned within-run progress. It does **not** activate a
calibrated policy or establish mature real-world support.

## Operator interface

```
target/debug/tsr engine --directory RUN shadow-configure --update shadow.json
target/debug/tsr engine --directory RUN shadow-view
```

The update JSON contains `state_root` (current factual root), `reason`, and
`config`: `bundle_directory`, `bundle_id`, optional `dataset_directory`,
`event` = `task_accepted_in_budget@v1`, declared `family`, `lineage`, and optional
`training_as_of` RFC3339 cutoff. These are explicit local artifact paths; there is
no discovery, inference, network access or automatic artifact promotion. Use the
actual bundle digest and dataset cutoff, not placeholders presented as evidence.
Normal direction/adaptive execution then records shadow attempts automatically.

Configuration and receipts use the existing Store/CAS contracts in `RUN/shadow`.
They do not alter the factual root used for seeded selection. An unavailable
artifact produces an unavailable receipt; side-store failure emits a diagnostic
and cannot authorize or prevent factual execution. Existing aliases in database,
WAL/SHM and CAS storage are rejected. This is an operator-owned foreground store,
not a sandbox against a concurrent hostile filesystem writer.

## Bindings and eligibility

Receipts bind the candidate/action, common event, immutable pre-offer prefix digest,
factual state, source revision, prediction cutoff, pinned bundle, training root,
feature schema and maturity policy/proof when available. No future return or label
is supplied. Generator common-event forecasts stay missing when not provided.
Chosen and unchosen receipts have null labels; this implementation does not join
later labels or treat unchosen possibilities as failures.

The shared learning evaluator reuses feature extraction, bundle validation, support
windows and close-plus-seven-day maturity validation. Explicit artifacts are copied
with capped reads to private snapshots (16 MiB/file, 64 MiB/set), validated with
existing loaders, and removed afterward. Task/lineage overlap, wrong event/pin,
future evidence, closed tasks, already committed candidates, changed proof/cutoff
and dataset/training-root mismatch are refused. Row support is distinct from unique
mature task support; repeated correlated rows cannot supply 150 independent tasks.
Even mature support is **not** calibration quality or authenticated independent review.
The calibrator remains diagnostic and `calibrated_probability` is null.

## Current adapter boundary — not completed numerical integration

Notebook `coverage` and adaptive/project progress are not the trained event
`task_accepted_in_budget@v1`. Their adapter projections remain incompatible with the native model feature contract.
The project lifecycle bridge reuses actual Executed duration/exit observations
(`adaptive/coding.rs`). Existing engine
event timestamps use unix-ms provenance rather than the native RFC3339 contract. They explicitly emit
`unsupported_common_event_lifecycle_or_action` for a valid bundle (missing artifacts
emit unavailable). Action-kind similarity is not sufficient applicability.

The numeric path is exercised with native lifecycle **synthetic fixtures**, including
an independently checked sigmoid-zero probability. It is not operational numerical
shadow prediction for current engine adapters, real-corpus validation, or live model
verification. Features@v1 represents history/action/family, not patch semantics.

A [project lifecycle evidence bridge](engine-project-lifecycle.md) now records
prospective admission, tool-time accounting, exact event/timestamp provenance and
independent review-bound closure. It retains machine-enforced incompatibility:
receipt features now have a separate versioned schema and instrumented prospective
budgets, but legacy coverage and matching validated model artifacts remain missing. Accepted review is separate from a common-event label. No native numeric
prediction or new mature-episode claim is enabled by this bridge.

## Verification

New tests cover artifact replacement/size limits, source/pin/cutoff/proof mismatch,
query overlap, legacy unknowns, row-versus-task support, pre-return leakage guards,
source provenance, restart and storage alias rejection. Cloned-prefix notebook and
adaptive controls compare factual choices/policy and observed values with shadow
absent versus unavailable. Clock-derived event IDs are provenance and are excluded
from cross-run equality; decision inputs and probabilities are compared.

Affected suites: CLI 114 passed (one existing ignored), learn 133 passed.
Local logs: `target/shadow-final-tests.log`, `target/shadow-final-focused.log`,
`target/shadow-artifact-tests.log`, `target/shadow-clippy.log`,
`target/shadow-build.log`. Independent review identified alias and read-bound issues;
these were fixed before commit. No inference or external transmission was used.

151 qualified corpus tasks remain recorded; the independently mature eligible
common-event episode count is unknown. Installed-provider inability in the preserved
conflicting-desire state remains unresolved. No claim of full engine completion.

Receipt telemetry now exposes versioned pre-action project features and prospective
instrumented resource accounting; see [the contract](engine-project-telemetry.md).
This does not make native features@v1 compatible or enable labels/numeric predictions.
