# Plan #12

## Approach

One opt-in profile on top of #10 and #11. One additive contract field (`Reading.gap`). Everything else is in the Python cycle, adapter and a new policy module.

- `crates/tsr-domain/src/lib.rs` (`contract:` commit): optional `gap` on `Reading`. It is bounded like the other fields and omitted when unset, so existing readings keep their identity.
- `scripts/engine-process.py` (new):
  - admission fields
  - the notebook prompt and page checks
  - the proposal-prompt addition, forms and envelope
  - `tune`
  - candidate-copy construction, try return text and review evidence
  - want detection over the reading view
- `scripts/engine-project-cycle.py`:
  - before the review call, `trial_run_<n>` runs `ability_try` when the profile is on;
  - after a rejection, `try_returns_<n>` records the try's return beside the reading.
- `scripts/engine-browser-adapter.py`:
  - admission wiring: `tsr_process_profile`, `process_context`;
  - `abilities_revealed` returns `play` from the start, and `need` is refused;
  - `ability_try`, `record_try_returns` (also called after execution), and `latest_play_controls`;
  - notebook reading, the process proposal body, tuned context, try evidence in the review input;
  - `same_view` follows `return_root_` markers, so earlier criticism still applies after a return is recorded.
- Retention: `development_declined` is optional in every profile. Prompt texts and shape checks are updated, and the field is still folded into the reason when given.
- `experiments/tsr-process-v1/`:
  - `prepare.py` takes `TSR_CONTEXT=tuned|full`;
  - `run.py` is the one-launch runner;
  - `report.py` gives counts, tries, detected wants and whether a person's return exists.
- `docs/tsr-process-test.md`: the pre-registered push/ease protocol.

## Interfaces

- Admission: `tsr_process_profile: tsr-process@v1`, `process_context: tuned|full`. The existing `context_profile` key is the capacity profile and is untouched.
- Proposal responses: `{play:{actions,checks}}` from the first turn. `{need:…}` is refused.
- Reading: the notebook page with `gap`.
- Journals:
  - cycle: `trial_run_<n>`, `try_returns_<n>`;
  - adapter: `ability_try_<k>`, `try_evidence_<k>`, `try_recorded_<k>`, `play_controls_<k>`, `return_root_<root>`.
- Returns use the existing `project check-return`, so there is no new wire format.

## Risks

- Each try is an accounted execution. Capped campaigns need room.
- A try of an edit that does not apply runs nothing and says so. It is never repaired.
- With `tuned`, something that mattered may drop out of the prompt. `full` is the control.
