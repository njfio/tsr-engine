# Continuous game v3: prepared, not admitted

This is the observation package for the next corrected-engine test, after the
[development-decision repair](../../docs/development-decision-repair.md). It is
derived from `continuous-game-v2/protocol.json`, whose hash is recorded in
`derived_from`. Nothing here dispatches, admits or authorizes anything.

## What differs from v2, and why

- **Trace length.** v2 used six fixed inputs (12 harness actions) for every
  game. Its traces reached neither the fly nor the far bank, so every finding was
  Unknown. Here, the reviewed mapping declares for each criterion the minimum
  inputs needed to reach a state where it could be judged (`coverage.min_inputs`,
  with a `basis`). Each word is sized from that before any result is seen: at
  least 12 inputs and at most 32, which is 64 harness actions.
- **Gate before dispatch.** `mapping.py` `plan` refuses a batch that
  `scripts/engine-observation-coverage.py static` reports blocked.
- **Check after each batch.** Run
  `python3 scripts/engine-observation-coverage.py observed --selectors <json> <reports...>`
  and record any criterion whose readout never changed. An invariant criterion
  is reported as undiscriminated, never as not_met.
- **Neutral feedback text.** v2's observation feedback ended with "Retention and
  abstention remain valid". Here it names change, development, retention and
  abstention, with none preferred.
- **Engine contract.** Results are not comparable with v2: retention now states
  `development_declined` whenever development is offered, and the wording is
  balanced.

## Before a live run

1. Use a fresh admission. The changed scripts are hash-pinned, and
   `correction_allowance` must be at least 1.
2. After the artifact exists, write the source-bound mapping and have it
   independently reviewed. This includes `coverage` with an honest `min_inputs`
   and `basis` for each criterion.
3. Run `plan` for every batch. If it refuses a batch, do not dispatch that
   batch; record it as not covered.
4. Keep every raw result. Don't add a favorable route after seeing results.

The frozen `experiments/game-wants-v3/mapping.py` is reused for the v1 field
rules and is not modified.
