# Development-decision repair (November 2026)

This change follows the corrected continuous-development test
([result](continuous-game-live-v2-result.md)). In that test, no development
proposal reached review and Luna chose retention four times. The code gates had
already been removed. Reading the engine afterwards turned up two causes that
don't depend on the model. This change repairs both. It doesn't show that a
model will now develop meaning: only a new live test can show that, and none has
run since this change.

## Cause 1: the observations could not resolve any criterion

The protected browser harness and the native verifier accepted at most 12
ordered probe actions: one start click, six keys and five waits. The reviewed
mapping (`experiments/game-wants-v3/mapping.py`) planned six-key traces. Those
reached neither the first fly nor the far bank, so every gameplay finding was
Unknown. Unknown is not unmet, so no Contrast and no observed success could form.
Those are the evidence routes to new wanting, and `choice_summary` correctly
withheld `contrast_preference` and `success_preference`.

Repairs:

- **Higher action bound.** `scripts/engine-browser-harness.cjs` and
  `browser.rs` now accept 64 actions (`MAX_PROBE_ACTIONS`). The recorded report
  bound rose to match: the harness's own report limit (`MAX_REPORT_BYTES`) and
  the native record (`STDOUT_BYTES`, `OUTPUT_BOUND`) are all 32000 bytes or more,
  and a test keeps the two sides equal.
- **Earlier projects keep working.** The previous harness is preserved byte for
  byte as `scripts/engine-browser-harness-v1.cjs`. Projects admitted with it keep
  validating under its own 12-action bound, in Rust (`LEGACY_HARNESS_V1`) and in
  Python (`supported_harness`). New projects get the current harness.
- **Coverage check.** `scripts/engine-observation-coverage.py` checks a stage
  before dispatch:
  - `static` blocks a stage (exit 2) when no planned sequence for a criterion
    reaches the operator's reviewed `min_inputs` for that criterion, or when a
    sequence exceeds the harness bound. Run against the v2 six-key traces, it
    reports both criteria blocked.
  - `observed` flags a criterion whose readout was identical in every trace,
    meaning the stage discriminated nothing for it.

The frozen v2 mapping is unchanged. Its file hashes are bound in the
`continuous-game-v2-launch` manifest.

## Cause 2: retention was the cheap, cautious default

- **Cost.** Retention was a one-line `{retain:reason}`. Development needed the
  deliberation envelope, request-local aliases and extra validators.
- **Wording.** The prompt called the options "equally valid", but almost every
  sentence about development was a limit.

Repairs:

- **Retention may state its development decision (superseded by spec #12).**
  This was first a requirement: a bare `{retain}` got a structural diagnostic
  whenever development was offered. Reading *The Sovereign Mirror* afterwards
  showed that requiring it was Focus pushed against the same gap (ch. 11–12).
  Spec #12 made it optional.
  - `{retain:reason}` is always accepted.
  - `development_declined:reason` is recorded when given, folded into the native
    retention reason under the fixed label `Development considered and declined:`,
    and never required.
- **Balanced prompt.** One shared system message, still identical across paired
  arms, names three choices with none preferred. It states each limit once, in
  ground rules that apply to every choice.
- **Balanced interpretation wording.**
  - Post-action interpretation uses `qualitative-evidence@v4`, which weighs a
    revised expectation and possibilities like retaining both. v1–v3 replay
    unchanged.
  - The initial-context instruction in `qualitative.rs` now says unchanged
    meaning, development and abstention are equally legitimate, "without
    manufacturing or suppressing novelty".

## Diagnosability

`choice_summary.unavailable_development_routes` says why a development route is
absent:

- no current contributors, so provisional wanting has nothing to connect to;
- fewer than two contributors, so no initial relationship is possible;
- the provisional contract was not supplied.

A run without development options can then be told apart from a model that
declined them.

## Limits and operator notes

- **New admissions are required.** `pins()` hashes the changed scripts, so
  existing admissions bound to the old implementation won't run with this code.
- **Allow at least one format correction.** Admit `correction_allowance >= 1`
  for live runs. Otherwise a model that returns a bare `{retain}` stops the
  session at the first correction instead of receiving the diagnostic.
- **Results aren't comparable with v2.** The prompt, the interpretation contract
  and the retention protocol changed, so results after this change can't be
  compared directly with the v2 result.
- **What would show the repair worked.** A stage that passes the `static`
  coverage check, followed by findings other than Unknown, followed by a
  development proposal or a recorded `development_declined` reason. None of
  these has been run yet.
- **Bounds stay finite.** 64 actions, at most eight readouts, waits of at most
  1000 ms each, and an unchanged per-case deadline (at most 180 s).

## Later change

Spec #10 adds an opt-in that replaces this menu with a single reading. See `docs/architecture/adr-001-single-reading.md`. Runs that don't opt in keep the behavior described here.
