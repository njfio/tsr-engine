# M0 evidence — the recorder, the executor, and reconstruction from the log alone

Milestone M0 (PRD §B11). Every command below was run from the workspace root with the sandbox's
cargo environment:

```bash
export CARGO_HOME="$PWD/.cargo-home"
export PATH="/usr/local/cargo/bin:$PATH"
```

## What landed

| Deliverable | Where |
| --- | --- |
| `tsr hook --stage <PreToolUse\|PostToolUse\|SessionStart\|Stop>` | `crates/tsr-cli/src/hook.rs`, wired in `crates/tsr-cli/src/main.rs` |
| `tsr exec -- <argv>` (sealed sibling container) | `crates/tsr-cli/src/exec.rs` |
| `tsr task open \| status \| export <id>` | `crates/tsr-cli/src/task.rs` |
| `tsr forecast --event <id> --p <0..1> --source <…>` | `crates/tsr-cli/src/hook.rs::record_forecast` |
| `tsr_cli::reconstruct::from_events` | `crates/tsr-cli/src/lib.rs` |
| Hook self-test harness + fixture | `crates/tsr-cli/src/hook.rs::selftest`, `fixtures/m0/session.events.jsonl` |

## The commands and their captured output

### Build, format, lint

```
$ cargo build --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.03s

$ cargo fmt --check
fmt OK

$ cargo clippy --workspace --all-targets -- -D warnings
    Checking tsr-cli v0.0.1 (/work/crates/tsr-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.34s
```

### The whole test suite, including the M0 exit test

```
$ cargo test --workspace --no-fail-fast
running 1 test
test m0_session_reconstructs_from_the_log_alone ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`crates/tsr-cli/tests/m0_reconstruct.rs` is green **with its assertions unchanged**.

### No container runtime is present in this sandbox

```
$ command -v podman docker ; echo "exit=$?"
exit=127
```

Neither `podman` nor `docker` exists here, so the exec path exercised in this workspace is the
refusal path (exit 3, nothing recorded). The container invocation itself is covered by
`exec.rs::container_argv` and is not runnable in this sandbox.

### `tsr exec` refuses with exit 3 when no runtime is reachable

```
$ ./target/debug/tsr exec -- true ; echo "exit=$?"
tsr: no container runtime reachable (podman/docker); refusing to run on the host
exit=3
```

The decision is made in `crates/tsr-cli/src/exec.rs`:

- `Runtime::detect()` (exec.rs:40) returns `None` when neither `podman` nor `docker` is on `PATH`
  and answers `--version`;
- `run()` (exec.rs:120) returns `Err(ExecRefusal::NoRuntime)` (exec.rs:128) **before** any event is
  appended, so nothing is recorded;
- `main.rs::run_exec` (main.rs:288) maps that refusal to `ExitCode::from(EXIT_NO_RUNTIME)` = 3
  (main.rs:300-304, `EXIT_NO_RUNTIME` defined at exec.rs:20).

The agent's argv is never handed to the host: the only `Command` built is the container runtime
invocation (`container_argv`, exec.rs:76), and the argv is passed as arguments to it, never
interpolated into a shell string.

### The hook records and never blocks

```
$ echo '{"session_id":"s","hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":"x"}}' \
    | ./target/debug/tsr hook --stage PreToolUse ; echo "exit=$?"
exit=0
```

Every hook path exits 0 and emits no `deny`/`block` decision (M0 records and never blocks).

### The self-test harness writes the fixture

```
$ ./target/debug/tsr hook --selftest
wrote fixtures/m0/session.events.jsonl
```

The harness drives realistic Claude Code payloads through the hook — a task opened, two
inspections, a forecast, a sandboxed test run, a patch proposal, a forecast, a second run, a stop —
and writes the trace as `events.jsonl`. The recorded sequence:

```
seq  kind
1    task_opened
2    proposed          (inspect)
3    committed         (inspect)
4    proposed          (inspect)
5    committed         (inspect)
6    forecast          sv=6
7    proposed          (run_approved_test)
8    forecast          sv=8
9    committed         (run_approved_test)
10   dispatched        in_sandbox=true
11   executed
12   proposed          (propose_patch)
13   forecast          sv=13
14   committed         (propose_patch)
15   forecast          sv=15
16   proposed          (run_approved_test)
17   forecast          sv=17
18   committed         (run_approved_test)
19   dispatched        in_sandbox=true
20   executed
21   stop
```

Each gated commitment (seq 9, 14, 18) has a forecast whose `state_version` equals the commitment's
0-based index in the file (8, 13, 17) — the B5 invariant, checked by the exit test.

### `tsr task` and `tsr forecast`

```
$ TSR_DB="$PWD/.tsr-verify/tsr.db" ./target/debug/tsr task open --name demo --workspace /w --intent "test the cli"
e7a3404c4b49c4ed80bb0dae5703fe45d450bb2a01292103d4934315bbce6212

$ TSR_DB="$PWD/.tsr-verify/tsr.db" ./target/debug/tsr task status demo
task e7a3404c4b49c4ed80bb0dae5703fe45d450bb2a01292103d4934315bbce6212: 2 events

$ TSR_DB="$PWD/.tsr-verify/tsr.db" ./target/debug/tsr forecast --event task_accepted_in_budget@v1 --p 0.7 --source generator --task demo
fd2bd9af443009aa58a5ea15ecc1aae77309c3d642a23f6c1b0d382e9a645375

$ TSR_DB="$PWD/.tsr-verify/tsr.db" ./target/debug/tsr task export demo | head -1
{"event":{"id":"fb45f4eb…","kind":{"kind":"task_opened",…},"seq":1}
```

## Notes

- `crates/tsr-domain` was not touched by this lane; the additive changes it needed
  (`EventKind::Stop`, `Forecast::new`, `Event::state_version`) landed in commit `608cf64`.
- `crates/tsr-store`'s `latest_seq` / `append_all` helpers landed in the store lane.
- The fixture is produced by the tool under test (`tsr hook --selftest`), never written by hand.

## The commits

```
$ git log --oneline -10
c5e3361 docs: M0 evidence page and README hook registration
ad9d810 cli: tsr hook/exec/task/forecast + reconstruct::from_events (M0)
608cf64 contract: EventKind::Stop, Forecast::new, Event::state_version (additive, M0 hook needs)
a525758 docs: keep the cargo cache inside the workspace when sandboxed
6bbfeca chore: M0 skeleton — contract crate, append-only store, red M0 test, lane briefs
```