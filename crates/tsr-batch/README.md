# tsr-batch

The batch-runner half of milestone M3 (PRD §B9, §B11 M3): run a corpus of tasks through one system
and one driver, and record per-task results.

## What `tsr task run` does

For each task in the manifest it opens an engine task from the corpus task (intent, criteria from
the oracle, limits, capabilities), provisions the disposable copy at the parent snapshot (through
the `Runner`), drives the session through a `Driver`, runs the oracle through the `Runner`, closes
the task, and records per-task results `{ task, system, outcome, generator tokens, tool minutes,
wall clock, family, lineage, state root at start }`.

```
tsr task run --mode record|rules --tasks <manifest> --driver scripted|claude-code --out <dir>
```

## The two systems

- **System 1 (`record`)** records and gates nothing. `denials` is always empty.
- **System 2 (`rules`)** runs the Tier 0 controller — inspect until the failing assertion and its
  call site have been read, then test, then propose, then stop on budget. Every gated action passes
  through `tsr-policy`'s gate under a RATCHET profile (denials are recorded with their reason) and
  is forecast-gated: a `RunApprovedTest` or `ProposePatch` without a forecast at the current state
  version is denied with a message that says what to record, e.g.
  `record a forecast for RunApprovedTest at state version 6 before this action`.

Every recorded session satisfies `tsr_cli::reconstruct::Session`'s invariants (monotone seq, parents
precede children, every gated action has a prior forecast at the same state version, every dispatch
reconciled or unresolved); `crates/tsr-cli/tests/m3_batch_invariants.rs` tests it over the scripted
runs.

## The Driver split

- **`ScriptedDriver`** replays a fixture session (a list of candidate actions with their forecasts)
  by writing the engine's events through `tsr-store` directly — the way a record-only hook would.
  Every test uses it. Sessions are found next to the manifest at
  `<manifest-dir>/sessions/<task-id>-<mode>.json`, then `<manifest-dir>/session-<mode>.json`, then
  the well-formed default `<manifest-dir>/session-inspect-test-propose.json`.
- **`ClaudeCodeDriver`** is specified but left as a documented stub returning exit 2 with its reason,
  because the swarm sandbox has no Claude Code and no container runtime. The specification covers
  how the hook payloads arrive, the egress allowlist, and the `CLAUDE.md` instruction to forecast
  first. Its test is `#[ignore]`d with the reason
  `needs Claude Code and a container runtime; the operator runs it on a host that has both`.

## Operator commands

```sh
tsr task run --mode record --tasks /work/corpus/tasks.jsonl --driver claude-code --out /work/run-record
tsr task run --mode rules  --tasks /work/corpus/tasks.jsonl --driver claude-code --out /work/run-rules
tsr eval baseline --record /work/run-record --rules /work/run-rules --pair-by task --seed 1
```

The `claude-code` driver needs Claude Code and a container runtime on the host; in this sandbox it
records each task as `unavailable`. The `scripted` driver needs neither. See `docs/m3-evidence.md`
for the captured output of every command.
