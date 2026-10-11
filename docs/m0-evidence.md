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

## Notes

- `crates/tsr-domain` was not touched by this repair; the additive changes M0 needed
  (`EventKind::Stop`, `Forecast::new`, `Event::state_version`) landed in commit `608cf64`.
- The fixture is produced by the tool under test, never written by hand.
