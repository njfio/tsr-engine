# Plan #10

## Approach

Add a new path rather than changing the old one, so existing logs still fold identically.

- **`tsr-domain`** (own `contract:` commit): one additive event kind for a crossing, recorded with `Source::Rules` over a `Generator`-origin preference.
- **`tsr-ledger`**: a crossing method running only the mechanical checks; `fold` opens a preference on a valid crossing event. `arrival_status` and the reviewed paths stay as they are.
- **`tsr-cli` project `cycle.rs`**: a development choice under the opt-in takes the crossing path, with no `Review` input.
- **`scripts/engine-project-cycle.py`, `engine-browser-adapter.py`**: an admission opt-in `wanting_crossing@v1`, like `correction_replay`; skip the review stage for development only; paired edits still go through review.
- **Request bound**: show a bounded view of crossed wanting with an omitted count, computed by fold. No stored weights (invariant 6).
- **Docs**: PRD amendment recording the author's words, B2 and B6, invariant 8, and `docs/development-decision-repair.md`.

## To confirm first

- Whether meaning development and relationships (`project_desire_admit_unlocked`, `project_relational_admit_unlocked`) can share the path. If not, they get their own task and this spec says so.
- Exact Contrast and success route bindings in `expansion.rs`. Their reviewed aspiration step goes away only under the opt-in.

## Risks

- Hash-pinned launches: frozen experiments don't opt in and keep their manifests.
- Wire change: one new event kind, additive. Old binaries can't read logs containing it.

## ADR

`docs/architecture/adr-001-wanting-crossing.md`, written with the implementation.
