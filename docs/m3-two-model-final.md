# Comparison halted before inference

The frozen six-attempt comparison did not complete. The first DeepSeek-labelled native task ended unavailable before any inference request; the other five scheduled tasks were not launched. This is a harness integration failure, not a model failure or comparative result.

## Cause and evidence

The new wrapper reads `TSR_M3_EXPERIMENT_ATTEMPT` to select its frozen model. The unchanged native `ProcessBroker` explicitly clears its child environment and preserves only HOME and PATH (`crates/tsr-batch/src/broker_v2.rs:724`). Consequently the wrapper exits before it reads the request or calls the transport. Native stderr suppression leaves the generic unavailable closure. The experiment controller then rejects the zero-request diagnostic delta with `arm call bound` and halts.

The source-level cause was discovered after launch. Both implementation and independent preflight review missed the native child-environment boundary. The offline tests exercised lifecycle/accounting and historical exports, but not the new wrapper through the native process boundary. No model-facing contract or frozen controller was changed after this failure, and no retry occurred.

The native process ran 2026-10-17 15:14:45.756912 to 15:14:55.900477 UTC (about 10.144 seconds including worker polling). Its recorded model wall time was 98 ms, generator tokens zero, no proposals, no tests, and no final oracle. The process exited normally with no cleanup errors. The native unavailable result and its independently checked event root, CAS files and empty diagnostics are preserved under `m3-two-model-execution/0/evidence/`. The terminal guard rechecked frozen inputs, accounting, runtime identity, idle state and disk reserve.

## Accounting and outcome

- New inference requests: **0**; new settled charge: **$0.000000**.
- Aggregate settled: **$0.231086**; outstanding reservations: **$0**.
- One harness attempt launched; five scheduled attempts unlaunched.
- Model schema failures, stale-digest failures and final-oracle completions: no new model observations.
- Freeze: `077692edf385b24e6c3b2580932c1ffa626e1b1e52f7bbdaa48b53b4b7b03c71`, 652 pinned inputs.

See `m3-two-model-result.json`, `m3-two-model-export-sha256.json`, `m3-two-model-readiness-review.md`, and `m3-validation/m3-two-model-*.log`. Six local lifecycle/budget tests and five v3 adapter tests passed; historical schema/stale exports and a scripted successful receipt replay passed. Those checks did not establish native wrapper compatibility.

## Remaining work

A separately reviewed experiment revision must bind each model through a controller-owned entrypoint that survives the existing native environment clearing. It should first test the exact native process boundary offline. This report proposes that correction only; the frozen six-attempt run was not redesigned or restarted after failure.

The experiment provides no new real-corpus evidence and does not meet M3 acceptance criteria. No larger corpus comparison, push, PR, merge, deployment, publication or deletion was performed. The branch remains `codex/m3-evidence-pipeline` in `/Users/n/Documents/Codex/2026-10-14/task/tsr-engine-m3`.
