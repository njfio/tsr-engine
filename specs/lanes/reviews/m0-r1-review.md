**M0 r1: not done** — everything works except that a relative or symlinked `TSR_DB` inside the workspace is neither mounted read-only nor refused, so the sandbox can still write the event store.

## (1) Probe asks the daemon: DONE
exec.rs:104-135 runs `<rt> info` with a 5 s kill-timeout. The real `/usr/bin/docker` (CLI, no daemon), a fake with `--version` ok and `info` failing, and a fake whose `info` sleeps 30 s all give exit 3 and 0 events (the hang returns in 5.06 s). No binary: exit 3, 0 events.

## (2) Container does not start: DONE
exec.rs:298 maps `run` exit 125/126/127 to `StartFailed`; exec.rs:503 records `Unresolved`. Each gives `proposed, committed, dispatched, unresolved`, a reason naming the runtime message, no `executed`, exit 3. A spawn error (binary deleted after `info`) behaves the same. Cosmetic: the message repeats "container runtime failed to start:" three times; the main.rs:505 comment ("unreachable from the CLI") is now false.

## (3) Command's own failure: DONE
`run` exit 1 → `executed{exit_code:1}`, exit 1.

## (4) Selftest: DONE
- Unreachable daemon (fake and real CLI): exit 3. Workspace and TMPDIR listings and hashes are identical before and after. `--out` creates nothing; `--update-frozen-fixture` changes nothing.
- Working fake, no flags: writes only to a fresh temp dir it prints. The scratch `fixtures/m0` is byte-identical afterwards. `--out <dir>` writes there.
- `--update-frozen-fixture` rewrites `fixtures/m0` (and overrides `--out` silently).
- `run`=125: exit 3, `--out` not created.
- Low: that refusal leaves `$TMPDIR/tsr-m0-selftest-db-<pid>` behind (hook.rs:823); a successful run creates `<cwd>/.tsr` (gitignored).

## (5) `container_argv`: NOT DONE
Works for absolute paths. The default `.tsr` and `TSR_DB=<ws>/data/tsr.db` get `--mount type=bind,src=<dir>,dst=<dir>,readonly` after the workspace mount (argv lines 22-23 against 8-9). `TSR_DB` outside the workspace gets none. `TSR_DB=<ws>/tsr.db` exits 2. The directory is created first (exec.rs:285).

**Defect (medium).** `store_dir_inside_workspace` (exec.rs:202-219) and `store_dir_is_workspace` (exec.rs:395) compare the raw `TSR_DB` with the absolute workspace via `starts_with`, with no resolution or canonicalisation. With the fake docker recording its argv:

| `TSR_DB` | Result |
|---|---|
| `data/tsr.db` | no readonly mount, exit 0 |
| `.tsr/tsr.db`, `./.tsr/tsr.db` | no readonly mount, exit 0 |
| `tsr.db` | no readonly mount, not refused, exit 0 |
| outside path that is a symlink into `<ws>` | no readonly mount |

In each case the only mount is the read-write workspace, so a container command can rewrite the log and session file. The new tests cover only absolute paths. Fix: resolve against the workspace and canonicalise first.

Also writable: `current_session` (store_path.rs:121-131) walks up to `<ws>/session`, outside `.tsr`; with `.tsr/session` absent a planted one was used. Moot once the hook has written `.tsr/session`; out of scope (low, pre-existing). `.tsr/session` and the sqlite sidecars are covered.

## (6) Regressions: DONE
- Real-session 2/2 and `m0_reconstruct` 1/1 pass. `fixtures/m0` is byte-identical to 8f08b03 (sha256 and mtimes).
- Diff: exec.rs, hook.rs, main.rs, m0_harness.rs, new m0_honest_sandbox.rs, two specs/ files. Only main.rs is off the Owns list: the two selftest flags plus a `StoreIsWorkspace` → exit 2 arm in `run_exec`, so not only selftest flags.
- fmt clean. `cargo +1.90.0 clippy --workspace --all-targets -- -D warnings` clean.
- `cargo test --workspace --no-fail-fast`: 270 passed, 0 failed, 0 ignored. The two daemon-dependent harness tests pass by printing a skip.

## Other, low
- exec.rs:298: a command whose own status is 126/127 (`sh -c nosuchcmd`) is Unresolved, exit 3, as the brief specified.
- If the daemon dies between probe and `run`, docker exits 1 ("failed to connect") and the record is `Executed{1}` (time-of-check/time-of-use).

## What I ran
Fake `docker` driven by env vars, scratch workspaces, events read via `tsr task export | jq`, find+sha256 snapshots, then fmt, clippy 1.90 and the full suite. `git status` is clean.
