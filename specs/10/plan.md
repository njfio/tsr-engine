# Plan #10

## Approach

Add one opt-in path. Old routes stay for replay, but opted-in runs don't use them.

- **`tsr-domain`** (own `contract:` commit): two additive event kinds.
  - `Reading { task, want, sense, belief, expect, expected_return, falsifier }`, recorded by the controller with `Source::Rules` over `Generator`-origin text.
  - `Returned { task, reading: Option<EventId>, from: Source, text, evidence: Option<Hash> }`, where `from` is `Operator` or `Rules` only.
- **`tsr-ledger`**: `record_reading` and `record_return` run the mechanical checks. `fold` opens the latest reading's want as a `Generator`-origin preference with a `from_reading` lineage field, and supersedes the task's previous reading want. `arrival_status` and all reviewed paths stay as they are.
- **`tsr-cli` project `cycle.rs`**: an opted-in cycle runs reading, then proposal. It refuses development forms. A new operator command records a return.
- **`scripts/engine-project-cycle.py`, `engine-browser-adapter.py`**: admission opt-in `reading@v1`, like `correction_replay`; a reading prompt and strict schema; a shorter proposal prompt with no menu, no `development_declined`, and no development half of the deliberation envelope; `max_calls` raised to 3 per decision when opted in.
- **Request view**: recent readings beside their returns, with an omitted count, computed by fold. No stored weights (invariant 6).
- **Docs**: PRD amendment recording the author's words, B2 and B6, invariant 8, and `docs/development-decision-repair.md`.

## Interfaces

Reading response:

```json
{"want": "text|null", "sense": "text|null", "belief": "text|null", "expect": "text|null",
 "expected_return": {"from": "text", "what": "text"} , "falsifier": "text"}
```

`expected_return` and `falsifier` may be null only when `want` is null. Each text field has a byte bound. The reading prompt supplies the brief, the current source, and recent readings and returns. It names no forms and doesn't use the book's terms as instructions.

## To confirm first

- That `Preference` and its users accept a want with no contributors. Provisional wanting requires them. A reading deliberately doesn't. If downstream code assumes contributors, keep readings out of `Preference` and give them their own view.
- That nothing in the opted-in cycle's continuation or closure requires an admitted development, including `project_desire_admit_unlocked` and `project_relational_admit_unlocked` callers.
- Whether the operator return can reuse the `feedback_events` admission or needs its own command.
- Which controller-recorded checks produce returns. Start with test and browser check results.

If any of these changes the design, this plan and the spec are updated before code.

## Risks

- Hash-pinned launches: frozen experiments don't opt in and keep their manifests.
- Wire change: two additive event kinds. Old binaries can't read logs containing them.
- Budget: an opted-in run makes more calls per decision. Its admission states that.

## ADR

`docs/architecture/adr-001-single-reading.md`, written with the implementation.
