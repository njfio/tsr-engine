# Reusable admitted synthetic external cycle

The normal `scripts/engine-directions.py` entry point now has an explicit synthetic-only
external mode. It uses reusable `engine-synthetic-cycle.py` and
`engine-synthetic-transport.py`, the existing archive contract, proposal/assessment
normalization and Rust controller. No evaluation-specific parent script is required.
The default installed-local-provider mode is unchanged.

This version deliberately admits only the existing evaluator-authored archive fixture
(hash `612f479aea815883e97d9b3de772ab3ffeb5056114d068770b94d648b565b200`)
and its frozen rich parents. It is not a general repository/source exporter. It may
continue explicitly admitted generated hypotheses from that synthetic history;
source snapshot, current state, implementation files and controller binary are pinned.
Full operator/review reports and normal driver contexts are never serialized for the
external provider. Relational context projects eligible latest hypotheses, not merely
the last historical record; unsupported normative prose remains uncertain guidance.

## Runnable interface

Run from the repository root. `RUN` must be a dedicated copy of an existing admitted
synthetic archive run (for example the preserved `target/archive-return-cycle/run`).
Do not reuse a directory with another external admission. The example chooses a 30-minute expiry with an explicit timezone. These commands are instructions for a future
explicitly authorized live run; no such call was made during implementation.

```sh
RUN=target/next-admitted-synthetic-run
cp -R target/archive-return-cycle/run "$RUN"
TSR_SYNTHETIC_EXPIRY=$(python3 -c 'import datetime as d; print((d.datetime.now(d.timezone.utc)+d.timedelta(minutes=30)).isoformat())')
python3 scripts/engine-directions.py \
  --directory "$RUN" --model deepseek-v4.1-flash \
  --prepare-synthetic-admission "$RUN/admission.json" \
  --calls 3 --synthetic-actions 1 \
  --synthetic-expires "$TSR_SYNTHETIC_EXPIRY"
```

Preparation performs no network or credential access. Inspect the frozen manifest and
printed allowlisted projection. Passing the resulting manifest to run is the explicit
external-run admission:

```sh
python3 scripts/engine-directions.py \
  --directory "$RUN" --model deepseek-v4.1-flash \
  --synthetic-admission "$RUN/admission.json"
```

Reissue that same command for a safe restart. Do not remove receipts or change the
manifest. Stage choice comes from controller state: due aspiration/revision first,
otherwise fresh possibilities, frozen assessment, one selected operation, then due
return revision. A run starting without applicable aspiration but with observed contrast
may need four calls; preparation permits a bounded explicit count of 1–12, not implicit
budget expansion. Calls default to three in the example only; the manifest is authoritative.

The fixed provider/model is CheaperInference `deepseek-v4.1-flash`, using existing
configured access. There is no provider switch, credential creation or account change.
The provider alias is checked in receipts; an immutable hosted model-weight digest is
not available or claimed. All stage requests use temperature zero. The manifest pins
8192 completion tokens for possibilities/relational review and 32768 for assessment,
including reasoning, with at most 240 seconds each and 16384 payload bytes. Manually
lowered manifest limits may be admitted before first use; increases beyond built-in
ceilings fail. A run's frozen manifest cannot be changed to reset limits.

`max_transport_seconds` is the conservative sum of reserved provider-call deadlines,
not a measurement of total process runtime. `expires_at` also gates dispatch and apply;
each transport deadline is shortened to remaining expiry. Controller CLI operations
retain their existing 30-second bound. Completion/input reservations remain distinct
from actual reported usage; local fixture calls in tests have no real provider charge.
No server-enforced dollar cap or calibrated price estimate is asserted. Actual settled
USD, token counts, reasoning/cache details, elapsed time and uncertainty are recorded.

## Stop, restart and accounting

Files under `RUN/synthetic-external/` are exclusive/immutable and fsynced. One local
lock excludes simultaneous external-cycle drivers. Stages record:

1. Request/schema and dispatch intent, with source, admission and payload hashes,
   reserved call/input/completion/time resources.
2. Provider outcome: safe metadata/body hash and schema-valid typed output only.
   Raw reasoning, raw invalid body and credentials are not logged. Invalid/truncated
   results retain metering and stop; they are not relabeled as inability.
3. Application intent, controller receipt, then applied record with before/after root.
   Original parent/authority/world scope is checked at every continuation.

A recorded settled outcome resumes validation/application without another provider
call. A recorded controller receipt resumes bookkeeping after checking current root,
without repeating its action. Unknown dispatches and application acknowledgments stop;
no assumption of failure, refund or exactly-once provider execution is made.
Expiry/withdrawal terminal receipts retain unmatched dispatches, token/time/call
reservations and unknown application outcomes separately from known settled charges.
Known costs are a partial sum when uncertainty remains, not a claim of zero expense.

Create `RUN/synthetic-external/STOP` to pause before dispatch/application. An optional
`--stop-file PATH` is also honored. Remove the stop file only when resuming intentionally.
A settled response paused before application remains recoverable. Permanently withdraw
unused admission with the same run command plus `--expire-remaining 'reason'`.
Terminal withdrawal/expiry cannot be reopened. Source/goal withdrawal or state drift
stops instead of silently rebinding. Recorded history is never rewritten.

The final action's due return review may use remaining call allowance, without another
action grant. Empty/inability/invalid outcomes, no selected action, resource exhaustion,
expiry and withdrawal retain their distinct evidence. An unsuccessful controller apply
is conservatively unacknowledged until reconciled rather than guessed retry-safe.

## Verification and honest limits

All validation this turn was offline. Real Rust controller state is combined with
explicitly authored fake-provider replies; no tested fixture output counts as model
emergence or semantic fulfillment. Fresh subprocesses exercise full aspiration →
possibility → assessment → actual fixture operation → return revision and recovery
after recorded outcomes, frozen possibilities, controller receipts and applied stages.
Interruptions before a response and before/after unacknowledged application never cause
automatic repeat. Additional checks cover source/request/admission drift, unknown usage,
invalid output, expiry/withdrawal uncertainty, stop/resume, private-field exclusion,
zero action allowance, unchanged parents/authority, and exhausted restart.

Independent review corrected two issues before handoff: unresolved dispatch accounting
on expiry, and omission of eligible relational predecessors. Terminal reporting also
preserves unknown application acknowledgment. A final read-only review found no further
dispatch/authority blocker. No assertion that guidance reliably improves behavior was
added; the paired counterexample and unsupported normative language remain recorded.

Tests/logs are in `target/synthetic-driver-integration/`. The offline Python engine
suite ran 205 tests, with 10 skipped, and passed after granting its existing localhost
fake-provider sockets permission. The restricted run's 11 bind-permission errors remain
in `tests.log`; the successful loopback run is `tests-loopback.log`. Final focused
cycle checks passed nine test methods with fresh-process subcases; eight transport
tests passed, including inconsistent usage preservation. The final combined sweep
ran **206 tests: 196 passed, 10 skipped**, recorded in `final-engine-tests.log`. No new software,
persistent service, protected test edit, external transmission or paid call occurred.

## Still manual before unattended live validation

- Select a dedicated synthetic run, review/approve the source projection and explicit
  call/action/expiry bounds. No budget or purpose is inferred from old admissions.
- Existing configured credential must be available at live dispatch; no credential is
  read during preparation or fake-provider tests.
- Reconcile uncertain provider/application outcomes from preserved receipts/history;
  this version intentionally supplies no automatic abandonment/retry or budget refund.
- Obtain independent semantic assessment of genuine outputs. The pipeline validates
  typed provenance and effects, not arbitrary social meaning or fulfillment.

The next validation can use this single normal command within a fresh bounded admission.
It needs no parent stage-by-stage calls or evaluation-specific runner. Live compatibility
and meaningful substantive-return evolution are still unverified for this new integration;
provider reliability, mature calibration and general autonomy remain separate limits.
