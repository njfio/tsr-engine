# Run the persisted TSR loop

The new `tsr engine` command joins existing event storage, candidates, commitments, forecasts, the preference ledger and the Contrast/Surprise split into a continuing local loop. It needs neither Docker nor a model provider.

```sh
cargo +1.90.0 build -p tsr-cli --bin tsr --offline
./target/debug/tsr engine --directory ./target/my-engine init --batch-size 3
./target/debug/tsr engine --directory ./target/my-engine step
# Exit/restart the process; the next command resumes the same history.
./target/debug/tsr engine --directory ./target/my-engine step --cycles 2
./target/debug/tsr engine --directory ./target/my-engine show
```

`step --json` emits one complete before/after trace per line. `show` includes preference IDs; `engine --directory PATH admit ID` explicitly admits a generated refinement. Initialization refuses to overwrite existing history.

The domain is a deterministic queue simulation: the cheap bulk route delivers only one item, while the checked route delivers the whole batch. A small probe can support an initial commitment without proving full-batch success. The first full return supplies the missing evidence. This deliberately exposes the difference between a proposal, a supported commitment and a successful outcome; there is no branch on cycle number.

The captured run in `engine-demo-output.txt` shows:

| Cycle | Choice | Return | What changes |
|---|---|---|---|
| 1 | Bulk | 1/3 delivered | Negative batch evidence; estimate .5→.333; unmet pressure 0→.5; one pending preference refinement |
| 2 | Checked | 3/3 delivered | Contradicted bulk proposal not admitted; checked estimate .5→.667 |
| 3, after restart | Checked | 3/3 delivered | Prior commitment and evidence retained; checked estimate .667→.75 |

Desire is the standing criterion and accumulated preference text, never a scalar. Intuition proposals read the current commitment, evidence-derived expectations and unmet pressure. Belief is the chosen commitment, with evidence and a forecast frozen beforehand. Returns update the next snapshot. Surprise is forecast residual; Contrast describes the unmet criterion and alone produces the refinement. The derived `unmet_pressure = failures/(1+failures)` is a bounded engineering proxy for the contrast-response signal, not a claim that the software feels emotion. It neither grants authority nor multiplies utility. Repeated refinements are deduplicated; successful surprises add no new preference. The history accumulates without a stored weight.

SQLite transactions commit each complete local cycle atomically. CAS blobs are synced before their referencing events; an interrupted transaction may leave an unreferenced blob but cannot leave half a committed cycle. Reads use one snapshot. Restart validates event identity/chain and referenced evidence. This is a local single-operator store, not a protection boundary against its owner rewriting every file.

`engine::Proposer` is the extension point for bounded route proposals. It receives a reconstructed view and has no store handle. The current domain transition remains local and deterministic; this slice does not attach provider egress, execute arbitrary commands, calibrate a production model, implement a general coding agent, or complete M3 acceptance. The estimates use a simple Beta(1,1) count fold, with observed contradiction controlling admission. A future domain adapter must define its own observations and effects before reusing this loop outside the simulation.

Validation: `cargo test -p tsr-cli --offline --lib --test engine_cli` passes 26 library tests and one independent-process CLI test (six focused engine tests included). The tests cover three cycles/resume, explicit refinement admission, rollback when no proposal has support, forecast ordering, successful Surprise without expansion, evidence corruption and disconnected history rejection. No paid calls were used.
