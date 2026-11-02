# Real-model synthetic engine demonstration

On 2026-10-18, the integrated engine completed a real CheaperInference
`deepseek-v4.1-flash` call and a sealed coding cycle successfully. It used only the
new synthetic addition fixture. No private repository corpus, personal data,
unrelated source, sessions, environment values or credentials entered the prompt.

[Payload preflight](engine-live-synthetic-preflight.json) records the exact outbound
body hash and 3552-byte size. The fresh engine had no previous patch/outcome history.
Its source was `combine(a,b) { a - b }`; its protected probe checks `(2,3) -> 5` and
`(7,0) -> 7`. The admitted goal, criterion, source, probe and state were inspected
before dispatch. Authentication used the already configured key normally; it was
not displayed or copied.

The user-authorized limits were two steps, 200000 tokens, $1 additional, and the
existing $100 aggregate cap. The old fixed $2 reservation would prevent any call
under that ceiling. The engine now uses the shared broker's optional request-sized
reservation: ten times the conservative cost bound, rounded up to cents. The bound
uses actual serialized request bytes plus 8192 framing tokens and the existing
16384-output-token upper bound, at the already capped prices. Legacy broker callers
retain their existing $2 reservation. The live request's bound was $0.023184 and its
reservation was $0.24; unknown charges still retain that reservation and stop.

## Exact executed command

From the worktree root, after creating and inspecting the fresh generated-edit
fixture:

```sh
python3 scripts/engine-run.py \
  --directory target/engine-live-synthetic-75kilpne --approved-live \
  --ledger "$PWD/target/m3-provider-cost.jsonl" \
  --max-steps 2 --max-usd 1 --max-tokens 200000 \
  --call-timeout-seconds 55 --steps-this-invocation 1 \
  > target/engine-live-synthetic-75kilpne/live-invocation-1.json
```

The one-step checkpoint allowed any second payload to be reviewed before another
call. Success ended the run at step one; no second call or hidden retry occurred.

## Actual result

[Persisted evidence](engine-live-synthetic-result.json) contains the candidate
identity, frozen forecast, bounded probe observations, four sealed run receipts,
ledger delta and final result. It contains no raw provider response body or key.

The model proposed the exact replacement:

```diff
-pub fn combine(a: i32, b: i32) -> i32 { a - b }
+pub fn combine(a: i32, b: i32) -> i32 { a + b }
```

Rust validated the existing scope, source digest, inspection handle and unique
span, then committed the candidate with a frozen 0.5 success estimate. The sealed
runner applied the edit, compiled the protected probe, and executed both inputs
with the workspace mounted read-only. All four commands exited 0. Probe stdout
was exactly `5\n` and `7\n`; the protected probe hash remained unchanged. The only
workspace files were the implementation, protected probe and compiled probe.
No containment failure was observed.

A new CLI process reconstructed the completed outcome, no pending commitment, and
the patch's updated estimate of 2/3. The next request contains the actual observations
and frozen expectation. The loop's result is `success`.

| Accounting | Verified value |
| --- | --- |
| Completion calls | 1 |
| Billed tokens | 1735 |
| Reserved before dispatch | $0.24 |
| Settled additional cost | **$0.000423** |
| Aggregate before | $0.617031 |
| Aggregate after | **$0.617454** |
| Unresolved reservations | 0 |

The shared ledger retained the exact prior bytes and appended one reservation and
one matching settlement. No benchmark or other cohort was resumed.

The actual engine, transport journal, synthetic payload preview and raw execution
receipts remain under `target/engine-live-synthetic-75kilpne`. Read-only inspection:

```sh
target/debug/tsr engine --directory target/engine-live-synthetic-75kilpne request
```

Validation for the reservation change: 16 broker tests and 4 codec tests passed.
Four loopback checks passed for the $1 ceiling, settlement, unknown-charge restart
stop, exhausted run budget and aggregate-cap enforcement. The real run above
then exercised the same code path. Results and charges were preserved without
retrying or changing the proposal.

This establishes that the integrated model-backed engine works end to end for
its bounded synthetic domain. It does not establish arbitrary-project capability,
unseen-input correctness or M3 real-corpus acceptance. No push or deployment occurred.
