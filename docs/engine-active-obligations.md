# Active standing obligations and preserved coverage recovery

The preserved prerequisite run recovered full original account coverage on local
attempt 21, using one of the newly allowed ten calls. The installed
`qwen2.5-coder:32b` generated the proposal on code `0afc816`; no candidate, next task,
or repaired answer was supplied by the operator. Observation changed from
`[2/3, 1, 2/3, 1]` to `[1, 1, 1, 1]`. These values measure note coverage, brevity,
and two generated coverage refinements. They do not establish truth, explanatory
quality, or understanding. Earlier failed and timed-out calls remain in the journal.

The temporary local provider was stopped after this terminal result. There were
21 cumulative calls, one used in this batch, nine unused, and no paid/external calls.
Evidence is in `target/prerequisite-live/coverage-recovery-summary.json`,
`coverage-recovery-restart.json`, and `run/{directions-21.json,
normalized-directions-21.json,turn-21.json,local-directions.jsonl}`. The rebuilt
active-obligation implementation reads the same satisfied state and unchanged root
`e2488af3b8b115f2a7682f90fad4e140898dbf3be5abe93a766d16ea8a69ac5b`.

## Decision correction

Previously, ledger attention could exclude a withdrawn desire while ranking,
completion, and observed-progress labels still counted it. Generated descendants
could also reenter attention. One snapshot-derived active view now governs all
these paths. Fulfilled standing wants remain active against regression; withdrawn,
out-of-scope, superseded and overlapping wants remain historical. A descendant
requires every refinement ancestor to remain active. Original identities and
conserved influence do not change.

Each new return freezes its active obligation identities. Historical success labels
transfer only across the same canonical active original set; changing the goal
scope is not treated as a failed strategy. Revision-compatible forecast residuals
remain separately usable and separately attributed in `calibration_evidence`.
Legacy receipts remain readable without rewriting them. The counterfactual learning
audit exposes both success-rate and residual evidence.

Notebook refinements already admitted by mechanical review retain that admission;
when the generic ledger has no status for such an identity, explicit operator
status history takes precedence over the local default. This neither invents an
operator admission nor changes the generic ledger's admission rule.

## Operator interface

Read the current root and identity, then write an update JSON with `state_root`,
`preference` and `status` (`open`, `withdrawn` or `out_of_scope`):

```sh
target/debug/tsr engine --directory RUN direction-request
target/debug/tsr engine --directory RUN direction-status --update update.json
```

Updates are locked, version-bound, append-only operator events. Unknown identities,
stale roots and attempts to declare fulfillment are rejected. Generated directions
cannot invoke this interface. With no active original wants the state is `inactive`,
never falsely `satisfied`. Restoring a want preserves its identity and prior evidence.

## Verification and boundary

CLI library: 91 passed, one existing ignored; Python direction tests: 26 passed.
CLI all-target Clippy and build passed on Rust 1.90. Tests cover withdrawal,
restoration, conflicting standing wants, exact unchanged/restored controls, nested
lineage, restart, historical success-scope separation and operator authority.
Fresh ordinary-command smoke evidence is `target/active-status-smoke/result.json`;
logs are `target/active-final-tests.log`, `active-python.log`,
`active-final-clippy.log`, and `active-build.log`. Independent review found no
remaining blocker and requested the separate calibration audit provenance.

The new status behavior is deterministically verified, not a second live-model
experiment. Protected tests and prior live receipts were not rewritten.

The next PRD boundary is explicit expectation support. The current related-turn
progress rate is a bounded engineering estimate, not validated Tier 1 calibration.
PRD B8 gates its Tier 2 support-floor policy on mature episode evidence and validated
calibration. An arbitrary new threshold or counting correlated turns as independent
episodes would not close that gap. A concrete preparatory step is to expose named
outcome identity, compatible evidence weight, independent episode support, feature
coverage and validation availability, then connect an explicitly declared fallback
policy. Existing sparse notebook evidence does not meet the PRD's statistical gate.
