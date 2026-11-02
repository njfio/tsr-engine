# A1 synthetic archive continuation: output-bound stop

On 2026-10-22, a fresh three-call admission authorized the same CheaperInference
`deepseek-v4.1-flash` model, synthetic-only frozen archive/parents/A1, at most one
simulated operation, and no uncertain retries. This did not reuse previous slots.
The criteria and initial request were frozen before dispatch, based on commit `66f2251`.

Only the first **possibility-generation** request ran:

| Observation | Recorded result |
|---|---|
| HTTP / billing | 200 / settled |
| Actual returned model | deepseek-v4.1-flash |
| Input tokens | 2,002 |
| Completion tokens | 2,048 (the configured maximum) |
| Reasoning tokens | 2,043 |
| Total tokens | 4,050 |
| Charge | $0.001146 |
| Duration | 8.996 seconds |
| Structural outcome | incomplete output; no validated typed proposal |
| Assessment / operation / revision | not attempted |
| Unused calls | two expired at closure |

The provider's finish reason was not `stop`. The runner rejected it before JSON
validation, preserved the response-body hash and metered receipt, and did not log
raw provider content. It did not retain the exact non-stop finish reason, so this
report does not reconstruct that field. Token accounting shows the completion
allowance was exhausted and almost entirely used for reasoning. This establishes
an output-bound failure at this configuration, not explicit generator inability or
an assessment of the semantic relevance of a proposed action.

A fresh-process request after closure equals the complete pre-dispatch request.
The root remained `7ba7eda3615ab1a7c2e32588b2a9da3f38ccabf5eb58a220dd4ae7be286d17ea`.
Original parents, exact A1 aspiration, history, events and budgets are unchanged.
No new action or relational revision occurred. The evaluator-authored seed event
from fixture setup remains explicitly separate from model-generated behavior.
No local runtime was started. No account settings, shared-key budget, original
repository, protected tests or private data were changed or transmitted.

Evidence: `target/synthetic-archive-live/` contains the admission, initial request,
allowlisted payload, started marker, metered receipt, terminal summary and checked
`invariants.json`. There is one started request, no second or third started request,
and no action-started marker. The request process exited normally after closing the
admission. No production code changed, so the prior affected-test result remains
207 Rust tests passed / 5 ignored and 87 Python tests passed; this turn did not rerun it.

## What remains

The intended generated possibility → assessment → selected operation → observed
return → revision cycle is **not yet validated** in the synthetic archive. There is
no action/revision output for independent semantic assessment. No claim about
social fulfillment or causal influence of A1 is warranted.

A materially justified next experiment would use a new bounded admission with a
larger completion allowance (or a documented model-compatible reasoning limit),
keeping this identical state, model, desires and controller rules. Increasing output
capacity changes the resource configuration; it does not repair an observed engine
selection defect. It must not silently reuse these expired slots or blindly repeat
this 2048-token request. The exact finish reason should also be retained in future
receipts before incomplete-output rejection. No further call was dispatched.

## Independent review and offline correction

Independent review confirmed the incomplete-output diagnosis, one settled call and
unchanged root. It also found an unused assessment-path bug: the runner checked the
presence of its validated-proposal marker without comparing its root/digest with the
loaded proposal. This did not affect the terminated possibility-only attempt.

Original `run.py` is preserved. `runner-next.py` rechecks the exact root and proposal
digest before assessment transmission and rechecks the proposal before application.
It also preserves a bounded finish-reason value before incomplete-output rejection.
Offline temporary-fixture tests accepted an unchanged proposal and rejected both a
changed proposal and wrong root. These test inputs were never runtime candidates or
provider payloads. The closed admission still prevents dispatch; a fresh admission
and directory are required for any future continuation.
