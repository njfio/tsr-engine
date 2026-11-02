# Demo reproducibility checks — 2026-10-22

No live inference, paid calls or project-target executions were performed in this turn.
The historical workshop evidence remains unchanged. Both owned test commands reached
terminal results; process listing was not used to bypass the earlier sandbox denial.

## Focused checks and review

Three `scripts/test_engine_demo.py` tests passed: fresh-process identical read-only replay,
unchanged evidence hashes and no replay subprocess; missing/modified/symlink evidence
fails closed; paid execution without opt-in stops before driver invocation. These were
run separately after the aggregate Python discovery began.

Independent code review found no blocker in replay/opt-in boundaries. Independent
semantic/documentation review accepted the bounded claims and proposed next scenario,
with the explicit qualification that differing-condition reports need not contradict
one another. The index is a historical trust anchor, not a fresh SQLite/CAS audit.
The old queue guide was preserved byte-for-byte as `engine-queue-demo.md`.

## Complete offline aggregate results

Python: `python3 -m unittest discover -s scripts -p 'test_engine_*.py'`.
221 tests: **200 passed, 10 skipped, 11 errors**, 70.893 seconds, exit 1.
All errors occur in `test_engine_run.EngineRunTests` while creating a local HTTPServer:
`PermissionError: [Errno 1] Operation not permitted`. Affected tests:

- budget_stops_before_network
- busy_ledger_stops_without_network_or_waiting
- existing_aggregate_cost_blocks_new_reservation
- explicit_settled_decode_recovery_preserves_bounds_and_history
- one_dollar_scope_settles_sized_reservation
- settled_invalid_scope_never_executes_or_retries
- settled_refusal_stops_and_cost_is_known
- settled_response_lost_before_journal_never_duplicates_call
- timeout_keeps_reservation_and_does_not_retry
- token_budget_stops_before_network
- unsettled_charge_stops_restart_without_another_request

Each name has the `test_` prefix in the log. No denied socket operation was retried
through another access mechanism.

Rust: `rustup run 1.90.0 cargo test --workspace --no-fail-fast --offline`,
`CARGO_NET_OFFLINE=true`, `CARGO_BUILD_JOBS=2`, 1200-second wrapper bound.
Completed in **730.961 seconds**, Cargo exit **101**, no timeout.
Across all 77 result summaries: **957 passed, 7 failed, 11 ignored**.
Five failures are local-listener sandbox errors (`Operation not permitted`, OS error 1):

- local_partial_http_is_hashed_but_not_accepted
- local_live_cancel_and_lock_exclusion
- local_timeout_and_precancel_preserve_reservations
- local_transport_isolated_metered_quarantined_and_not_replayable
- local_transport_rejects_bad_usage_malformed_truncated_and_injected_reports

These are in `tsr-cli` independent-review local transport tests. Two protected M0
failures match the existing runtime-dependent cases documented in `AGENTS.md`:

- `m0_forecast_default::exec_resolves_the_session_task_and_refuses_without_one`:
  expected exit 3, observed 126.
- `m0_harness::selftest_is_reproducible_structurally_excluding_duration_ms_by_name`:
  fixture read returned NotFound after cleanup.

No assertion, deadline, protected fixture or Rust implementation was changed. No loop
until green was performed. This is **not a clean aggregate pass**; socket-dependent
behavior remains unverified in this sandbox.

Local full logs: `target/demo-regressions/python.log`, `rust.log`,
`rust-result.json`. They are not published. The useful next semantic work is the
separately frozen different-desire/return validation described in `engine-demo.md`,
not more bookkeeping or an unsupported general-autonomy claim.
