# Lane 01c — M0 repair, second pass: the real session fails the frozen test

**Workspace:** `tsr-engine` on branch `lane/core` (01b landed at `e180a9f`; this brief and the real fixture are the commits after it).
**Owns:** `crates/tsr-cli` (hook.rs, exec.rs, lib.rs, tests/m0_harness.rs, tests/m0_forecast_default.rs), `docs/m0-evidence.md`, `README.md`.
**Frozen:** `fixtures/m0/session.events.jsonl` (now the REAL record, see below) and `crates/tsr-cli/tests/m0_reconstruct.rs`. Do not edit either; the verifiers compare them byte for byte with the run-start baseline.
**Reads first:** `AGENTS.md`; this file; `specs/lanes/01b-honest-record.md`; `crates/tsr-cli/src/hook.rs` (`record_tool`, `record_post_tool`, `selftest_inner`), `exec.rs` (`run`, the committed-candidate lookup), `lib.rs` (`reconstruct`), `tests/m0_harness.rs`.

## Why

The operator ran `tsr hook --selftest` on a host with Docker (the only place it can run) at `e180a9f`. It produced a real record — every command ran in the sealed container — and the frozen test `m0_session_reconstructs_from_the_log_alone` **fails** on it at the B5 assertion: a gated action without a forecast at the same state version. The record, by sequence (kind; candidate kind; detail):

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

Two defects, both invisible in the sandbox because `exec::run` refuses there and the frozen test was only ever run against the old fabricated fixture:

1. **`record_post_tool` proposes and commits.** For a Bash call whose command is a `tsr exec -- …` invocation, PostToolUse must record nothing (`tsr exec` recorded the dispatch and the execution). For a host-run Bash it records `Executed`/`Unresolved` against the candidate the PreToolUse committed — it never proposes or commits. Events 11–12 and 23–24 are the hook inventing gated actions after the fact, which is the exact fault 01b was meant to remove.
2. **`exec::run` does not find the second commitment.** The candidate id is content-addressed (kind, args, parents), so the second run of the same command has the same id as the first. The lookup treats "this id has a `Dispatched`" as "dispatched" and mints a new candidate (19–20) with no forecast. The lookup must be per occurrence, in event order: the most recent `Committed` for a matching digest that has no `Dispatched` after it. 01b's own commit `a623a32` ("a re-proposed candidate is pending again") already treats `Proposed` this way; `Dispatched` needs the same rule.

A third finding, a design decision rather than a defect: two consecutive selftests on the host differ only in `Executed.duration_ms` (12521 vs 13258; `log_digest` and `exit_code` are identical). A byte-for-byte reproducibility test therefore cannot pass on a real host, and it must not pass by fabricating the duration. Reproducibility means: the sequence of `(seq, kind, candidate id, forecast state_version, in_sandbox, exit_code, log_digest)` is identical across two selftests; `duration_ms` is a measured quantity and is excluded by name.

## Goal (the text handed to `ds-swarm run`)

Repair milestone M0 of the TSR Decision Engine a second time: the operator's real `tsr hook --selftest` record (now committed as fixtures/m0/session.events.jsonl, and listed in specs/lanes/01c-real-session.md) fails the frozen test tests/m0_reconstruct.rs because the hook and tsr exec still invent gated actions. Read AGENTS.md, then specs/lanes/01c-real-session.md (the record and the two defects), then crates/tsr-cli/src/hook.rs, exec.rs, lib.rs and tests/m0_harness.rs. Deliver, in this order:
(1) record_post_tool: for a Bash call whose command is `tsr exec -- …` (first tokens `tsr exec`, optionally with a path prefix on `tsr`, exactly as record_tool already detects it) it appends nothing; for any other Bash call it appends Executed or Unresolved for the candidate the matching PreToolUse committed (the most recent Committed with that candidate id that has no Executed/Unresolved after it), and if there is no such commitment it appends Unresolved { reason: "post-tool with no prior commitment" } and never Proposed or Committed. PostToolUse for Read/Edit/Write records nothing new. Delete every path by which record_post_tool can append Proposed or Committed.
(2) exec::run: factor the lookup into `pub fn pending_commitment(store, task, kind, args_digest) -> Option<CandidateId>` that walks the task's events in sequence order and returns the candidate of the most recent Committed with a matching (kind, digest) that has no Dispatched after it; use it in exec::run; mint a new candidate only when it returns None. Unit-test it over a store holding the exact sequence above (two runs of the same command): after events 1–18 it returns the candidate committed at 18.
(3) make exec::run drivable without a container: factor the container invocation behind a small `Runner` trait (the real one builds the docker/podman argv exactly as today; a `FakeRunner` under cfg(test) or in the harness returns a scripted exit code and log bytes). Refusal semantics unchanged: with no real runtime and no fake runner injected, exit 3 and nothing recorded. The seal flags (network none, read-only root, writable workspace only, TSR_EXEC_IMAGE validation) stay in the real runner and keep their tests.
(4) harness tests (tests/m0_harness.rs) that replay the selftest's exact sequence through record_session_start, record_forecast_for_action, record_tool, exec::run with the FakeRunner, record_post_tool and record_stop, and assert that the resulting events satisfy every invariant of tsr_cli::reconstruct::Session — in particular every_gated_action_has_prior_forecast_at_same_state_version — and that the sequence has exactly 19 events: no Proposed/Committed after a tsr exec PostToolUse, and the second run dispatches the candidate committed at its PreToolUse. Add the case where a host Bash (`cargo test …` without tsr exec) yields Dispatched { in_sandbox: false } at PreToolUse and Executed at PostToolUse with no new Proposed.
(5) reproducibility: the existing byte-for-byte test becomes a structural comparison — two selftests (through the FakeRunner in the harness; through the real runner on a host) are reproducible when the sequence of (seq, kind, candidate id, forecast state_version, in_sandbox, exit_code, log_digest) is identical; duration_ms is excluded by name and the test says why in its name. The selftest itself still refuses with exit 3 and writes nothing when no runtime is reachable; it never fabricates a duration.
(6) docs/m0-evidence.md: add the section "The real record" quoting the operator's listing above and the two defects, and state that the frozen fixture is now a real record that the operator regenerates after this repair; README: the agent protocol unchanged, plus one line on what PostToolUse records.
No tsr-domain or tsr-store change; conventional commits; fmt and clippy clean. You cannot run `tsr hook --selftest` in your sandbox (no runtime): prove the repair with the harness, and leave fixtures/m0/session.events.jsonl and tests/m0_reconstruct.rs exactly as they are — the operator regenerates the fixture on the host and the frozen test goes green there.
Operator notes: a create_task refused three times with the same text is a signal, not a retry — stop, post the refusal to the operator, and wait. A passing command is recorded once; do not run a green test again to confirm it. await_task waits at least 60 seconds. Start every command with `export PATH=/usr/local/cargo/bin:$PATH CARGO_HOME=$PWD/.cargo-home`; .cargo-home/ and target/ are build caches — never commit them, exclude them from grep and surveys. Every file write goes through the sandbox, never a host path. A landing commit holds deliverable files only. A review verdict names the file, the line and the behaviour it saw. Review tasks are created after a landing exists; if create_task refuses with clause (jj), move on and delegate the ready tasks. No reply needed.

## Definition of done (`--dod`)

cargo build, cargo fmt --all -- --check and cargo clippy --workspace --all-targets -- -D warnings pass. cargo test --workspace --no-fail-fast: every test passes except tests/m0_reconstruct.rs, which stays red in the sandbox until the operator regenerates the fixture on a host. The harness replays the selftest sequence through the FakeRunner and the resulting events satisfy every tsr_cli::reconstruct invariant, contain exactly 19 events, contain no Proposed or Committed after a tsr exec PostToolUse, and show the second run dispatching the candidate committed at its own PreToolUse. pending_commitment over the committed listing's events 1–18 returns the candidate committed at 18. A host Bash yields Dispatched { in_sandbox: false } and then Executed, with no new Proposed. The reproducibility test compares (seq, kind, candidate id, forecast state_version, in_sandbox, exit_code, log_digest) and excludes duration_ms by name. `grep -rn "in_sandbox: true" crates/tsr-cli/src` matches only exec.rs. record_post_tool contains no call that appends Proposed or Committed. `tsr hook --selftest` without a runtime exits 3 and writes nothing. fixtures/m0/session.events.jsonl and tests/m0_reconstruct.rs are byte-for-byte unchanged by this run. docs/m0-evidence.md and README updated.

## Out of scope

Policy gating; the real Claude Code session on rts (the operator records it after this lands); any tsr-store or tsr-domain change.

## Acceptance commands (operator, on a host with Docker)

```bash
cargo build --workspace && cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast          # m0_reconstruct red until the fixture is regenerated
./target/debug/tsr hook --selftest              # regenerates fixtures/m0/session.events.jsonl for real
cargo test --workspace --no-fail-fast          # all green, assertions unchanged; 19 events in the fixture
./target/debug/tsr hook --selftest && cargo test -p tsr-cli --test m0_harness   # structural reproducibility on the host
```

## Suggested run flags

```
--max-tokens 400000000 --deadline-secs 14400 --max-agents 4 --reviews security,architect,correctness
```
