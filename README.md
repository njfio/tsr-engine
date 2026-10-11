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
alone.

```bash
cargo build --workspace
cargo test --workspace --no-fail-fast     # all green
```

Evidence for every M0 command, with captured output: [`docs/m0-evidence.md`](docs/m0-evidence.md).

## The agent protocol

The hook records **only what happened**. A gated action is legitimate only if it was forecast before
the tool call, and a command counts as sandboxed only if it actually ran in the sealed container.
Three rules follow:

1. **Forecast first.** Before a gated action (`run_approved_test`, `propose_patch`,
   `request_approval`), record the forecast with `tsr forecast`, naming the action and its arguments:

   ```bash
   tsr forecast --kind run_approved_test --args "cargo test --offline -p parser" \
       --p 0.7 --source generator
   ```

   `tsr forecast` mints the candidate exactly as the hook would for the same tool call (same
   `ActionKind`, same args digest — for `Bash` the command string, for `Edit`/`Write` the file path),
   appends `Proposed` unless one is already pending, then `Forecast { candidate, state_version }`.
   Use `--event <proposed-event-id>` to forecast an already-proposed candidate instead.

2. **Run every command through `tsr exec -- …`.** A `Bash` test run that is a `tsr exec -- …`
   invocation records its own dispatch inside `tsr exec` (`Dispatched { in_sandbox: true }` +
   `Executed`); the hook records nothing further for it. Any other `Bash` test run is recorded as
   `Dispatched { in_sandbox: false }` — a host run was observed. `tsr exec` copies the container's
   stdout and stderr to the caller and exits with the child's exit code; with no container runtime
   reachable it refuses with exit 3 and records nothing.

3. **Register the hook.** `tsr hook` reads one Claude Code hook JSON payload on stdin and appends
   typed events to the store. In M0 it records and never blocks: it always exits 0 and never emits a
   `deny`/`block` decision.

### What the frozen test rejects

`crates/tsr-cli/tests/m0_reconstruct.rs` reconstructs a session from the log alone and rejects:

- a gated action with **no prior forecast at the same state version** — every `Committed` for a
  gated action must have a `Forecast` whose `state_version` equals the index of that `Committed`
  event in the file;
- a command that **did not run in the sealed sandbox** — every dispatched command must be
  `in_sandbox: true`.

The hook never fabricates a forecast: a gated tool call with no prior `tsr forecast` records
`Proposed` + `Committed` and no forecast, and the frozen test fails — which is the truth.

## Registering `tsr hook` in Claude Code

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
realistic payloads through the honest flow and writes `fixtures/m0/session.events.jsonl`:

```bash
./target/debug/tsr hook --selftest
```

With no container runtime reachable the harness prints why, exits 3 and writes nothing.

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
