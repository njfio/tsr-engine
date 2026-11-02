# M3 evaluation readiness after cohort 2

No further paid attempt is launched or recommended yet. The completed cohort is evidence about this frozen driver configuration, not proof that the controller improves repairs. There were no accepted repairs and only one fully observed pair. The original eight-arm denominator, unavailable outcomes, unstarted arm and billing records remain intact.

## Per-arm diagnosis

| Task / mode | Result and evidence | Classification / next action |
| --- | --- | --- |
| `29b4ceb35e05` record | `token_reservation_bound`: 119,529 tokens spent; 80,471 remain. Next wire body 73,524 bytes plus 8,192 framing and 16,384 output requires 98,100 tokens. Repeated inspections and patch proposals appear in the event trace. | Conservative admission to the next model call exhausted; no dispatch/charge for the refused call. Context efficiency and explicit budget-result reporting are future infrastructure work. Do not raise the frozen budget or retry this outcome. |
| `29b4ceb35e05` rules | Protected full-suite test evidence changed. The disposable `validation.rs` is 89 bytes versus the 20,086-byte baseline, and its original `#[cfg(test)]` module is absent. | Genuine invalid model edit; valid oracle rejection. Preserve the protection. A recorded missing-forecast denial is also present, but the closure reason is test-evidence tampering. |
| `1604d52d9b76` record | `token_reservation_bound`: 135,602 tokens spent; 64,398 remain. Next wire body 56,339 bytes plus 24,576 requires 80,915. Repeated inspections/proposals appear in the trace. | Budget/context exhaustion; refused before dispatch. Future context handling may reduce overhead, but this is not permission to replay the model failure. |
| `1604d52d9b76` rules | Inspection was recorded; patching was proposed before the required failing test. `stop:blocked:required inspections and failing test must precede patching`. | Genuine workflow violation; valid Tier0 rejection. No oracle or policy relaxation warranted. |
| `223a50b42faf` record | Two inspections of the same 60,380-byte source, identical argument/evidence digests; no test dispatch; `stop:budget`. | Cumulative observation exceeds the driver's 65,536-byte limit. This cause follows from saved actions and control flow; the generic closure does not explicitly name it. Both individual inspections fit, but their retained contents do not. |
| `223a50b42faf` rules | Inspection then named test; test exited 101 after 188,074 ms. Two calls, 20,835 billed tokens, four tool-minutes and 194,660 ms driver time; `stop:budget`. | Strong control-flow inference: accumulated source plus test output exceeded 64 KiB. Other frozen token/call/time/tool ceilings were not reached; no patch was proposed. Context-capacity limitation, not a completed repair verdict. |
| `0d00a69acdfe` record | Provider transport failed after about 55.7 seconds. Read-only billing reconciliation found the request completed after 206.279 seconds and charged $0.003789. | Transport availability failure; response never received. Settled accounting does not restore the response or convert the unavailable outcome. Do not replay. |
| `0d00a69acdfe` rules | Never started because the preceding dispatched call had an unresolved reservation. | Correct accounting halt, not an executed process failure or model rejection. Remains unstarted in this attempt. |

[Extracted diagnostic evidence](m3-cohort-2-diagnostic-evidence.json) contains action kinds, test exits, closure reasons and bounded broker diagnostics, not model response bodies. The analysis uses `crates/tsr-batch/src/broker.rs` (64-KiB cumulative observation check and generic budget closure), `run.rs` (protected test evidence), and the frozen broker's conservative token check. Independent read-only review confirmed the interpretation; observation overflow is explicitly distinguished from a directly logged reason.

## Readiness gates for a future version

1. Add and test explicit terminal reasons for observation, token reservation, call count, wall time, tool time and patch-size limits. Keep historical classifications unchanged.
2. Design bounded observations before another experiment: scoped/ranged inspections, deduplication and bounded test summaries with independently retained evidence references. Preserve essential evidence, provenance and untrusted-input handling. Freeze the strategy, then test it locally with harmless deterministic broker fixtures. Do not tune it during a paid cohort.
3. Test timeout and accounting behavior with delayed mock responses. Consider a separately frozen timeout policy within the overall arm deadline. Retain unknown charges, capture available request identifiers, and never retry an uncertain dispatched request automatically. The current 55-second ceiling remains part of the completed attempt.
4. Keep the test-evidence oracle and required inspection/test-before-patch policy intact. Verify invalid edits and premature patches remain rejected. Do not construct prompts to bypass refusals.
5. Require clean offline verification and independent review before proposing a new frozen comparison. Use fixed task selection, matched versions, explicit missingness rules and a separate output namespace. These four tasks have been inspected; they cannot become an untouched confirmatory holdout.

These are recommendations, not changes to the frozen driver or paid plan. Current code edits in this checkpoint: none. No model calls or replays were performed.

## Corpus and storage readiness

The exported corpus remains **44 distinct admitted tasks** (18 AgentID, 26 RTS). The frozen local plan began at 38 admissions and allows at most 110 candidates: **148 total even at perfect yield**. With six completed plan candidates, 104 remain (92 RTS, 12 AgentID). At least two additional distinct candidates plus replacements for any failures must be separately selected and frozen to reach 150. Planning additional local candidates in the already authorized repositories is within local qualification scope.

At the measured checkpoint, free disk was **278.59 GiB**, leaving **178.59 GiB above the 100-GiB reserve**. Representative retained completed directories allocate 5.112 GiB per RTS task and 1.968 GiB per AgentID task. The remaining queue projects about **493.90 GiB gross**, including partially allocated active copies. This is an estimate, not an APFS physical-space guarantee; source differences and failures affect it. Nevertheless, current retention and capacity are insufficient for the full successful queue. Expect a resource stop before corpus completion unless capacity changes.

[Capacity checkpoint](m3-qualification-capacity-checkpoint.json) records measurements and verifies every frozen controller/engine/input hash. The two existing workers and original coordinators remain running. The existing floor checks occur before jobs/cohorts; active jobs can continue allocating disk. Future resource handling should account for concurrent-job peak headroom rather than treating a prelaunch floor check as a continuous guarantee.

No cleanup was executed. The existing cleanup draft is not approved and must be refreshed against completed process identities and preserved strict evidence before presenting exact paths for deletion. Active copies, source snapshots, CAS, manifests, results and ledger records must be excluded. Additional disk capacity or specifically approved deletion is needed; do not lower the reserve to force completion.

## Authorization remaining

- Continue local-only bounded qualification and prepare supplementary candidates in the approved repositories: already authorized, subject to resources and the two-worker limit.
- Local defensive fixes and deterministic regression tests: within scope, but any version used for comparisons must be independently reviewed and frozen.
- Delete retained build artifacts: not approved. Requires an exact reviewed cleanup target list and explicit authorization.
- Further paid comparisons or model-failure replays: not authorized by this instruction. The $100 aggregate ceiling is a spending limit, not permission for another cohort or the proposed 32-task expansion.
- Push, publish a PR, merge, deploy, add credentials or use unrelated private data: remain outside authorization.
