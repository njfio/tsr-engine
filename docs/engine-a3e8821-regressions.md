# Offline aggregate verification of a3e8821

2026-10-23, branch `codex/m3-evidence-pipeline`. No model/provider calls, target
executions, downloads, protected edits or publication. Supported per-command permission
was granted for local test subprocesses and credential-free loopback stubs.

## Python aggregate

`PYTHONPATH=scripts python3 -m unittest discover -s scripts -p 'test*.py'`
with `TSR_ENGINE_TEST_IMAGE` and `TSR_ENGINE_PREPARED_SPEC` absent:
**571 tests: 551 passed, 2 failures, 2 errors, 16 skipped**, 105.266 seconds.
The 16 skips are ten sealed target opt-ins plus six explicit cfg(test) B9.1 native-worker
opt-ins. Prior separately verified native integration is documented in offline readiness;
this aggregate did not rerun those six tests. Discovery included the pre-existing untracked
`test_m3_cleanup.py`; it was not edited or committed.

The aggregate is not clean. Independent source review established:

- Two errors with the same `test_m3_fixed12.Tests.test_blocked_and_call_budget_counts_distinct`
  identity: V3/V5/V6 test modules mutate the shared imported `original.c`; V6's stricter
  controller receives the original fixture, and V3 aliases the original test class.
  The exact base test passes in an isolated fresh Python process (1 test, exit 0).
  This does not erase the aggregate errors. Correct repair is version-local dependency
  isolation, not weakening V6 validation or rewriting the old fixture.
- `test_m3_progress_validation` and `test_m3_smoke_controller` each fail
  `test_isolated_python_can_load_budget_without_credentials`. They read the entire mutable
  cost ledger but expect historical totals 0.208627 and 0.165094. The actual current
  settled subtotal returned is 0.620338. The broker intentionally returns current total
  after validating the historical prefix. Correct repair would isolate the baseline-bound
  ledger input, not change accounting or replace expected values with current spend.

These files predate this integration (fixed12: `4ee7c74`; progress: `7fd75e1`; smoke:
`4ac8d7e`) and are untouched by `a3e8821`. No production regression was established;
this verification-only pass leaves those older test contracts unchanged.

## Historical replay

Each command ran twice in fresh processes, returned zero, produced byte-identical output,
and reported zero dispatches and zero actions:

```sh
python3 scripts/engine-demo.py replay --scenario workshop
python3 scripts/engine-demo.py replay --scenario garden
python3 scripts/engine-demo.py replay --scenario garden-trial2
```

The replay validates pinned evidence hashes and never imports transport or regenerates
missing evidence. Workshop remains a historical bounded demonstration. Garden Trial2's
settled cost is $0.006318. Original Garden Trial1 remains `provider_unknown`, with one
consumed call, unknown usage and total cost null; its retained input/completion/time
reservations are 16,384 / 8,192 / 240 seconds. A settled subtotal of zero is explicitly
not a zero total charge. No historical output or reservation was rewritten.

Local evidence: `target/a3e8821-regressions/{python.log,python-result.json,replay-result.json}`
and each scenario's replay output. These are local artifacts, not published logs.

## Rust aggregate

Completed in **845.455 seconds**, exit **0**, no timeout. Across 78 summaries:
**991 passed, 0 failed, 12 ignored, 3 filtered**. This is the complete permitted
offline workspace sweep, not an unfiltered container-target verification. Both owned
suite sessions reached terminal results; neither was duplicated after the brief
execution-transport disconnection. No persistent runtime was started.


Command: `rustup run 1.90.0 cargo test --offline --workspace --no-fail-fast --`
with three explicit `--skip NAME` filters:

- `synthetic_parser_failure_cannot_emit_success_marker`
- `selftest_is_reproducible_structurally_excluding_duration_ms_by_name`
- `real_command_statuses_and_startup_uncertainty_match_ledger`

These automatically run real installed container targets, prohibited in this turn.
They are filtered, not passed; no test file, assertion or runtime detection was altered.
Existing ignores remain ignored. Build jobs were bounded to two, Cargo network disabled,
and the original command has a 1,200-second timeout. Full log and terminal status are
`target/a3e8821-regressions/rust.log` and `rust-result.json`.

Independent review checked the test-execution inventory, Python failure provenance,
replay claims and approval envelope. No source fix was made or implied by this report.
