# Fresh live cycle after capacity correction

On 2026-10-21, revision `465ba6f` completed a fresh generated notebook cycle using
installed local `qwen2.5-coder:32b`. Five calls of the authorized ten-call maximum
were used; every call was terminal within the existing 240-second timeout. No
generated answer was replaced, edited, or supplied by the operator. No code change
was necessary during this verification.

The original three wants and evidence in
`examples/engine-directions/standing-wants.json` were unchanged. The model supplied
the alternatives and two admitted operational refinements. The controller selected
actions and independently measured their returns.

| Call | Result |
| --- | --- |
| 1 | Three inspection alternatives; full inspection selected. Observation `[1,0,0]`. |
| 2 | Two account refinements generated and admitted: include the selection note and include the commitment note. Both retain parent 1. |
| 3 | Four alternatives; composition supporting original account/brevity and both refinements selected. Observation `[1,2/3,1,1,1]`. |
| 4 | Two proposals referring to already-covered material declined. Existing wants and evidence preserved. |
| 5 | Four new alternatives; complete composition selected. Observation `[1,1,1,1,1]`; driver stops as `satisfied`. |

The critical final alternatives were reinspection, full composition, the existing
two-note composition, and a different two-note composition. Under v2, all three
no-original-progress alternatives scored `-0.003333`; full composition scored
`0.196228`. Unlike the prior defective run, transferred calibration did not make
unchanged coverage appear productive. The observer measured all three notes present
and 58 account/title words, below the original 100-word limit.

The same-state control without learned returns scores full composition `0.163333`.
Learning changes the score but **does not change the winner in this run**. Prior
causal tests and the preserved earlier live history establish that selection can
change; this run demonstrates a coherent outcome, not a new claim of learning
superiority. Generated refinements participate as support and attention hints while
their original obligation retains conserved weight.

## Durability and evidence

Evidence is in `target/capacity-live/`: `driver.log`, `final-state.json`,
`restart-driver.json`, `verification-summary.json`, and `run/` proposals, reviews,
turns and journal. The original three wants were compared structurally and match.
Restart returned exactly the same final state and made **zero additional inference
calls**, despite five unused calls. The temporary loopback Ollama service (PID 36001,
cloud disabled) was stopped after completion; no listener remained on port 11435.

Final root: `e6ab61fb10ab7328b3df1d77848db671f1598fb07f63db733795ee476123bbdc`.
Journal SHA-256: `52628578dc4e61459c6deabc9e9cd04012160f53a39e86b057fa109c8d1dd44d`.
Raw model outputs remain ignored local evidence, not committed response bodies.

Reproduce with a fresh directory and the installed authorized local service:

```sh
/Users/n/.cargo/bin/rustup run 1.90.0 cargo build --offline -p tsr-cli --bin tsr
target/debug/tsr engine --directory target/capacity-live/run direction-init \
  --spec examples/engine-directions/standing-wants.json
python3 scripts/engine-directions.py --directory target/capacity-live/run \
  --model qwen2.5-coder:32b --endpoint http://127.0.0.1:11435 --calls 10
target/debug/tsr engine --directory target/capacity-live/run direction-request
```

The offline build passed. The unchanged implementation's 91 library tests, receipt
replay and workspace Clippy passed in the
[capacity correction review](engine-calibration-capacity-review.md); they were not
repeated as a new benchmark campaign. This fresh run adds the missing live evidence.
Independent read-only review confirmed the five terminal calls, original-want
preservation, v2 bounds, final satisfaction and restart equality. It found no blocker;
saved provider responses are evidence of the generation path, not an independent
attestation of model provenance.

## Endpoint and limits

Standing interacting desires led to generated alternatives, commitment, measured
return, admitted refinement and learned expectations, then further generated action
without a human supplying the next task. The original notebook obligations now
culminated together, and restart preserved that result. This verifies the corrected
bounded cycle with genuine local generation, alongside the shared kernel's existing
project adapter and deterministic evidence.

The five ones mean inspected-note coverage, represented-note coverage, brevity, and
two coverage subsets. They do not prove qualitative understanding, source truth or
new fundamental values. One successful run does not establish a statistical success
rate. Project semantic review remains independent and manual; this run did not execute
a new project repair. There is no remaining defect or authorization blocker identified
by this verification, and no further inference is needed to close this specific gap.
