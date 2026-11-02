# Verifier stub deadline investigation

Offline investigation after `dbd5734`, 2026-10-26. No model/provider calls, generated
target executions, acceptance changes or deadline changes. The frozen next-proposal
comparison and historical evidence remain unchanged.

## Established failure mechanism

`target/next-proposal-cli-tests.log` retains the original result: 234 passed,
one failed, eight ignored. The failed test is
`local_transport_isolated_metered_quarantined_and_not_replayable` in
`acceptance/reviewer/local_tests.rs`. Its admission is 2,000 ms. The final record
contains a complete chat response, measured 100 input/50 output tokens, both
successful model-identity fetches and `elapsed_ms:2112`; the diagnostic is
`invalid verifier report: unknown or exceeded verifier resources`.

This was not a socket timeout, missing server response, unknown provider completion,
or semantic rejection. `project_verifier_run` times the **entire** runner invocation
and refuses a report when elapsed time exceeds the admission. `Local::evaluate`
includes freshness/provenance checks, durable reservations and progress records,
three HTTP exchanges, settlement and terminal persistence. The HTTP deadline starts
after initial freshness/budget lookup; finishing HTTP within that window does not
excuse exceeding the outer whole-invocation budget. The stub's reported 1 ms duration
is not total host elapsed time. Quarantine and measured cost retention were correct.

## Bounded reproduction

Temporary cfg(test)-only monotonic breadcrumbs measured phase completion. They were
removed byte-for-byte before final verification; no production or harness repair
was inferred from a passing retry. All instances use private temporary stores and
loopback listeners, unchanged 2,000 ms admissions and actual persistence.

| Condition | Initial checks | Chat captured | Final pin/freshness | Terminal persisted |
| --- | ---: | ---: | ---: | ---: |
| Isolated exact test | 74.7 ms | 293.5 ms | 431.1 ms | 465.1 ms |
| Four concurrent exact tests, range | 77.6–79.3 ms | 335.8–399.6 ms | 500.4–558.7 ms | 532.8–627.2 ms |
| Default-parallel library aggregate, corresponding positive request | 240.5 ms | 1159.1 ms | 1582.4 ms | 1766.2 ms |

All five exact-test instances passed. The instrumented default-parallel aggregate
passed 235 tests, with eight ignored, in 55.70 seconds. Its delayed-response timeout,
pre-cancel, live cancellation, reconciliation and malformed-output controls passed.
No artificial busy-loop stress or successive reruns until green were used.

These data demonstrate load-sensitive host work and limited aggregate headroom.
They do **not** identify the precise CPU, disk, scheduling or persistence contribution
in the historical failure: that run lacks phase timings. The original run also
had opt-in Python/native verification processes active; their causal contribution
cannot be reconstructed. No shared-state collision, stub lifecycle race or unsafe
acceptance was reproduced. Therefore no harness or production change is justified
by this evidence, and the original aggregate failure is not relabeled harmless.

## Final verification

Final uninstrumented default-parallel CLI library aggregate: **235 passed, zero
failed, eight ignored**, in 55.44 seconds. The affected positive test and timeout,
cancellation, malformed-output and reconciliation controls all passed.
The original failure and every diagnostic run remain separate, not combined into a
clean historical result. Temporary diagnostics are absent from the committed code.

Evidence: `target/verifier-deadline-diagnostic-aggregate.log`,
`target/verifier-deadline-contention-{0,1,2,3}.log`,
`target/verifier-deadline-diagnostic-isolated.log`,
`target/verifier-deadline-comparison.json`, and
`target/verifier-deadline-final-aggregate.log`.

## Effect on next-proposal readiness

The prepared comparison uses the shared Python external shadow runner and native
read-only proposal validation; it does not invoke this local verifier transport.
Its own accounting, uncertainty, deadline and no-adoption controls remain separately
verified. This local timing fragility is a real broader regression reliability limit,
not a demonstrated defect in the comparison path. Fresh live approval is still
pending; no call is authorized or dispatched by this investigation. A future
approved run must create current context and verify worker/dependency hashes as
usual, rather than reuse an expired preparation manifest. The diagnostic rebuild changed
the test-worker binary hash despite restoring the exact source. The old manifest
correctly fails that hash check and is preserved; fresh offline preparation is
required before dispatch, without changing the frozen comparison conditions.

Fresh offline preparation and separate-process preflight/inspection passed at
`target/next-proposal-shadow-deadline-checked`, manifest `bacb251c41127ff8d08bb34b2b8d79d5691cf1a0103f05dd4e278b153d9e7d8b`.
Live authorization is false; zero calls/reservations/charges and no journal.
Frozen rubric and conditions are unchanged.
