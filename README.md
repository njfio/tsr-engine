# tsr-engine

On main, `tsr` records a Claude Code session into an append-only event log (`tsr hook`, with
commands run in a sealed container by `tsr exec`), records the forecast the agent makes with
`tsr forecast` before a gated action (nothing yet makes it forecast), lets you label the returns
afterwards and renders one closed episode (`tsr review`, `tsr reflect`). It does not yet block actions, calibrate forecasts, choose actions or
release models. The design it is built towards ([`docs/PRD.md`](docs/PRD.md) §B1, not current
behaviour): a Rust controller that makes a coding agent commit with a frozen expectation, receive the
return and let the difference change what it does next, with Rust holding authority and a
generative model only proposing.

## What works today

- Record a Claude Code session through its hooks into a content-addressed log, with the forecasts
  the agent records before gated actions (`tsr hook`, `tsr forecast`, `tsr task export`).
- Run a command in a separate, network-less container and record the run (`tsr exec`).
- Check one proposed action against a protection profile (RATCHET's format, PRD §B4–B5) (`tsr policy check`).
- Label returns after the fact; render an episode or a family's reliability bins (how often forecasts
  of a given probability came true; nothing is calibrated) (`tsr review`,
  `tsr reflect`, `tsr note`, `tsr task reconcile`).
- Build a dataset from the log, fit a calibration head with an isotonic calibrator, verify a bundle
  and score the controller against the generator on held-out rows (`tsr train dataset`, `fit`,
  `verify`, `report`, `apply`). So far this has run on a synthetic corpus only; the evidence and its
  known limitations are in [`docs/m4-evidence.md`](docs/m4-evidence.md).

Read [Known limitations](#known-limitations-on-main) before relying on any of it.

**Prerequisites:** Linux, or macOS through a container runtime (`docs/PRD.md` §B3: Linux first);
Rust 1.90 through rustup (`rustup toolchain install 1.90.0`, with clippy and rustfmt for the lints);
a working `podman` or `docker` (a running daemon for docker) for `tsr exec`; `python3` for the two
M2 store builders in `scripts/`; Claude Code to drive the hook.

**Terms.** A *candidate* is one proposed action; a *gated action* (run a test, propose a patch,
request approval) needs a *forecast*, a probability recorded before it at the current *state
version* (the task's event count); the *return* is what came back; *Surprise* is forecast minus
outcome and *Contrast* is desired minus actual against the task's criteria, kept apart; the *Mirror*
(`tsr reflect`) is one closed episode on one screen; a *family* is a group of similar tasks.
Definitions: PRD §B2, §B5–§B7.

More: [`AGENTS.md`](AGENTS.md) (rules for contributors and agents), [`specs/lanes/`](specs/lanes/)
(lane briefs), evidence in [`docs/m0-evidence.md`](docs/m0-evidence.md),
[`docs/m1-evidence.md`](docs/m1-evidence.md) and [`docs/m2-evidence.md`](docs/m2-evidence.md).

## Status

| Milestone (PRD §B11) | State | What main has |
| --- | --- | --- |
| M0 Record | on main; known limitations below | `tsr hook`, `tsr exec`, `tsr forecast`, `tsr task open/status/export`, `tsr_cli::reconstruct` |
| M1 Gate | on main | `crates/tsr-policy`, `tsr policy check`; the hook does not call the gate |
| M2 Reflect | on main | `tsr review`, `tsr reflect`, `tsr note`, `tsr task reconcile`; `crates/tsr-ledger`, `crates/tsr-forecast`; 30 synthetic episodes |
| M3 Corpus | not on main (in development) | `tsr corpus` is a placeholder |
| M4 Calibrate | on main; synthetic corpus only | `tsr train dataset/fit/verify/report/apply`, `crates/tsr-learn`, `fixtures/m4`, `scripts/m4-evidence.sh`; `tsr data` and `tsr eval` are placeholders |
| M5 Decide | not started | `tsr model` is a placeholder |
| M6 Extend | not started | — |

A placeholder prints one line naming its PRD section ("tsr: \`corpus\` is not implemented yet — see
docs/PRD.md B9 (M3). …") and exits 2.

## Build and test

Every block below runs from the repository root, pasted in order into one shell.

```bash
cargo +1.90.0 build --workspace
export PATH="$PWD/target/debug:$PATH"      # the examples below call `tsr`
cargo +1.90.0 test --workspace --no-fail-fast
```

The test run is 261 passed and 2 failed wherever `docker` or `podman` answers `--version`, with or
without a daemon: `m0_forecast_default::exec_resolves_the_session_task_and_refuses_without_one`
expects `tsr exec` to refuse with exit 3, and
`m0_harness::selftest_is_reproducible_structurally_excluding_duration_ms_by_name` deletes its own
output before reading it. Both pass where neither answers. `cargo +1.90.0 clippy --workspace
--all-targets -- -D warnings` is clean on 1.90 only; newer stable clippy (1.97 tested) fails on
`filter_next` in `crates/tsr-policy/src/gate.rs`.

## The store

Every recording command opens one SQLite store: `TSR_DB` if set, else `.tsr/tsr.db` under the
current directory, created on first use (`.tsr/` and `*.db` are git-ignored). The database refuses
`UPDATE` and `DELETE` on events. `tsr review`, `tsr reflect`, `tsr note` and `tsr task reconcile`
take `--store <path>` (default: the same rule) and exit 2 if the file does not exist.

```bash
( cd "$(mktemp -d)" &&
  tsr task open --name demo --intent "add a parser test" &&  # prints the task id
  tsr task status demo &&                                    # event count and last event
  tsr task export demo )                                     # one {"seq": n, "event": {...}} per line
```

`task status` and `task export` accept a 64-hex task id, a Claude Code session id, or the name given
to `task open --name`.

## Recording a Claude Code session (M0)

**The protocol.** Before a gated action the agent records a forecast; it runs commands as
`tsr exec -- <argv>`; the hook records the rest. Nothing on main enforces this (limitation 9).

- `tsr forecast --kind run_approved_test|propose_patch|request_approval --args <args> --p <p>
  --source generator|controller|operator` mints the candidate the hook will mint for the same call
  (`--args`: the Bash command, the part after `tsr exec --`, or the Edit/Write file path) and records
  `Proposed` (unless pending) and `Forecast`. `--event <candidate-id>` names a proposed candidate
  instead (unchecked, limitation 8).
- Session: `tsr forecast` takes `--task`, `--session` or `TSR_SESSION`, else the `session` file the
  hook writes beside the store; `tsr exec` takes `TSR_SESSION`, else that file. With none, both exit
  2 and record nothing. The file holds only the last session the hook saw.

**What the hook records.** `tsr hook --stage <stage>` reads one Claude Code hook payload on stdin,
prints nothing and always exits 0; it never denies or blocks. Tool names match exactly.

| Tool call (PreToolUse) | Recorded |
| --- | --- |
| `Read`, `Grep`, `Glob`, `rts` | `inspect`: `Proposed`, `Committed` |
| `Edit`, `Write` | `propose_patch`: `Proposed` (unless a forecast already proposed it), `Committed` |
| `Bash` whose text starts with `cargo test` | `run_approved_test`: `Proposed` (same rule), `Committed`, `Dispatched { in_sandbox: false }` |
| `Bash` whose first two words are `tsr exec` | `run_approved_test`: `Proposed` (same rule), `Committed`; `tsr exec` records the dispatch |
| `Bash` whose first two words are `tsr forecast` | nothing, before or after |
| anything else (other `Bash`, `MultiEdit`, `mcp__rts__*`, …) | `Generate`, `Committed` |

`SessionStart` records `TaskOpened` and `Stop` records `Stop`. `PostToolUse` records something only
for other `Bash` calls: `Executed` if `tool_response` has an exit code, else `Unresolved`. Claude
Code's Bash response has none (`docs/m0-evidence.md`), so in a live session only `tsr exec` records
an exit code.

**`tsr exec`** runs the argv with `podman` (else `docker`) in a separate container: no network,
read-only root, the current directory bind-mounted read-write at the same path, all capabilities
dropped, CPU, memory and process limits (`crates/tsr-cli/src/exec.rs`). It records
`Dispatched { in_sandbox: true }` and `Executed`, reusing the candidate the hook just committed for
the same argv, prints the container's stdout then its stderr, both on its own stdout, and exits with the
runtime's exit code. If no runtime answers `--version` it exits 3 and records nothing.

The image is `TSR_EXEC_IMAGE` (a value that is not a plausible image reference, such as
`--privileged`, is ignored), else `docker.io/library/debian:stable-slim`, which has no Rust
toolchain: `tsr exec -- cargo test` exits 127 there, as both runs in the committed
`fixtures/m0/session.events.jsonl` do.
For Rust, the image needs the toolchain, and since the container has no network the crate's
dependencies must already be available inside it (in the image, or vendored or prebuilt under the
mounted directory): `rust:1.90-bookworm` alone has only the toolchain, so a crate with dependencies
fails there even with `--offline`. The real session prebuilt the crate's test
dependencies once with its `tsr-rust:1.90` image at the same bind path, so the in-session
`tsr exec -- cargo test --offline` needed no network; the setup is in `docs/m0-evidence.md`, "The
real Claude Code session".

The flow without Claude Code, in a scratch directory:

```bash
( cd "$(mktemp -d)" &&
  echo '{"session_id":"demo","hook_event_name":"SessionStart"}' | tsr hook --stage SessionStart &&
  tsr forecast --kind run_approved_test --args "echo hello" --p 0.9 --source generator &&
  echo '{"session_id":"demo","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"tsr exec -- echo hello"}}' |
    tsr hook --stage PreToolUse;
  tsr exec -- echo hello;
  tsr task export demo )
```

With a runtime the export lists `task_opened`, `proposed`, `forecast`, `committed`, `dispatched`
(`in_sandbox: true`) and `executed`; with none, `tsr exec` exits 3 and the export ends at `committed`.

**Registering the hook** in `~/.claude/settings.json` or a project `.claude/settings.json`:

```json
{
  "hooks": {
    "SessionStart": [{ "hooks": [{ "type": "command", "command": "/path/to/tsr hook --stage SessionStart" }] }],
    "PreToolUse": [{ "matcher": "*", "hooks": [{ "type": "command", "command": "/path/to/tsr hook --stage PreToolUse" }] }],
    "PostToolUse": [{ "matcher": "*", "hooks": [{ "type": "command", "command": "/path/to/tsr hook --stage PostToolUse" }] }],
    "Stop": [{ "hooks": [{ "type": "command", "command": "/path/to/tsr hook --stage Stop" }] }]
  }
}
```

The hook, `tsr forecast` and `tsr exec` must resolve the same store path; an absolute `TSR_DB` in
the session's environment (for example the settings file's `env` block) does that from any directory.

**Reconstruct.** `tsr_cli::reconstruct::from_events` rebuilds a session from an exported log and
checks M0's rules: every `Committed` names a `Proposed` candidate, every gated commitment has a
forecast at its state version, every dispatch is settled and `in_sandbox: true`. There is no CLI
command for it; `tests/m0_reconstruct.rs` (frozen) and `tests/m0_real_session.rs` run it on
`fixtures/m0/`, the latter on a real Claude Code session on a disposable copy of rts.

**Self-test.** `tsr hook --selftest` drives the M0 sequence (reads, three forecasts, two `tsr exec --
cargo test --offline -p parser` runs, an edit, a stop) through the hook and writes
`fixtures/m0/session.events.jsonl` and `session.db` under the current directory, so run it in a
scratch directory (from the repository root it overwrites the frozen fixture):

```bash
( cd "$(mktemp -d)" && tsr hook --selftest )
```

On the default image both runs exit 127, as in the committed record; with no runtime it exits 3 and
writes nothing.

## Policy check (M1)

`tsr policy check <profile.toml> <event.json>` evaluates one proposed action against a RATCHET
protection profile (TOML: read scope, allowlisted test commands, budgets, which boundary actions need
an approval) and prints the verdict as one JSON line; exit 0 allow, 1 deny, 2 ask, 3 malformed input.
Formats: [`docs/ratchet-profile.md`](docs/ratchet-profile.md).

```bash
tsr policy check profiles/default.toml crates/tsr-cli/tests/fixtures/policy/no-forecast.json
```

prints `{"decision":"deny","reason":"record a forecast for RunApprovedTest at state version 1 before
this action"}` and exits 1. The gate reads only the profile and the envelope; `tsr hook` does not
call it.

## Review and reflect (M2)

`tsr review` labels a task's returns after close; `tsr reflect` renders the Mirror in the PRD's order
(§B7): Desire, Imagination, Belief, Return and the residual. Both take the 64-hex task id; for a hook
session, `tsr task status <session-id>` prints it.

The thirty episodes in `fixtures/m2/episodes/` are **synthetic** (a seeded generator in
`crates/tsr-cli/tests/m2_episodes.rs`; families `alpha`, `beta`, `gamma` with an over-confident,
unbiased and under-confident generator). Their store is git-ignored and written only under
`TSR_REGEN_FIXTURES=1`; the tests fail if it changes, so work on a copy:

```bash
TSR_REGEN_FIXTURES=1 cargo +1.90.0 test -p tsr-cli --test m2_episodes   # writes fixtures/m2/store/tsr.db
S="$(mktemp -d)/tsr.db"; cp fixtures/m2/store/tsr.db "$S"
A=1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b     # episode alpha-00
tsr reflect "$A" --store "$S"                    # the Mirror; --json for the same as a document
tsr reflect --family alpha --store "$S"          # reliability bins, Brier, log loss per forecaster
tsr review --family alpha --stats --store "$S"   # false-accept rate (green but rejected)
```

The last command prints:

```
family=alpha labels=10
false_accept_rate unweighted=0.2857 weighted=0.2857
greens=7 false_accepts=2 weighted_greens=7.0000 weighted_false_accepts=2.0000
```

Every fixture return is already labelled; the real session's log has one awaiting a label:

```bash
D=$(mktemp -d)
python3 scripts/m2-session-store.py fixtures/m0/claude-session.events.jsonl "$D/claude.db"
T=098763fe2bbdfe24b9a564340189cfe86f0d3a9cff56f29f9739e8e445b723b8     # the real session's task
tsr review "$T" --store "$D/claude.db"              # lists the return awaiting a label
tsr review "$T" accepted --store "$D/claude.db"     # reviewed 9171a5db… label=accepted status=observed …
```

Labels are `accepted`, `rejected --reason <text>` (which also records a Contrast and an open operator
preference) and `disputed`; `--return <id>` names the return and a later label supersedes an earlier
one; `--sample <rate> --seed <n>` samples greens with weight 1/rate. `tsr note <task> --text <text>`
records an operator note; `tsr task reconcile <task>` records `Unresolved` for a dispatch with no
outcome. Every command with its output: [`docs/m2-evidence.md`](docs/m2-evidence.md).

Limits: every forecast prints `uncalibrated` (no calibrator exists); `tsr reflect --family` over a
hook-recorded store prints `no scored forecasts yet` (hook sessions carry no family scope);
reconcile settles a dispatch only from a later `Executed`.

## Known limitations on main

Current behaviour of the M0 and M1 code:

1. **Runtime failures are recorded as runs.** The runtime probe is `--version`; a failure of the
   runtime itself (daemon down, image missing) is recorded `Dispatched { in_sandbox: true }` plus
   `Executed` with the runtime's exit code. With the Docker CLI and no daemon, `tsr exec` exits 1 and
   `tsr hook --selftest` exits 0, each recording runs that never happened.
2. **The store is inside the sandbox's writable mount.** The default `.tsr/tsr.db` is under the
   directory `tsr exec` bind-mounts read-write, so the sandboxed command can write the event log.
3. **Session lookup starts at the store's directory.** With the default relative store, `tsr forecast`
   or `tsr exec` run from a subdirectory finds no session (exit 2) and creates a second store there.
   An absolute `TSR_DB` avoids both.
4. **Test runs are recognised by their first words only.** `Bash` is `run_approved_test` only if its
   text starts with `cargo test` or its first two words are `tsr exec` (a path to `tsr` is allowed);
   `cd x && …`, `VAR=1 …`, `( … )`, `cargo +1.90.0 test` and `cargo nextest run` are `Generate`.
5. **A forecast is lost whenever the shell changes the text.** The hook digests the text after
   `tsr exec --` split on whitespace; `tsr exec` digests the argv the shell hands it. Quotes,
   redirects, pipes and expansions (`sh -c 'cargo test'`, `… 2>&1 | tail -20`) make them differ, and
   the commitment then has no forecast.
6. **An ordinary session does not pass reconstruct.** Each call in the table's last row is recorded
   `Generate` + `Committed` with no `Proposed`, which reconstruct rejects. One `ls` is enough.
7. **`tsr task status` miscounts.** It prints the task's highest sequence number in the store as its
   event count, wrong once the store holds more than one task.
8. **`tsr forecast --event` is unchecked.** It records a forecast for any 64-hex id, proposed or not.
9. **Nothing forces the protocol.** The gate is reached only through `tsr policy check`; the hook
   records a gated action without a forecast and never denies it. Every `Bash` command other than
   `tsr exec -- …`, plain `cargo test` included, runs on the host, and Claude Code applies
   `Edit`/`Write` on the host; the log only records them.

## Layout

```
crates/tsr-domain     the contract: facts, views, traits (PRD B6)
crates/tsr-store      SQLite index over the content-addressed, append-only log
crates/tsr-cli        the `tsr` binary (operator CLI and hook) and reconstruct
crates/tsr-policy     RATCHET profiles, the gate, approvals, budgets (M1)
crates/tsr-ledger     the Culmination ledger as folds over the log (M2)
crates/tsr-forecast   named events, forecast resolution, scoring (M2)
crates/tsr-learn      datasets, the calibration head and calibrator, bundles, the report (M4)
profiles/             the default RATCHET profile
fixtures/m0           recorded M0 sessions (frozen)
fixtures/m2           the thirty synthetic episodes
fixtures/m4           the synthetic M4 corpus
scripts/              m2-evidence.sh and the store builders it uses; m4-evidence.sh
specs/lanes/          lane briefs and their independent reviews
docs/                 the PRD, the profile format, evidence pages
```

## License

`Cargo.toml` declares `license = "MIT OR Apache-2.0"`. The repository has no LICENSE file.
