# M0 evidence — the honest record: forecast, hook, exec, selftest

Milestone M0 (PRD §B11), repaired by `specs/lanes/01b-honest-record.md`. The hook records **only
what happened**: a forecast enters the log through `tsr forecast` *before* the tool call, the hook
never fabricates one, and a command that ran on the host is recorded as having run on the host.

Every command below was run from the workspace root with the sandbox's cargo environment:

```bash
export CARGO_HOME="$PWD/.cargo-home"
export PATH="/usr/local/cargo/bin:$PATH"
```

## What landed

| Deliverable | Where |
| --- | --- |
| `tsr forecast --kind <…> --args <…> --p <…> --source <…>` (and `--event <id>`) | `crates/tsr-cli/src/main.rs::run_forecast`, `crates/tsr-cli/src/hook.rs::record_forecast_for_action` / `record_forecast_for_candidate` |
| The one shared args digest both paths use | `crates/tsr-cli/src/hook.rs::args_digest` |
| `tsr hook --stage <PreToolUse\|PostToolUse\|SessionStart\|Stop>` | `crates/tsr-cli/src/hook.rs`, wired in `crates/tsr-cli/src/main.rs` |
| `tsr exec -- <argv>` (sealed sibling container, stdout/stderr passthrough) | `crates/tsr-cli/src/exec.rs` |
| `tsr task open \| status \| export <id>` | `crates/tsr-cli/src/task.rs` |
| `tsr_cli::reconstruct::from_events` | `crates/tsr-cli/src/lib.rs` |
| Hook self-test harness + fixture | `crates/tsr-cli/src/hook.rs::selftest`, `fixtures/m0/session.events.jsonl` |

## The honest flow

A gated action (`run_approved_test`, `propose_patch`, `request_approval`) is forecast **before** the
tool call, and the forecast is what makes the commitment legitimate:

1. **`tsr forecast`** mints the candidate exactly as the hook would for the same tool call — same
   `ActionKind`, same args digest (`args_digest`: for `Bash` the digest is over the command string,
   for `Edit`/`Write` over the file path). It appends `Proposed { candidate }` unless a pending
   `Proposed` with that id already exists, then appends `Forecast { candidate, state_version }`
   where `state_version` is the index the next event will occupy — the number of events in the task
   after the forecast itself is appended.
2. **`tsr hook --stage PreToolUse`** for a gated action computes the candidate. If a pending
   `Proposed` with the same candidate id exists (proposed, not yet committed) it records `Committed`
   only. Otherwise it records `Proposed` + `Committed` with **no forecast** — such a session fails
   the M0 invariant, which is the truth.
3. **Dispatch.** If the `Bash` command is a `tsr exec -- …` invocation (first token's basename `tsr`,
   optionally with a path prefix, second token `exec`) the hook records nothing further: `tsr exec`
   records the dispatch itself. Any other `Bash` test run records `Dispatched { in_sandbox: false }`
   — a host run was observed — and `PostToolUse` records `Executed` / `Unresolved` as before.

After this repair the only `in_sandbox: true` in `crates/tsr-cli/src` is in `exec.rs`.

## The commands and their captured output

### Build, format, lint

```
$ cargo build --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.89s

$ cargo fmt --check && echo "fmt OK"
fmt OK

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.70s
```

### The whole test suite

```
$ cargo test --workspace --no-fail-fast
     Running unittests src/lib.rs (tsr_cli)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running tests/m0_harness.rs
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running tests/m0_reconstruct.rs
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running tests/policy_check.rs
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running unittests src/lib.rs (tsr_domain)
running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running unittests src/lib.rs (tsr_policy)
running 52 tests
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running tests/adversarial.rs
running 17 tests
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running unittests src/lib.rs (tsr_store)
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `tsr forecast` — how a forecast enters the log

`tsr forecast` names the action and its arguments; it mints the candidate the hook would mint for the
same tool call, appends `Proposed` (unless one is already pending), then `Forecast`:

```
$ export TSR_DB="$PWD/.tsr-verify/tsr.db"; rm -rf .tsr-verify
$ ID=$(./target/debug/tsr task open --name demo --workspace /w --intent "test the cli"); echo "$ID"
e7a3404c4b49c4ed80bb0dae5703fe45d450bb2a01292103d4934315bbce6212

$ ./target/debug/tsr forecast --kind run_approved_test --args "cargo test --offline -p parser" \
    --p 0.7 --source generator --task "$ID"
52bc63a8bea5a85ebc7281d5faf69dde8c6b6c3e9ae9cb49754755a2368bc910

$ ./target/debug/tsr task export "$ID" | wc -l
4
```

The `--event <proposed-event-id>` alternative records a forecast for an already-proposed candidate.

**Which task a forecast lands in.** `tsr forecast` and `tsr exec` resolve the session with the same
rule the hook uses, in this order: an explicit `--task <id>`; then `--session <id>` or `TSR_SESSION`;
then the `.tsr/session` file the hook writes beside the store on every stage (nearest ancestor). With
none of these it refuses with exit 2 and records nothing — there is no "latest task" and no
`hash("default")` fallback. The session file names the session of the **last hook stage**, so two
sessions sharing one working directory must each name themselves with `TSR_SESSION` (or `--session`).

Two sessions open, one forecast, one `PreToolUse` in session A — the forecast lands in A, and B holds
only its own `task_opened`:

```
$ export TSR_DB=$PWD/target/rep17/tsr.db
$ echo '{"session_id":"sess-A","hook_event_name":"SessionStart","cwd":"/work"}' | ./target/debug/tsr hook --stage SessionStart
$ echo '{"session_id":"sess-B","hook_event_name":"SessionStart","cwd":"/work"}' | ./target/debug/tsr hook --stage SessionStart
$ TSR_SESSION=sess-A ./target/debug/tsr forecast --kind run_approved_test --args "cargo test --offline -p parser" --p 0.7 --source generator
$ echo '{"session_id":"sess-A","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"tsr exec -- cargo test --offline -p parser"}}' | ./target/debug/tsr hook --stage PreToolUse
$ ./target/debug/tsr task export sess-A
0 task_opened
1 proposed
2 forecast  state_version=3
3 committed
$ ./target/debug/tsr task export sess-B
0 task_opened
```

A gated tool call with no prior forecast records `Proposed` + `Committed`; the forecast is then
appended for that same candidate:

```
$ ./target/debug/tsr forecast --event e07bad41b1e4c9dbfd308a93a53ebd1e41019f8651e715be100c8ea770ff91c3 \
    --p 0.7 --source generator --task event-session
34387cb3b3466fe25e7698319f894e9aa5428a4bbb23f9d00723e7d8aafe9ec7
```

### The honest hook flow

A forecast before the tool call, then the `PreToolUse` for the same command: the hook records
`Committed` only — no second `Proposed`, no second `Forecast` — and a host `Bash` run is dispatched
with `in_sandbox: false`:

```
$ export TSR_DB="$PWD/.tsr-happy/tsr.db"; rm -rf .tsr-happy
$ S=happy-session
$ echo '{"session_id":"'"$S"'","hook_event_name":"SessionStart","cwd":"/w"}' \
    | ./target/debug/tsr hook --stage SessionStart
$ ./target/debug/tsr forecast --kind run_approved_test --args "cargo test --offline -p parser" \
    --p 0.7 --source generator --task "$S"
$ echo '{"session_id":"'"$S"'","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo test --offline -p parser"}}' \
    | ./target/debug/tsr hook --stage PreToolUse
$ ./target/debug/tsr task export "$S" | python3 -c '...'   # print seq + kind
1 task_opened
2 proposed
3 forecast state_version=3
4 committed
5 dispatched in_sandbox=False
```

The `Committed` at seq 4 has a prior `Forecast` at `state_version = 3` — the index the `Committed`
event occupies (0-based) — so `every_gated_action_has_prior_forecast_at_same_state_version()` holds.

A raw `cargo test` `Bash` with **no** prior forecast records `Proposed` + `Committed` and no
forecast — the truth, and the frozen test rejects it:

```
$ export TSR_DB="$PWD/.tsr-host/tsr.db"; rm -rf .tsr-host
$ S=host-session
$ echo '{"session_id":"'"$S"'","hook_event_name":"SessionStart","cwd":"/w"}' \
    | ./target/debug/tsr hook --stage SessionStart
$ echo '{"session_id":"'"$S"'","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo test --offline -p parser"}}' \
    | ./target/debug/tsr hook --stage PreToolUse
$ ./target/debug/tsr task export "$S" | python3 -c '...'
1 task_opened
2 proposed
3 committed
4 dispatched in_sandbox=False
```

A `tsr exec -- …` `Bash` records `Committed` and **no** `Dispatched` from the hook — `tsr exec`
records the dispatch itself. The hook digests the **inner** command (`cargo test --offline -p
parser`), so its candidate is the one the forecast named, and it records `Committed` only:

```
$ export TSR_DB="$PWD/.tsr-repro3/tsr.db"; rm -rf .tsr-repro3
$ S=repro3
$ echo '{"session_id":"'"$S"'","hook_event_name":"SessionStart","cwd":"/w"}' \
    | ./target/debug/tsr hook --stage SessionStart
$ ./target/debug/tsr forecast --kind run_approved_test --args "cargo test --offline -p parser" \
    --p 0.7 --source generator --task "$S"
$ echo '{"session_id":"'"$S"'","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"tsr exec -- cargo test --offline -p parser"}}' \
    | ./target/debug/tsr hook --stage PreToolUse
$ ./target/debug/tsr task export "$S" | python3 -c '...'
1 task_opened
2 proposed candidate=e07bad41b1e4
3 forecast state_version=3
4 committed candidate=e07bad41b1e4
```

The `Committed` at seq 4 carries the forecast's candidate (`e07bad41…`) and there is no second
`Proposed` and no `Dispatched` from the hook — `tsr exec` records the dispatch when it runs.

### The hook records and never blocks

```
$ echo '{"session_id":"s","hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":"x"}}' \
    | ./target/debug/tsr hook --stage PreToolUse ; echo "exit=$?"
exit=0
```

Every hook path exits 0 and emits no `deny`/`block` decision (M0 records and never blocks).

### The only `in_sandbox: true` is in `exec.rs`

```
$ grep -rn "in_sandbox: true" crates/tsr-cli/src
crates/tsr-cli/src/hook.rs:18://! observed; only `tsr exec` records `in_sandbox: true`.
crates/tsr-cli/src/exec.rs:136:/// in that case **nothing is recorded**. Otherwise it records `Dispatched { in_sandbox: true }`
crates/tsr-cli/src/exec.rs:196:            in_sandbox: true,
crates/tsr-cli/src/lib.rs:22:    //! - every command ran in the sealed sandbox (`Dispatched { in_sandbox: true }`), never on the
```

The only non-comment occurrence is `exec.rs:196`.

### `tsr exec` refuses with exit 3 when no runtime is reachable

```
$ command -v podman docker ; echo "exit=$?"
exit=127

$ ./target/debug/tsr exec -- true ; echo "exit=$?"
tsr: no container runtime reachable (podman/docker); refusing to run on the host
exit=3
```

The refusal is decided in `Runtime::detect()` and acted on in `run()` **before** any event is
appended, so nothing is recorded. The container image is read from `TSR_EXEC_IMAGE` (default
`docker.io/library/debian:stable-slim`). After the container run, `tsr exec` copies the captured
stdout and stderr to the caller's stdout and stderr and exits with the child's exit code, so an
agent running `tsr exec -- cargo test …` sees the result.

### The self-test harness refuses rather than fabricate

```
$ ./target/debug/tsr hook --selftest ; echo "exit=$?"
tsr: hook selftest: no container runtime reachable (podman/docker); refusing to fabricate the fixture
exit=3
```

The harness drives realistic Claude Code payloads through the honest flow — a task opened, two
inspections, a forecast before each gated action, a test run through `tsr exec` in the real
container, a patch proposal, a second run, a stop — and writes the trace as `events.jsonl`. With no
container runtime reachable it prints why, exits 3 and writes nothing (the fixture's sha256 is
unchanged: `ec174a76…`).

## The fixture — regenerated by the operator on a host with Docker

`fixtures/m0/session.events.jsonl` is produced by the tool under test (`tsr hook --selftest`), never
written by hand. It cannot be regenerated in this sandbox: no `podman`/`docker` binary exists here
(`command -v podman docker` → exit 127), and the self-test refuses rather than fabricate. The
operator regenerates it on a host with Docker:

```bash
./target/debug/tsr hook --selftest
```

`crates/tsr-cli/tests/m0_reconstruct.rs` is the frozen exit test: it asserts on that fixture
byte-for-byte and **may be red until the operator regenerates it**. Its assertions are unchanged by
this repair. The reproducibility test in `tests/m0_harness.rs` skips with a printed reason when
`exec::Runtime::detect()` is `None`, so the suite is green in a sandbox with no runtime.

## The real record

The operator ran `tsr hook --selftest` on a host with Docker (the only place it can run) at
`e180a9f`. It produced a **real** record — every command ran in the sealed container — and that
record is now committed as `fixtures/m0/session.events.jsonl` (listed in
`specs/lanes/01c-real-session.md`). The frozen test `m0_session_reconstructs_from_the_log_alone`
**fails** on it at the B5 assertion: a gated action without a forecast at the same state version.
The record, by sequence (kind; candidate kind; detail):

```
 1 task_opened
 2 proposed inspect · 3 committed                         (Read)
 4 proposed inspect · 5 committed                         (Read)
 6 proposed run_approved_test · 7 forecast p=0.7 · 8 committed      (tsr forecast, then PreToolUse Bash "tsr exec -- cargo test --offline -p parser")
 9 dispatched in_sandbox=true · 10 executed exit=127 duration=12521  (exec::run, real container)
11 proposed run_approved_test · 12 committed              ← PostToolUse for the tsr exec call: NO forecast, nothing should be recorded here
13 proposed propose_patch · 14 forecast p=0.6 · 15 committed        (tsr forecast, then PreToolUse Edit)
16 proposed run_approved_test · 17 forecast p=0.6 · 18 committed    (tsr forecast, then PreToolUse Bash, second run)
19 proposed run_approved_test · 20 committed              ← exec::run minted a NEW candidate: it did not find the commitment at 18
21 dispatched in_sandbox=true · 22 executed exit=127 duration=13258
23 proposed run_approved_test · 24 committed              ← PostToolUse again
25 stop done
```

Two defects, both invisible in the sandbox because `exec::run` refuses there and the frozen test was
only ever run against the old fabricated fixture:

1. **`record_post_tool` proposed and committed.** For a Bash call whose command is a `tsr exec -- …`
   invocation, PostToolUse must record nothing (`tsr exec` recorded the dispatch and the execution).
   For a host-run Bash it records `Executed`/`Unresolved` against the candidate the PreToolUse
   committed — it never proposes or commits. Events 11–12 and 23–24 were the hook inventing gated
   actions after the fact, the exact fault 01b was meant to remove. Repaired in
   `crates/tsr-cli/src/hook.rs::record_post_tool` (commit `46440b9`): the function now returns
   without appending for a `tsr exec -- …` call, reconciles the most recent unreconciled `Committed`
   for any other Bash call, and records `Unresolved { reason: "post-tool with no prior commitment" }`
   when there is none — no path appends `Proposed` or `Committed`.
2. **`exec::run` did not find the second commitment.** The candidate id is content-addressed
   `(kind, args, parents)`, so the second run of the same command has the same id as the first. The
   old lookup treated "this id has a `Dispatched`" as "dispatched" and minted a new candidate
   (19–20) with no forecast. The lookup must be per occurrence, in event order: the most recent
   `Committed` for a matching digest that has no `Dispatched` after it. Repaired in
   `crates/tsr-cli/src/exec.rs::pending_commitment` (commit `eb1e81c`), which walks the task's events
   in sequence order and returns the candidate of the most recent `Committed` with a matching
   `(kind, args_digest)` that has no `Dispatched` after it; `exec::run` mints a new candidate only
   when it returns `None`.

A third finding, a design decision rather than a defect: two consecutive selftests on the host differ
only in `Executed.duration_ms` (12521 vs 13258; `log_digest` and `exit_code` are identical). A
byte-for-byte reproducibility test therefore cannot pass on a real host, and it must not pass by
fabricating the duration. Reproducibility means the sequence of
`(seq, kind, candidate id, forecast state_version, in_sandbox, exit_code, log_digest)` is identical
across two selftests; `duration_ms` is a measured quantity and is excluded by name.

**The frozen fixture is now a real record.** `fixtures/m0/session.events.jsonl` is no longer the
fabricated trace of 01b: it is the operator's real `tsr hook --selftest` output, and it is the input
the frozen test `crates/tsr-cli/tests/m0_reconstruct.rs` asserts on. It cannot be regenerated in this
sandbox (no `podman`/`docker` binary exists here; the self-test refuses rather than fabricate), so the
frozen test stays red here. **The operator regenerates the fixture on a host with Docker after this
repair**:

```bash
./target/debug/tsr hook --selftest
```

With the two defects repaired, the regenerated record has 19 events (no `Proposed`/`Committed` after
a `tsr exec` PostToolUse, and the second run dispatching the candidate committed at its own
PreToolUse), and the frozen test goes green on the host. The fixture and the frozen test are
byte-for-byte unchanged by this repair.

## The real Claude Code session (operator, 2026-10-11)

PRD §B11's M0 exit evidence is "one real session on a disposable copy of rts is fully reconstructed
from the log; every command ran in the sandbox". Everything above it was the hook's own harness. This
section is a real Claude Code session.

**Setup.** `~/Developer/_orchestration/m0real-*.sh` on the operator's Mac.
- **tsr:** built natively (release) from this branch.
- **rts copy:** a fresh clone of rts at `d95314f`. Its `.claude/settings.json` holds only the four
  `tsr hook` stages (SessionStart, PreToolUse, PostToolUse, Stop); the repository's own hooks were
  replaced in the copy. A short TSR protocol section was prepended to its `CLAUDE.md`: forecast before
  each Edit/Write and before the tests, and run tests only as `tsr exec -- …`.
- **Store:** outside the copy (`TSR_DB=<base>/.tsr/tsr.db`, with `<copy>/.tsr` a symlink to it). The
  sandbox mounts only the copy, so inside it the link dangles and the log is out of reach.
- **Dependencies:** rts-core's test dependencies were prebuilt once, in the `tsr-rust:1.90` image at
  the same bind path, so the in-session run works with `--offline` and `--network none`.
- **Claude Code:** 2.1.225, headless (`claude -p`). Tools allowed: Read, Grep, Glob, Edit, Write,
  `Bash(<base>/bin/tsr forecast:*)` and `Bash(<base>/bin/tsr exec:*)`.
- **Model:** `claude-sonnet-5.5` through an Anthropic-compatible endpoint (cheaperinference), because
  the Mac's own Claude Code login had expired.
- **Task:** add a unit test for the edge cases of `detect_language_from_path` in rts-core, then run
  the crate's library tests through the sandbox.

**First attempt: the record failed the frozen test, honestly.** The session itself did everything
right: inspect, forecast the edit, edit, forecast the test, run it through `tsr exec` (269 passed).
The log still failed `every_gated_action_has_prior_forecast_at_same_state_version`:

```
13 proposed propose_patch · 14 forecast (state_version 15) · 15 unresolved "no exit code in the hook response"
16 committed propose_patch                                   ← committed at 16, forecast for 15
```

`tsr forecast` is itself a Bash tool call. The hook recorded that call as a `Generate` turn, and its
PostToolUse appended an `Unresolved` *between* the forecast and the commitment it names. The selftest
never sent the hooks around its own forecast calls, which is why it passed. **Fix:** the hook records
nothing before or after a `tsr forecast …` call, the same way it already records nothing after a
`tsr exec -- …` call. The selftest now sends those two payloads exactly as Claude Code does (its
frozen fixture is unchanged, because the hook now ignores them). Two unit tests cover it:
`forecast_call_appends_nothing_before_or_after` and `forecast_call_hooks_keep_the_state_version`.

**Second attempt, with the fixed tsr: the record.** Exported with
`tsr task export 076c2b25-e42f-4b19-bd3c-a500e2a9ea3d` to `fixtures/m0/claude-session.events.jsonl`.
The patch it made is `fixtures/m0/claude-session.rts.diff`. Claude Code reported 9 turns.

```
 0 task_opened
 1-8 proposed inspect / committed          (Grep, Glob, Read lib.rs, Read AGENTS.md)
 9 proposed propose_patch · 10 forecast (state_version 11, p 0.93, generator) · 11 committed
12 proposed run_approved_test · 13 forecast (state_version 14, p 0.9, generator) · 14 committed
15 dispatched in_sandbox=true · 16 executed exit_code=0 duration_ms=51001     (tsr exec, real container)
17 stop reason=done
```

```
$ cargo test -p tsr-cli --test m0_real_session --test m0_reconstruct      # tsr-rust:1.90
test the_real_session_exercises_the_whole_path ... ok
test the_real_session_reconstructs_and_keeps_every_rule ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test m0_session_reconstructs_from_the_log_alone ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`tests/m0_real_session.rs` runs the frozen test's six invariants over the real record. It also checks
that the record covers the whole path: a forecast patch, a forecast test run dispatched into the
sandbox and executed there, no host dispatch, and a stop. That second test exists because the frozen
test alone passed on a one-event log: an attempt whose model call failed authentication recorded
only `task_opened`.

**What this record does not show.**
- The edit was applied by Claude Code's Edit tool directly to the disposable copy. M0 gates it and
  records it (`proposed` → `forecast` → `committed`), but does not apply it inside the sandbox; PRD
  path A's "Edit and Write … applied inside the sandbox" is later work.
- Claude Code's PostToolUse for a Bash call carries no exit code (stdout, stderr, interrupted), so a
  Bash call outside `tsr exec` can only ever be recorded `Unresolved`. The selftest's Bash payloads
  still carry an `exit_code` field that real Claude Code does not send.
- One session is the milestone's bar. It is not a sample.

## Notes

- `crates/tsr-domain` was not touched by this repair; the additive changes M0 needed
  (`EventKind::Stop`, `Forecast::new`, `Event::state_version`) landed in commit `608cf64`.
- The fixture is produced by the tool under test, never written by hand.
