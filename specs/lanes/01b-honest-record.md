# Lane 01b — M0 repair: the record says only what happened

**Workspace:** `tsr-engine` on branch `lane/core` (M0 landed at `090a8cd`; `main` merged in).
**Owns:** `crates/tsr-cli` (hook, exec, forecast, selftest, tests), `docs/m0-evidence.md`, `README.md`.
**Reads first:** `AGENTS.md`; this file; `crates/tsr-cli/src/hook.rs`, `exec.rs`, `lib.rs`
(`reconstruct`), `tests/m0_reconstruct.rs` (frozen), `tests/m0_harness.rs`; `docs/PRD.md` §B4, §B5.

## Why

The merge-time review of M0 found two shortcuts in `crates/tsr-cli/src/hook.rs` that make the
exit evidence hollow. `tests/m0_reconstruct.rs` passes because the recorder manufactures the
evidence the test looks for, not because the session did the right things:

1. **A fabricated forecast.** `record_tool` appends a `Forecast` (p = 0.5, `Forecaster::Generator`)
   for every gated action, pinned to the index the `Committed` event will occupy. The invariant
   "every gated action has a forecast at the same state version" is therefore satisfied by
   construction. PRD §B5 says the forecast comes *before* the action, from the generator or the
   controller; the recorder must never write one.
2. **A fabricated containment claim.** `record_tool` appends `Dispatched { in_sandbox: true }` for
   any `Bash` call whose command starts with `cargo test`, although Claude Code ran that command on
   the host. Only `tsr exec` runs anything in the sealed sibling container, so only `tsr exec` may
   write `in_sandbox: true`. `every_command_ran_in_sandbox` is vacuous as long as the hook asserts it.

A log that says what happened is the whole point of M0 (AGENTS.md invariants 3 and 5). After this
repair, a session passes the frozen test only if the agent really forecast before each gated action
and really ran every command through `tsr exec`.

## Goal (the text handed to `ds-swarm run`)

Repair milestone M0 of the TSR Decision Engine so the hook records only what happened. Read
AGENTS.md, then specs/lanes/01b-honest-record.md (the reasons), then crates/tsr-cli/src/hook.rs,
exec.rs, lib.rs (reconstruct) and tests/m0_harness.rs. Deliver, in this order:
(1) `tsr forecast` is how a forecast enters the log, BEFORE the tool call, naming the action:
`tsr forecast --kind <run_approved_test|propose_patch|request_approval> --args <string> --p <0..1>
--source <generator|controller|operator>`; keep `--event <proposed-event-id>` as the alternative
for an already-proposed candidate. It mints the Candidate exactly as the hook would for the same
tool call (same ActionKind, same args digest — factor one shared digest function and use it from
both paths; for Bash the digest is over the command string, for Edit/Write over the file path, as
the hook does today), appends `Proposed { candidate }` unless a pending Proposed with that id
already exists, then appends `Forecast { candidate, state_version }` where state_version is the
index the next event will occupy, i.e. the number of events in the task after the Forecast itself
is appended. The hook never constructs a Forecast — delete that code.
(2) hook PreToolUse for a gated action: compute the candidate; if a pending `Proposed` with the same
candidate id exists (proposed, not yet committed), record `Committed` only; otherwise record
`Proposed` + `Committed` with no forecast — such a session fails the M0 invariant, which is the
truth. Dispatch: if the Bash command is a `tsr exec -- …` invocation (first tokens `tsr exec`,
optionally with a path prefix on `tsr`), record nothing further — `tsr exec` records the dispatch
itself; otherwise record `Dispatched { in_sandbox: false }` (a host run was observed) and let
PostToolUse record `Executed` / `Unresolved` as it does now. After this change the only
`in_sandbox: true` in `crates/tsr-cli/src` is in exec.rs.
(3) exec.rs: when the task has a Committed-but-not-Dispatched `RunApprovedTest` candidate whose
args digest matches the argv (the hook committed it a moment ago), dispatch that candidate
instead of minting a new one; otherwise mint as now. After the container run, copy the captured
stdout and stderr to the caller's stdout and stderr and exit with the child's exit code, so an agent
running `tsr exec -- cargo test …` sees the result. Read the container image from `TSR_EXEC_IMAGE`
(default unchanged, `docker.io/library/debian:stable-slim`). Refusal semantics unchanged: no
runtime → exit 3, nothing recorded.
(4) `tsr hook --selftest` produces the fixture by the honest flow and never fabricates:
SessionStart; two Reads; `forecast(run_approved_test, "cargo test --offline -p parser", 0.7)`;
PreToolUse Bash `tsr exec -- cargo test --offline -p parser` (hook: Committed only), then
`exec::run` for that argv in the real container (Dispatched { in_sandbox: true } + Executed), then
the matching PostToolUse; `forecast(propose_patch, "crates/tsr-cli/src/lib.rs", 0.6)`; PreToolUse
Edit (Committed); `forecast(run_approved_test, …, 0.6)`; the second run the same way; Stop. With no
container runtime reachable the selftest prints why, exits 3 and writes nothing. You cannot run it
in your sandbox (no nested runtime); the operator regenerates `fixtures/m0/session.events.jsonl` on
a host with Docker. Do not edit or commit that fixture yourself — leave it as it is.
(5) tests: in `tests/m0_harness.rs` the byte-for-byte reproducibility test skips with a printed
reason when `exec::Runtime::detect()` is `None`; add tests, driving real payloads through the hook
(unit or integration), for: forecast-then-tool-call yields Proposed, Forecast whose state_version
equals the index of the Committed event, Committed — and no second Forecast; a gated tool call with
no prior forecast yields Proposed + Committed and makes
`every_gated_action_has_prior_forecast_at_same_state_version()` false; a raw `cargo test` Bash yields
`Dispatched { in_sandbox: false }` and makes `every_command_ran_in_sandbox()` false; a
`tsr exec -- …` Bash yields Committed and no Dispatched from the hook; `tsr forecast --kind … --args …`
and the hook compute the same candidate id for the same tool call. `tests/m0_reconstruct.rs` stays
byte-for-byte unchanged; it may be red until the operator regenerates the fixture — say so in
docs/m0-evidence.md.
(6) docs: rewrite docs/m0-evidence.md around the new commands and their captured output (mark the
fixture section "regenerated by the operator on a host with Docker"); in README document the agent
protocol — forecast first with `tsr forecast`, run every command through `tsr exec -- …`, register
the hook — and what the frozen test will reject. No tsr-domain change should be needed; if one is,
it is additive and in its own `contract:` commit. Conventional commits; fmt and clippy clean.

## Definition of done (`--dod`)

cargo build, cargo fmt --check and cargo clippy --workspace --all-targets -- -D warnings pass.
cargo test --workspace --no-fail-fast: every test passes except that tests/m0_reconstruct.rs may be
red until the operator regenerates the fixture, and the selftest reproducibility test skips (with a
printed reason) where no container runtime is reachable. `grep -rn "in_sandbox: true"
crates/tsr-cli/src` matches only exec.rs. hook.rs constructs no Forecast. `tsr forecast --kind
run_approved_test --args "cargo test --offline -p parser" --p 0.7 --source generator` followed by a
PreToolUse Bash payload for that command records Proposed, Forecast, Committed with the forecast's
state_version equal to the Committed index, and nothing else. `tsr hook --selftest` without a
runtime exits 3 and writes no fixture. fixtures/m0/session.events.jsonl and
tests/m0_reconstruct.rs are unchanged by this run. docs/m0-evidence.md and README updated.

## Out of scope

Policy gating (tsr-policy decides; the hook still never blocks). The real Claude Code session on
rts (the operator records it after this lands). Any tsr-store schema change.

## Acceptance commands (operator, on a host with Docker)

```bash
cargo build --workspace && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast        # m0_reconstruct red until the fixture is regenerated
./target/debug/tsr hook --selftest            # regenerates fixtures/m0/session.events.jsonl for real
cargo test --workspace --no-fail-fast        # now all green, assertions unchanged
grep -rn "in_sandbox: true" crates/tsr-cli/src   # exec.rs only
```

## Suggested run flags

```
--max-tokens 600000000 --deadline-secs 21600 --max-agents 4 --reviews security,architect,correctness
```
