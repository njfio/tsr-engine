# Completed six-attempt synthetic comparison

The corrected experiment completed all six planned actual model attempts, three per model, with no retry or discarded failure. Each model achieved two independently verified final-oracle successes. This is a bounded launch/reliability observation on one trivial synthetic task, not real-corpus M3 validation or a statistically supported model ranking.

## Results

| Attempt | Requested catalog model | Terminal outcome | Requests | Worker wall seconds | Broker elapsed seconds | Settled USD |
|---|---|---|---:|---:|---:|---:|
| 0 | deepseek-v4.1-flash | Accepted; verified final oracle | 5 | 40.086 | 14.776 | 0.001122 |
| 1 | glm-5.3-flash | Accepted; verified final oracle | 6 | 70.081 | 57.727 | 0.002757 |
| 2 | deepseek-v4.1-flash | Stale edit digest; no final oracle | 4 | 20.129 | 12.913 | 0.000726 |
| 3 | glm-5.3-flash | Accepted; verified final oracle | 6 | 110.097 | 100.305 | 0.001991 |
| 4 | deepseek-v4.1-flash | Accepted; verified final oracle | 7 | 30.092 | 18.574 | 0.001304 |
| 5 | glm-5.3-flash | Invalid forecast type; no final oracle | 3 | 30.132 | 20.874 | 0.000597 |

Worker wall time includes local execution and the worker's 10-second polling granularity. Broker elapsed time sums recorded adapter durations, including catalog access, inference, billing and decoding; it is not pure model inference latency.

| Model | Final-oracle successes | Schema failures | Stale-digest failures | Requests | Settled USD |
|---|---:|---:|---:|---:|---:|
| deepseek-v4.1-flash | 2/3 | 0 | 1 | 16 | 0.003152 |
| glm-5.3-flash | 2/3 | 1 | 0 | 15 | 0.005345 |

Attempt 2 persisted inspect, failing test, and patch proposals. A subsequent edit failed the native `stale or oversized edit source` precondition. The retained source was 105 bytes, below the size bound, so this was a digest mismatch. No final proposal, Stop, task closure, native result or oracle receipt was invented. The rejected expected digest was not retained; its exact value and the model's intent are unknown.

Attempt 5's third response proposed an edit with `/arguments/forecast` as a string, where a number is required. The settled `forecast_type` diagnostic retains field/type information without response text. The native task closed unavailable. Both failures count, and the ordinary failure continuation policy worked. Visible test results alone were never classified as successful completion.

## Budget and scope

The experiment used **31 of 120 permitted inference requests** and **$0.008497 of the $5 additional allowance**. Aggregate settled spending is **$0.239583**, with **zero outstanding reservations**. All attempts remained below 20 requests and 1,800 seconds. The primary terminal guard passed frozen-input, accounting, runtime/process idle, pinned-image and disk checks. No further inference or larger corpus evaluation followed.

The new catalog read verified exact IDs `deepseek-v4.1-flash` and `glm-5.3-flash`, both advertising chat-completions. Discounted input/output prices per million tokens were respectively **$0.091243/$0.364970** and **$0.082500/$0.275000**; list prices were $0.15/$0.60 and $0.15/$0.50. Catalog quotes can change; actual costs above are provider-settled charges. See `m3-two-model-v2-catalog.json` for the timestamped snapshot.

The task, source snapshot, preparation, record mode, v3 schema, required tool, 16,384 output cap and existing transport settings were unchanged. Temperature and reasoning settings remained omitted on both models. No unsupported parameter substitution or support error occurred. Provider-default sampling and routing remain uncontrolled; requested model IDs do not independently prove upstream served-model identity. Three attempts per model on one synthetic task cannot establish general performance differences.

## Launch repair and validation

The earlier failed launch remains preserved under `m3-two-model-*` and commit `79c1dc6`. It made zero provider requests and is separate from these six model attempts. The minimal correction added six controller-owned static entrypoints and a common wrapper, selecting a model without an environment variable. The wrapper verifies the entrypoint against the frozen manifest. The engine's environment clearing and credential boundary were not weakened.

Validation actually run:

- Six lifecycle/budget tests against the corrected controller: passed.
- Six offline end-to-end checks through the actual frozen native ProcessBroker and exact production entrypoints: passed, 30 scripted actions. Credential/network calls were forbidden and transport substituted; no paid requests occurred.
- Independent reread of all six scripted final-oracle receipts: passed. These are integration checks, not model evidence.
- Independent review of the narrow correction, all 709 frozen inputs and 90 offline evidence files: passed before launch.
- Live controller checks of event roots, CAS, result identities, receipts, diagnostics, settlement bindings and durable exports: passed for all six attempts.
- Terminal evidence export: 167 files independently rehashed by the primary agent. Separate terminal review is recorded in `m3-two-model-v2-terminal-review.md`.

Freeze SHA256: `fe2038a733c36a9d7ac3cb3c2a0b1cfaf651ff228772d71874dc0ee8a590e8e8`.

## Deliverables and remaining limitations

Implementation: `scripts/m3_two_model_v2_core.py`, six `scripts/m3-two-model-v2-arm-*.py` entrypoints, versioned controller/freezer/report and native-check/evidence scripts. Machine-readable results: `m3-two-model-v2-result.json`; adapter timing/type observations: `m3-two-model-v2-observations.json`; complete evidence: `m3-two-model-v2-execution/` and its SHA256 manifest. The failed revision and unrelated work were preserved.

Branch: `codex/m3-evidence-pipeline`. Worktree: `/Users/n/Documents/Codex/2026-10-14/task/tsr-engine-m3`. No push, PR, merge, deployment, publication, deletion or larger corpus comparison was performed. The requested bounded comparison is complete. **M3 real-corpus acceptance remains unmet by this experiment; zero qualified corpus tasks were added.**
