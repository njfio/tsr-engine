# Project lifecycle evidence bridge

This extends the isolated shadow path with a prospective episode admission, fixed
tool-time ceiling, original-to-projected event mapping, timestamp conversion,
independent review binding and durable closure. It does not change factual action
selection, authorize work, or make project evidence eligible for native calibration.

## Commands

```
target/debug/tsr engine --directory RUN project-lifecycle-open --admission admission.json
target/debug/tsr engine --directory RUN project-lifecycle-view
target/debug/tsr engine --directory RUN project-lifecycle-close
```

Admission JSON: `state_root`, `tool_ms_ceiling` (1..86400000), and bounded `reason`.
Use the current root from `shadow-view`. Admission requires an initialized project
and no pending execution. It applies prospectively; it cannot make earlier work
in-budget. One episode is admitted per run directory. Identical admission retries
are idempotent; changing/resetting its budget is refused. This accounting ceiling
is diagnostic and does not replace executor limits or grant execution authority.

Ordinary proposal/execution/review commands remain unchanged. Each shadow attempt
includes the admitted episode projection where available, before any observed
return. Closure explicitly requires a current independently accepted or rejected
review, a post-admission receipt and no incomplete execution. Review tests alone
cannot close an accepted episode. An unresolved/uncertain execution blocks reviewed
closure; neither an absent duration nor a failed transport is invented as zero.

The same existing side Store/CAS records admission and a complete closure in an
atomic event. A crash before the event leaves no closure; retry after the event
returns the stored closure without another TaskClosed. Repeated view/close after
restart preserves its fixed snapshot. Later source changes do not update that
historical closure or supply a new episode.

## Projection and provenance

The bridge reuses actual Proposed, selected Committed and Rules Executed records,
including exit status and duration. Original IDs and timestamps remain unchanged
in the source store. Projected events have content-derived identities and a mapping
back to original event IDs; legacy unix-ms timestamps are converted exactly to
RFC3339 milliseconds. Backward chronology, duplicate execution, missing commitment
and prefix mismatch are refused. Event order remains causal sequence, not sorting
by time. Candidate identities and their original causal parents remain original:
this projection references the source history and is not a self-contained native DAG.

Tool usage sums observed execution durations with overflow checks. A ceiling breach
adds a projected Stop(budget) at closure. Current review identity and source snapshot
are retained separately as `reviewed_accepted`, not inferred from passing tests.

## Machine-enforced remaining incompatibility

[Versioned receipt features and prospective resources](engine-project-telemetry.md)
now cover bundled test outcomes, context issuance, and instrumented local model
and execution budgets. Existing `features@v1` bundles remain incompatible with the
new adapter schema, and legacy/uninstrumented coverage remains unknown.

Consequently `native_feature_compatible=false` and `label=null` remain enforced.
A matching dataset/model contract plus qualifying independent mature evidence is
required; seven days alone cannot make an incompatible episode eligible. Original
task and clone/lineage grouping must survive projection so repeated repairs cannot
manufacture independent mature tasks.

## Evidence

- Adapter tests run the existing project initialize→proposal→execution→review path
  with its deterministic fixture Runner. They cover accepted/rejected review,
  budget breach, unresolved execution, repeated close/open, restart, invalid
  chronology, duplicate execution, shadow attachment and unchanged factual root.
  These are integration tests, not real model or production eligibility evidence.
- A SQLite backup of the preserved real RTS correction run was inspected in
  `target/lifecycle-rts-history-copy`. CAS/work/receipt copies were used; the source
  run and original target repository were not changed. Admission succeeded, and
  closure refused its old review/test receipt as pre-admission evidence. That is
  the correct outcome, not retrospective successful validation.
- `target/lifecycle-rts-*.json` and `target/lifecycle-rts-close-error.log` preserve
  this operational CLI check. `target/lifecycle-tests.log`,
  `target/lifecycle-clippy.log`, `target/lifecycle-build.log` record checks.
- Independent review found partial-budget positive labeling and native feature-count
  mismatch; both were fixed with explicit machine-enforced unknown/incompatible
  status before commit. No inference, external transmission or deployment occurred.

151 qualified corpus tasks remain recorded. The independent mature eligible
common-event episode count remains unknown; this bridge adds no such claim.
