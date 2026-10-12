**tsr-engine README: mostly accurate** — the M0 forecast/hook/exec flow works as written, but M1 is missing and nine claims are false or only conditionally true.

Checked at d2b5f96. I read crates/*/src and ran `tsr` in scratch dirs, piping hook payloads and putting a fake `docker` on PATH.

## Wrong

1. **L14–17 Status:** says "M0 landed". M1 is merged too (`tsr-policy`, `tsr policy check`). No command "reconstructs a session"; `reconstruct` is called only by tests.
2. **L21 "all green":** 2 tests fail on a host with the docker CLI but no daemon (this one). All 129 pass once `docker --version` fails.
3. **L60–61, L121 "no runtime → exit 3, nothing recorded/written":** a runtime counts as reachable when `docker --version` succeeds (exec.rs:78). With no daemon:
   - `tsr exec` recorded `dispatched{in_sandbox:true}` + `executed{exit_code:1}` and exited 1.
   - `hook --selftest` exited 0 and wrote a fixture whose runs never happened. That fixture passes the frozen invariants.
   - Run from the repo root (L118), the selftest overwrites the committed fixture.
4. **L47–48 "nearest ancestor… subdirectory still finds it":** the lookup walks up from the store's directory, not from cwd. With the default `TSR_DB`, `tsr forecast` from a subdirectory exits 2 and creates a second store.
5. **L45–47, L53:** `tsr exec` rejects `--task` and `--session` (exit 2).
6. **L43 `--event <proposed-event-id>`:** takes a candidate id, not an event id, and never checks it. A random id was recorded with exit 0.
7. **L59–60 "copies stdout and stderr":** both go to stdout, with stderr appended after the container exits.
8. **L58–59 "any other Bash test run":** only commands starting with `cargo test` count. `RUST_BACKTRACE=1 cargo test` was recorded as `generate` with no dispatch.
9. **L30 "Three rules follow":** following them does not give a passing record in an ordinary session.
   - Any other tool (Bash `ls`, TodoWrite) records `generate`+`committed` with no `proposed`. Reconstruct rejects that (lib.rs:94–96).
   - `tsr exec -- sh -c 'a; b'` fails because the hook and exec hash the arguments differently, so exec creates a new candidate with no forecast.
   - The real session passed only because its tools were restricted.

## Missing

- **M1 policy check:** `tsr policy check <profile> <event>` exits 0 allow / 1 deny / 2 ask / 3 malformed. Both fixtures returned deny. The hook does not call it.
- **Layout:** omits `crates/tsr-policy`, `profiles/`, `docs/ratchet-profile.md` and `docs/m1-evidence.md`.
- **Stub commands:** seven placeholders (review, reflect, corpus, data, train, eval, model) exit 2 with "not implemented".
- **Task commands:** `tsr task open|status|export` usage is not shown. `task status` miscounts when the store holds several tasks (a 2-event task showed 4).
- **Sandbox:** `TSR_EXEC_IMAGE` and Podman preference are not documented. The workspace is mounted read-write, so the default `.tsr/` store is writable from inside the sandbox.
- **Bash exit codes:** Claude Code sends none for Bash, so a Bash PostToolUse is always recorded `Unresolved`.
- **Real-session record:** `fixtures/m0/claude-session.events.jsonl` and `tests/m0_real_session.rs`.
- **LICENSE:** no license files.
- **AGENTS.md:**
  - build and `fmt --check` pass.
  - clippy `-D warnings` fails on 1.97 (`filter_next`, tsr-policy/src/gate.rs:411). No toolchain is pinned.
  - The "red test / from_events is a stub" text is stale.
  - `crates/tsr-ledger` does not exist.

## Right

Purpose; PRD v0.3.1 sections; links; `cargo build`; `TSR_DB` default and `TSR_SESSION`; forecast flags, state version, exit 2 with no session; the hook always exits 0 and records nothing around `tsr forecast` or after `tsr exec`; an unforecast gated call fails the frozen test; exec passes the child's exit code and exits 3 with no runtime binary; container flags; hook JSON (as the real session used it; not checked against docs); selftest path; license string.
