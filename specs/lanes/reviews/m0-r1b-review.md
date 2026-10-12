**M0 r1b: done** — every requested `TSR_DB` spelling gets the read-only store mount (or the exit-2 refusal) exactly when its resolved directory is inside the workspace; one low-severity edge remains (symlink followed by `..`).

## TSR_DB spellings (fake docker, cwd = scratch workspace W, `tsr exec -- echo hi`)

| TSR_DB | Expected | Seen |
|---|---|---|
| unset | RO `<W>/.tsr` | same |
| `data/tsr.db` | RO `<W>/data` | same |
| `.tsr/tsr.db`, `./.tsr/tsr.db` | RO `<W>/.tsr` | same |
| `tsr.db`, `./tsr.db`, `a/../tsr.db` | exit 2 | exit 2, no docker run, nothing recorded |
| abs inside W | RO that dir | same |
| abs outside W | none | none |
| outside symlink → `<W>/real` | RO `<W>/real` | same (src = dst = resolved) |
| outside symlink → W itself | exit 2 | exit 2 |
| `<W>/link` → outside | none | none |
| `a/../b/../.tsr/tsr.db` | RO `<W>/.tsr` | same |
| `deep/not/yet/tsr.db` | RO, dir created first | RO `<W>/deep/not/yet`; src existed when `run` was called |
| `TSR_DB=` empty | as unset | same |
| `../out/tsr.db` | none | none |
| outside file symlink → `<W>/real/tsr.db` | RO `<W>/real` | same |
| cwd reached via symlink | resolved W | both mounts use resolved path |

The RO mount always follows the workspace mount (argv lines 22–23 vs 8–9).

## (1) r1 behaviours: hold
- `info` failing: exit 3, no `run`, 0 events.
- `run` exit 125: `proposed, committed, dispatched, unresolved`, exit 3 (message still repeats "container runtime failed to start" three times, cosmetic).
- `run` exit 1: `executed`, exit 1.
- `hook --selftest` without the flag writes only to a fresh temp dir; a scratch copy of `fixtures/m0` is byte-identical afterwards. Daemon unreachable: exit 3, nothing written.

## (2) Diff and gates
- Diff: exec.rs, store_path.rs (+ unit tests), m0_honest_sandbox.rs, specs/ x2, and **`tests/m0_forecast_default.rs`** (2 lines), which is off the Owns list. The commit message discloses it and it is forced: that test's store was `<ws>/tsr.db`, now refused, so it moved to `<ws>/.tsr/tsr.db`. Low, process only.
- fmt clean; `cargo +1.90.0 clippy --workspace --all-targets -- -D warnings` clean.
- `cargo test --workspace --no-fail-fast`: **280 passed, 0 failed, 0 ignored** (270 before). The `m0_harness` real-runner half self-skips and passes; no Docker-only failures.

## New defects
1. **Low: symlink followed by `..`.** `..` is collapsed lexically before symlinks resolve; the OS does the opposite. With `$O/link → <W>/deep/sub`, `TSR_DB=$O/link/../x/tsr.db` opens `<W>/deep/x/tsr.db` but resolves to `<O>/x/tsr.db`, so no RO mount. The reverse over-mounts (harmless). It needs an operator-written path, and the brief prescribed lexical-first. Fix: resolve component by component.
2. **Low:** the exit-2 refusal comes after `open_store()`, leaving an empty `<W>/tsr.db` (nothing recorded).

## What I ran
A fake `docker` recording argv and whether each bind src existed at `run`; a driver over about 25 fresh workspaces with the built `tsr`; `tsr task export s1` for events; a copied `fixtures/m0`; fmt, clippy 1.90, full suite. `git status` clean at 98be827.
