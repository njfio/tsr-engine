# ADR-001: A single reading each cycle

Status: Accepted by the author on 2026-11-04. Spec: `specs/10/spec.md`.

## Context

Generated wanting reached the system through a menu of forms, needed an independent model review to open, and never had to meet a return. *The Sovereign Mirror* says the crossing into wanting is free, and that the filter is not a judge one party runs on another (ch. 10). It puts Tune before Focus (ch. 7, 12), and it says what goes out comes back from someone real (ch. 13, 21).

The author chose "Cross freely", then "I think it should be a single reading. Simplified and improved."

## Decision

Under the admission opt-in `reading@v1`:

- Each cycle starts with one reading, written before any proposal. Its request carries no action contract.
- A reading holds `want`, `sense`, `belief` and `expect`, any of which may be null. A want also needs `expected_return` and `falsifier`. These are the only conditions on it, and they are mechanical.
- A want crosses when it is recorded, with no reviewer. Only the latest want is current. It continues only if it is stated again.
- Returns come from the operator (`tsr engine --directory <dir> project return --text "..."`) or from controller-recorded checks. The engine shows them beside readings and gives no verdict.
- Development forms and the decline reason are gone from the proposal turn, and generated development is refused.

Readings live in their own view (`tsr_ledger::reading_view`), not among ledger preferences. Open preferences feed refinement parents and `Open | Fulfilled` entries feed acceptance aims, so a want stored as a preference could reach authority with no review.

Artifact review, scope grants, budgets and the sandbox don't change. The old routes stay in the code so old logs still replay, and a run that doesn't opt in behaves as before.

## Consequences

- Two additive event kinds, `Reading` and `Returned`. Old binaries can't read logs that contain them. The preference fold doesn't read them.
- Nothing checks what a want means. Only its provenance is guaranteed.
- Asking for a reading is still an assignment. This removes suggestion, not assignment, and it doesn't produce or detect wanting nobody assigned.
- The six fields can become a new menu filled in by rote. The counts from `project reading-view` may show this, but nothing prevents it.
- Readings enter later prompts, so a model can shape its own later proposals.
- One more model call per decision. Opted-in admissions allow 3 calls per decision rather than 2.
- One current want can't hold several at once, and meaning developments and relationships aren't offered in opted-in runs.
- Opted-in runs stop if the native cycle offers advisory reconsideration, since that route isn't offered. This needs an existing admitted relationship, which an opted-in run can't create but a resumed project could already hold.
- An operator return changes the state root, so it belongs between cycles. A return made during a model call stops that cycle on the source check, which fails closed.
