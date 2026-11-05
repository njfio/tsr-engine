# Want loop v1: built, not yet run live

Spec: [specs/11/spec.md](../specs/11/spec.md). Launcher: `experiments/want-loop-v1/`.

One launch runs until it stops by itself. There are no stages to prepare. A proposal can state a `need`; the first need reveals a local `play` that runs the current source with the model's own controls, and the result comes back beside the reading. A later need goes to the operator. Retaining with nothing new, abstaining, or reaching a spending cap, if the launch sets one, also ends the run.

## What is known

Everything below comes from authored transports and the real local browser harness. No model has seen this prompt, so nothing here says what a model will do or want.

- Need, play, retention, cap and restart paths run through the real native controller.
- A set cap is checked before every dispatch, including inspections, format corrections and recovery. A call under way can overshoot by that one call.
- The harness ran on the r4 game offline and returned its inline-style error. That doesn't show the game plays.

## Limits

- Offering `need` is a prompt, so a need shows that a model used an offered form.
- A launch with `none` has no limit on spending, calls or time. Identical plays, alternating inspections and approved edits are limited only by the model stopping. What's left: 12 review rejections, 20 format corrections, 1 provider retry, a block on unknown usage, about 1 MB per request, 512 MiB of journal, and a `STOP` file in the cycle directory. The request and journal limits and a second provider failure stop without a terminal record, so `report.py` shows `stop: null`. `run.py check` shows the spend.
- The first need unlocks `play` whatever it says, so a rote need skips the want.
- Nothing reads a need's meaning, so a later need ends the run even when `play` could have met it.
- A control that fails makes the harness exit 2 and drop what came before it. Fixing that means changing the protected harness.
- The reviewer sees play results, which hold page text from code the author wrote.
- Independent review happens once, at launch.
- The readouts come back at most 256 characters per check, so a broad selector shows little.
- `cargo-mutants`, `proptest`, `cargo-fuzz` and the `contracts` crate aren't used here: this is Python. Hand mutants and a deterministic 20k random-value test stand in.
- No `tracing`: the Python engine records to its journals instead.

## To launch

```bash
TSR_LIVE_LABEL=r1 TSR_SPEND_CAP_MICROUSD=<cap|none> TSR_LIVE_COMMIT=$(git rev-parse HEAD) python3 experiments/want-loop-v1/prepare.py <authorization.json>
```

Then record the independent review as `target/want-loop-r1/tsr/binding-review.json`, and run `experiments/want-loop-v1/run.py run`. `report.py` prints the counts afterward. A launch needs the operator's approval first.
