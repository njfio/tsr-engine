# tsr-engine

The TSR Decision Engine: a Rust controller that makes an AI coding agent run a full reflective
cycle — state what is wanted and what would count as return, generate possibilities, commit with a
frozen expectation, receive the return, and let the difference change future behaviour. Rust owns
state, authority, history, evaluation and release; a generative model proposes and never authorizes.

- Spec: [`docs/PRD.md`](docs/PRD.md) (PRD v0.3.1). Start with B1, B2, B2.1, then your milestone.
- Rules for agents and people working here: [`AGENTS.md`](AGENTS.md).
- Milestones: `docs/PRD.md` §B11. Swarm lane briefs: [`specs/lanes/`](specs/lanes/).

## Status

M0 landed. The contract crate (`tsr-domain`) compiles with its invariant tests; the store enforces
append-only in the database; the CLI records hook payloads, runs commands in a sealed sibling
container, opens/statuses/exports tasks, records forecasts, and reconstructs a session from the log
alone. `crates/tsr-cli/tests/m0_reconstruct.rs` is green with its assertions unchanged.

```bash
cargo build --workspace
cargo test --workspace --no-fail-fast     # all green
```

Evidence for every M0 command, with captured output: [`docs/m0-evidence.md`](docs/m0-evidence.md).

## Registering `tsr hook` in Claude Code

`tsr hook` reads one Claude Code hook JSON payload on stdin and appends typed events to the store.
In M0 it records and never blocks: it always exits 0 and never emits a `deny`/`block` decision.

Add the hook to your Claude Code settings (`~/.claude/settings.json`, or a project
`.claude/settings.json`). Point `command` at the built binary (or an installed `tsr` on `PATH`):

```json
{
  "hooks": {
    "PreToolUse": [
      { "matcher": "*", "hooks": [ { "type": "command", "command": "/path/to/tsr hook --stage PreToolUse" } ] }
    ],
    "PostToolUse": [
      { "matcher": "*", "hooks": [ { "type": "command", "command": "/path/to/tsr hook --stage PostToolUse" } ] }
    ],
    "SessionStart": [
      { "hooks": [ { "type": "command", "command": "/path/to/tsr hook --stage SessionStart" } ] }
    ],
    "Stop": [
      { "hooks": [ { "type": "command", "command": "/path/to/tsr hook --stage Stop" } ] }
    ]
  }
}
```

The store location is taken from `TSR_DB` (default `.tsr/tsr.db` under the working directory). To
see the recorder work end to end without Claude Code, run the self-test harness, which drives
realistic payloads through the hook and writes `fixtures/m0/session.events.jsonl`:

```bash
./target/debug/tsr hook --selftest
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
