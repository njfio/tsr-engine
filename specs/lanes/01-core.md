# Lane 01 — Core (milestone M0)

**Workspace:** a clone of `tsr-engine` on branch `lane/core`.
**Owns:** `crates/tsr-store`, `crates/tsr-cli`, `fixtures/m0`, `docs/m0-evidence.md`.
**Reads first:** `AGENTS.md`; `docs/PRD.md` §B4 (path A), §B5 (actions and the forecast gate),
§B6 (state and contracts), §B11 (M0 row).
**Exit evidence (PRD B11, M0):** one real session on a disposable copy of rts is fully reconstructed
from the log alone; every command ran in the sandbox.

## Goal (the text handed to `ds-swarm run`)

Implement milestone M0 of the TSR Decision Engine in this Rust workspace. Read AGENTS.md, then
docs/PRD.md sections B4, B5, B6 and B11, then specs/lanes/01-core.md. Deliver, in this order:
(1) `tsr hook --stage <PreToolUse|PostToolUse|SessionStart|Stop>`: reads one Claude Code hook JSON
payload on stdin and appends typed events to the store (`tsr-store`) with content-addressed ids and
parent links, mapping tool names to `ActionKind` as B4 says (Read/Grep/Glob and rts calls → Inspect;
Bash matching the allowlisted test command → RunApprovedTest; Edit/Write → ProposePatch; the
model's own turn → Generate; Stop → Stop). In M0 the hook records and never blocks.
(2) `tsr exec -- <argv>`: runs the command inside a sealed sibling container — Podman or Docker,
`--network none`, read-only root, the workspace as the only writable mount, cgroup limits — and
records `Dispatched { in_sandbox: true }` then `Executed` or `Unresolved`. With no container runtime
reachable it refuses with exit code 3 and records nothing; it never runs the command on the host.
(3) `tsr task open | status | export <id>`: task intake (workspace, intent, criteria, limits), status,
and trace export as JSON containing every event of the task in sequence order.
(4) `tsr forecast --event <id> --p <0..1> --source <generator|controller|operator>`: records a
Forecast event for the current state version (the number of events in the task so far).
(5) `tsr_cli::reconstruct::from_events`: rebuilds a `Session` from an events.jsonl file.
(6) A hook self-test harness (`tsr hook --selftest` or an integration test) that drives realistic
Claude Code hook payloads through the hook — a task opened, two inspections, a forecast, a sandboxed
test run, a patch proposal, a forecast, a second run, a stop — and writes
`fixtures/m0/session.events.jsonl`, so that `cargo test --workspace` passes including
`crates/tsr-cli/tests/m0_reconstruct.rs` with its assertions unchanged. Keep changes to
`crates/tsr-domain` minimal, additive, and in their own commit prefixed `contract:`. Write
`docs/m0-evidence.md` with the commands you ran and their captured output, and document in README
how to register `tsr hook` in Claude Code's hooks settings.

Before any cargo command, run `export CARGO_HOME="$PWD/.cargo-home"` (the sandbox mounts only the
workspace; the cache must live inside it).

## Definition of done (`--dod`)

cargo build, cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings, and
cargo test --workspace --no-fail-fast all pass. tests/m0_reconstruct.rs passes with its assertions
unmodified against a committed fixtures/m0/session.events.jsonl that the hook produced. `tsr exec --
true` exits 3 and records nothing when no container runtime is present, and runs inside a container
when one is. `tsr task export <id>` emits JSON with every event of the task in seq order. No code
path executes agent-provided argv on the host. docs/m0-evidence.md shows the commands and output.
README documents hook registration. tsr-domain changes, if any, are additive and in a `contract:`
commit.

## Out of scope for this lane

Policy gates and approvals (lane 02). The Culmination ledger, reflect view, calibration, corpus
tooling. Network in the sandbox. Any model call from the engine itself.

## Acceptance commands

```bash
cargo build --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
./target/debug/tsr exec -- true; echo "exit=$?"          # 3 without a runtime, 0 inside a container
./target/debug/tsr task export <id> | head -c 400
```

## Suggested run flags

```
--max-tokens 900000 --deadline-secs 10800 --max-agents 6 --reviews security,architect,correctness
```
