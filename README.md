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
