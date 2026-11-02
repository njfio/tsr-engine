# Live verification after shared-kernel migration

[Follow-up capacity review](engine-calibration-capacity-review.md) establishes and
corrects the impossible-progress defect behind the counterproductive choices below.
It preserves this historical run and verifies the correction with offline replay.

Verified on 2026-10-21 from `ff30187`, with one provider-schema correction:
support-array cardinality is capped by the eligible support set. The Rust controller
still rejects duplicates; generated answers were not edited or replaced.

The migrated bounded decision cycle operated end to end with installed local
`qwen2.5-coder:32b`. This run **did not achieve every standing desire**. It exhausted
its ten-call budget with account coverage at two-thirds. Functional cycle evidence
must not be described as reliable goal completion or semantic understanding.

## Reproduction and evidence

From the repository root, using a fresh output directory and an authorized installed
local Ollama service (this run used loopback port 11435 with cloud disabled):

```sh
/Users/n/.cargo/bin/rustup run 1.90.0 cargo build --offline -p tsr-cli --bin tsr
target/debug/tsr engine --directory target/shared-kernel-live/run direction-init \
  --spec examples/engine-directions/standing-wants.json
python3 scripts/engine-directions.py --directory target/shared-kernel-live/run \
  --model qwen2.5-coder:32b --endpoint http://127.0.0.1:11435 --calls 10
target/debug/tsr engine --directory target/shared-kernel-live/run direction-request
```

The first invocation stopped after six calls on repeated invalid offers. After the
schema correction, the same command resumed the same journal with four calls remaining.
The third invocation, after exhaustion, made no inference call and returned identical
state. No new batch was granted. Every inference call was terminal; no timeout or
uncertain call was retried. Per-call timeout remained 240 seconds.

Local evidence is under `target/shared-kernel-live/`: `driver.log`,
`resumed-driver.log`, `final-state.json`, `restart-driver.json`,
`verification-summary.json`, and `run/local-directions.jsonl`. Provider proposals
and action/refinement receipts remain in the ignored run directory, not in this commit.
Journal SHA-256: `204fdbd9db0d81554bf7ed358647fad7e8f20d430846db8265a498b947827f0a`.
Final state root: `aa859bddea7ba81fef53f892c9b4764a3f83c9cc8cea139372e7614ef9887566`.

## Observed cycle

| Call | Observed result |
| --- | --- |
| 1 | Four generated inspection alternatives; selected all three notes. Original observations moved from `[0,0,0]` to `[1,0,0]`. |
| 2 | Two generated, evidence-grounded account subsets admitted with original parent lineage. |
| 3 | Generated alternatives included both refinements; selected composition used their two notes. Original account coverage became 2/3 and brevity 1. |
| 4 | Two invalid refinements referred to already-covered material; both declined without weakening wants. |
| 5–6 | All alternatives repeated support IDs; offers rejected and state root unchanged. |
| 7 | Schema correction allowed valid alternatives. Selected new composition changed which two notes were included; original coverage did not improve and one refinement reopened. Return recorded no original progress. |
| 8 | No additional refinement admitted. |
| 9 | Next generated composition restored the reopened subset; original coverage still 2/3. |
| 10 | No additional refinement admitted; driver stopped at its frozen budget. |

Four actions and four refinement reviews were recorded, plus two failed offers.
Every action assessment used `conserved-obligations@v1` through the migrated notebook
adapter. Observations come from the separate deterministic notebook observer, not
the generator's predictions or explanations.

The live same-state learning audit removes only learned returns while preserving
observation, desires, alternatives and commitment. At call 7, learning selected index
1 versus index 2 without learning; scores were `[.197895,.204063,.196228]` versus
`[.075625,.075625,.163333]`. This influence was real but counterproductive: the
unlearned winner would have covered all three notes. At call 9 the score maxima
were 3 versus 2, but every unlearned score was negative: the actual counterfactual
policy would abstain. Learning enabled commitment rather than a different executed
alternative; prior successful and failed-progress returns both entered prediction.
These are causal learning observations, not evidence that the learning rule is optimal.

Final observations were `[1,2/3,1,1,1]`: inspected-note coverage, account-note coverage,
word budget, and two generated coverage subsets. No qualitative understanding,
source truth, or new fundamental value was established. The model was not given a
new operator task between turns, nor an authored repair answer.

## Checks and boundary

- Offline Rust build passed; log `target/shared-live-build.log`.
- Affected library checks: 77 CLI + 12 domain tests passed; `library-checks.log`.
- Driver checks: 11 passed, including the single-support duplication regression;
  `driver-checks.log`. `git diff --check` passed.
- Independent review found no blocker in the schema correction. Cardinality does not
  guarantee uniqueness for larger eligible sets; controller validation remains final.
- Restart returned exactly equal state with ten total started calls, preserving the
  unmet desire, refinements and learned-return history.
- Temporary Ollama PID 92841 was stopped after the terminal run. No paid inference,
  external transmission, downloads, publication or original target edits occurred.

The prior broad workspace failures remain qualified in
[the failure investigation](engine-mutant-failure-review.md); this targeted run does
not turn that sweep green. Project evidence remains replayed execution/review evidence;
project semantic review is still an independent operator responsibility.

The shared architecture's live-cycle integration gap is closed. This particular
bounded run's goal-achievement gap is not: model proposals and transferred calibration
failed to converge within ten calls. A further live attempt requires another explicit
local batch; no paid request is needed or pending for this verification. Any follow-up
should examine calibration overshoot and novelty/context weighting using these fixed
receipts before spending more calls, rather than hiding the incomplete result.
