# tsr-engine

The TSR Decision Engine: a Rust controller that makes an AI coding agent run a full reflective
cycle — state what is wanted and what would count as return, generate possibilities, commit with a
frozen expectation, receive the return, and let the difference change future behaviour. Rust owns
state, authority, history, evaluation and release; a generative model proposes and never authorizes.

- Spec: [`docs/PRD.md`](docs/PRD.md) (PRD v0.3.1). Start with B1, B2, B2.1, then your milestone.
- Rules for agents and people working here: [`AGENTS.md`](AGENTS.md).
- Milestones: `docs/PRD.md` §B11. Swarm lane briefs: [`specs/lanes/`](specs/lanes/).

## Status

M0 skeleton. The contract crate (`tsr-domain`) compiles with its invariant tests; the store enforces
append-only in the database; the CLI names every command group and exits 2 where a milestone has
not landed. One test is red by design — `crates/tsr-cli/tests/m0_reconstruct.rs` — and M0 is done
when it is green with its assertions unchanged.

```bash
cargo build --workspace
cargo test --workspace --no-fail-fast     # 10 pass, 1 red (M0)
```

## Review and reflect

M2 closes the loop: a task's returns are labelled, and one closed episode is rendered in the book's
order. The commands are `tsr review` and `tsr reflect`.

```bash
# Label a task's returns: accepted | rejected --reason <text> | disputed.
tsr review <task> --store tsr.db
tsr review <task> rejected --reason "the diff touched the schema" --store tsr.db

# Sample greens for review at a rate, with the inverse-probability weight recorded on the record.
tsr review <task> --sample 0.25 --seed 7 --store tsr.db
# The false-accept rate (green but rejected), weighted and unweighted, per family.
tsr review <task> --family alpha --stats --store tsr.db

# One closed episode on one screen: Desire, Imagination, Belief, Return, and the residual
# (Surprise per forecaster and Contrast per criterion, side by side, never added).
tsr reflect <task> --store tsr.db
# Each forecaster's calibration curve (reliability bins and Brier), including the operator's.
tsr reflect --family alpha --store tsr.db
# The same content as a document, for tests.
tsr reflect <task> --json --store tsr.db
```

The thirty episodes under `fixtures/m2/episodes/` are **synthetic**: a seeded generator
(`crates/tsr-cli/tests/m2_episodes.rs`) drives the store API through
open -> propose -> forecast -> commit -> dispatch -> execute -> close -> review across three families
with different generator calibration. The real thirty come from operator sessions after M3's batch
runner. `fixtures/m2/store/tsr.db` is the same corpus as a store the CLI can fold. See
[`docs/m2-evidence.md`](docs/m2-evidence.md) for every command and its captured output.

## Layout

```
crates/tsr-domain   the contract: facts, views, traits (PRD B6)
crates/tsr-store    SQLite index over the content-addressed, append-only log
crates/tsr-cli      the `tsr` binary (operator CLI + hook) and reconstruct
fixtures/           recorded sessions (produced by the tool under test, never by hand)
specs/lanes/        swarm briefs: goal, definition of done, acceptance commands
docs/               the PRD and evidence pages
```

## License

MIT OR Apache-2.0.
