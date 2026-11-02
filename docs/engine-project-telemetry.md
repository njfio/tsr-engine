# Project receipt features and prospective resources

This extends the project lifecycle bridge. It records evidence; it does not
activate native numeric predictions, create mature labels, or change authority.

## Receipt feature contract

`project-receipt-features@v1` binds a schema digest, exact event-prefix digest and
`as_of`. Only test receipts available in that prefix are counted. Structured case
results distinguish passed, failed and unknown; integrity failures, timeouts and
legacy receipts without structured outcomes remain unknown. Reviews never supply
feature values. Future or backward timestamps and incompatible schema requests
are refused. Shadow capture uses the prefix and cutoff captured before action.

Context issuance records source-path/content digests in the existing side store,
not raw source text. Repeated identical context at the same state is deduplicated.
Issued context is not proof of model inspection or understanding. No issuance
receipt means null context counts, not zero inspections. Side observations cannot
alter factual roots or seeded selection. Historical receipts are not rewritten.

```
target/debug/tsr engine --directory RUN project-features
target/debug/tsr engine --directory RUN project-budget
```

## Prospective admission and execution

The existing `project-lifecycle-open --admission FILE` accepts optional `non_tool`:

```json
{
  "state_root": "CURRENT_ROOT_FROM_SHADOW_VIEW",
  "tool_ms_ceiling": 60000,
  "reason": "Bounded local episode",
  "non_tool": {
    "model_calls": 4,
    "model_tokens": 80000,
    "wall_ms": 900000,
    "patch_bytes": 20000,
    "external_cost_microusd": 0
  }
}
```

Omitting it preserves legacy serialization and explicitly inactive non-tool
accounting. It cannot be added retrospectively to an existing admission.
`engine-local-model.py` reserves before its installed local provider request and
journals request/settlement intent before controller mutation. It uses measured
prompt/output tokens and provider duration when available. Local external charge
zero is separate from tokens, elapsed wall time and unknown compute use.
The older uninstrumented `engine-run.py` broker refuses active budget episodes;
use the accounted local transport and ordinary project execution/resume commands.

Resource APIs are `project-resource-reserve --request FILE` and
`project-resource-settle --usage FILE`. They are controller/transport accounting
interfaces, not authenticated billing evidence. A reservation binds request ID,
current root, envelope digest, model identity and upper token/cost bounds. Exact
retries deduplicate; changed same-ID records reject. Pending reservations block
further requests and closure. Unknown/not-sent terminal outcomes retain their
reserved calls, tokens and cost, permitting a new request only within remaining
limits. Missing usage is never invented as zero. Measured overruns are retained
and stop subsequent work. An uncertain settlement is journaled, not replaced by a
second conflicting settlement. No exactly-once provider execution is claimed.

Project execution reserves command timeout bounds and full prepared replacement
UTF-8 byte lengths before dispatch. Byte accounting is deliberately conservative,
not diff size. Unknown execution keeps its reservation; observed execution charges
actual duration. Remaining wall time bounds command execution. Reservations do not
grant action permission. Closed views freeze elapsed time; closure preserves a
resource snapshot and still emits no common-event success label.

## Verification

- CLI library: 119 passed, one existing diagnostic ignored; clippy all CLI targets
  with warnings denied and CLI build passed (Rust 1.90).
- Local transport tests: 17 passed with mocked network, including timeout,
  uncertain settlement, reserve refusal and uninstrumented-broker rejection.
- Existing engine-run suite: 11 passed, nine existing skips using temporary
  loopback fixtures. Sandbox initially refused fixture binding; approved exact
  rerun passed. No real inference was invoked.
- Integration cases exercise initialize/context/proposal/execution, schema/cutoff
  rejection, unchanged pre-action features, budget ceilings, unknown charging,
  deduplication, restart and unchanged factual roots.
- Preserved real RTS history copy: three case results remain unknown, context
  counts null, legacy resource accounting inactive. No historical pass/fail,
  inspection, token or cost evidence was fabricated.
- Logs: `target/project-telemetry-{tests,clippy,build}.log`,
  `target/project-budget-{python,run-python}.log`;
  existing-evidence output: `target/project-features-rts-history.json` and
  `target/project-budget-rts-history.json`.
- Independent review identified unknown-cost refunds and overconfident case
  outcomes; both were corrected and re-reviewed before commit.

## Remaining concrete dependency

Native `features@v1` models cannot consume this adapter schema. A matching versioned
dataset/model contract and validated artifact are still required before numeric
prediction or common-event labeling. Legacy and uninstrumented resource coverage
remain unknown. Supplied usage declarations alone do not establish independent
eligibility. Independently reviewed, deduplicated episodes must also satisfy the
existing close-plus-seven-day/as-of policy; their eligible count is unknown.
The 151 qualified corpus tasks are unchanged and are not 151 mature episodes.
No model call, external transmission or target-repository edit occurred here.

The next evidence layer is now available as a [validated receipt inventory](engine-project-receipt-datasets.md).
It preserves CAS and causal prediction bindings but deliberately cannot enter the
native fitter; see that document for the exact remaining integration work.
