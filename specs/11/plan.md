# Plan #11

## Approach

Add one opt-in profile. All changes sit in the Python cycle and adapter; the native controller and harness are unchanged.

- `scripts/engine-abilities.py` (new): profile fields, prompt text, forms, shape checks, probe construction, return text, and the Docker runner using the same flags as `discover.py`.
- `scripts/engine-project-cycle.py`: an optional `abilities` callback. Adds the need, play, retention and cap stops. `ability_` journal keys count as decisions, so each one is followed by a new reading.
- `scripts/engine-browser-adapter.py`: admission fields, `abilities_callback`, need and play effects, forms and contracts in the proposal body, shape and decode paths.
- `scripts/engine-native-cycle.py`: passes the callback in `build_cycle`.
- `experiments/want-loop-v1/`: `prepare.py` and `run.py`. One launch. `run.py` also checks the review's decision and hashes, which fixes a limitation the r4 review named.

## Interfaces

- Admission: `abilities_profile`, `spend_cap_microusd`.
- Proposal responses: `{need:{what,because}}`, `{play:{actions,checks}}`.
- Terminals: `need_outside_scope`, `spend_cap`.
- Returns use the existing `project check-return`, `from: rules`, so no wire or schema change. Text is bounded to `READING_TEXT_MAX`; the full report's digest is the evidence and the report stays in the journal.

## Risks

- `run_execution` needs execution room in the campaign. Unlimited runs have it; capped fixtures must allow for plays.
- The bounded reading view shows a limited number of readings, so old play returns drop out of the prompt, though not the log.
- The harness probe limit is 8192 bytes. Oversized plays are refused in the shape check.

## ADR

No new dependency or wire format. This changes what ends a run, which the spec records. An ADR isn't needed unless the stops change after human review.
