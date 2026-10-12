**Lane 05rrr (M3, third repair): not merge-ready.** The session now works in its copy and nowhere else, but the central requirement still fails. With tests inline in `src/lib.rs` (the default Rust layout, and the previous review's repository), the oracle's restore step erases a correct mined fix and reverts every mutant itself, so a do-nothing session is accepted 5/5. A session that writes a `.cargo/config.toml` runner is accepted with the bug present on any layout. And no batch `tsr exec` carries TSR_SESSION, so a real run rejects every task.

Checked at 23006f9 with a stand-in `tsr exec` that logs its argv, cwd and TSR_SESSION, then runs the argv in its cwd. I built two real 4-commit repositories (c0 green, c1 refactor, c2 bug, c3 fix): **repoA** has its tests inline in `src/lib.rs`, **repoB** has them in `tests/tests.rs`.

## Items

**(1) D3, the session works in the copy: FIXED.**
- `drive` takes the copy (driver.rs:138), run.rs:421 passes it, and edits are applied there (driver.rs:713, symlink guard at :216).
- repoB, `task run --mode record`, no `--scripted-runner`, a session that fixes the bug: `1 accepted`. The fix is in the copy, the repository's `git status` is empty, and every exec cwd is `copies/<id>/record`.
- On repoA the edit also lands in the copy, then P1 overwrites it.

**(2) D3, the oracle judges the copy with the snapshot's tests: PARTLY.**
- The table through `tsr task run` (record mode, no scripted runner):

  | Variant | repoA (`[src/lib.rs]`) | repoB (`[tests/tests.rs]`) |
  |---|---|---|
  | bug present | rejected | rejected |
  | bug fixed | **rejected** (the copy is back to `a - b`) | accepted |
  | test deleted / edited / renamed | rejected | rejected |
  | `.cargo/config.toml` `runner = "true"`, bug present | **accepted** | **accepted** |
- A task with no `oracle_test_files` is refused at load (task.rs:201, exit 2). That part is FIXED.
- "A filter that matches no test fails" does not work (P3). The renamed row fails only because the restore puts the test back.

**(3) D4, D9, mutant tasks: PARTLY.**
- Run: real `tsr corpus mutants` with a stand-in cargo-mutants that runs `cargo test` per mutant; outcomes.json carries no `diff`. Result: 6 listed, 5 tasks.
- Every task has `base_snapshot` and a `mutation_diff` in the corpus's `cas/`. All 5 reversals pass `git apply --check` and restore the base exactly.
- On repoB, `task run` provisions all 5 with the mutation present, and a do-nothing session is rejected 5/5. On repoA, a do-nothing session is **accepted 5/5** (P1).
- A provisioning or mutation-diff failure still aborts the whole batch (run.rs:387-392, 404-418, 664), not just the task. With one task's diff deleted from `cas/`: exit 2, and no results file is written.

**(4) N2, N3, N5, the sandbox boundary: PARTLY.**
- Holds:
  - Provisioning is a host step that only reads the repository (tsr-corpus lib.rs:309-317).
  - Every batch exec's cwd is one task's copy.
  - The mutants list and kill run happen in a `/tmp` copy.
  - The repositories are clean after mine and after mutants.
  - A relative `--out` and relatively mined repos work from the same cwd.
- **TSR_SESSION is never set** (lib.rs:318-324); every log line shows `<unset>`. The real `tsr exec -- true` in a copy exits 2 (`no session`).
- The oracle nests `tsr exec` inside `tsr exec` (corpus_cmd.rs:134). Docker-dependent; not run.
- A relative `repo` is resolved against the batch's cwd, not the manifest's directory (run.rs:364). From another directory: `materialize … exited 1`, and the batch exits 2.

**(5) R4, N1, R5: FIXED.**
- The two-lineage case prints `resamples: 1003 used of 2000 requested`, then `interval n/a — 997 of 2000 resamples dropped (50% > 5%)` and `INCONCLUSIVE`.
- mine_d4 is green; the r5 test exists; the worktree list is unchanged.

**(6) N4, Tier 0 reads the task: NOT FIXED.**
- batch_cmd.rs:203-261 ignores `oracle_test_files` and guesses `src/<head>.rs` and `tests/<head>.rs` from the test path. For `tests::flipped_add` the guess includes `src/tests.rs`, which exists in neither repository.
- Rules mode, on both corpora:

  | Session | Result |
  |---|---|
  | test first | denied (correct) |
  | reads the task's files first | **denied** |
  | reads the non-existent guessed files first | allowed |
  | runs the test first as `cargo-test-workspace` | allowed (only one id is checked, batch_cmd.rs:217-237) |

**Spot checks: all hold.**
- R1: `"/etc"` exits 2; the exec log is empty.
- R2: an absolute or `../../` `log_path` exits 1 with `unsafe log_path`.
- R3: a two-test task is accepted; the argv is `cargo test -- tests::other tests::flipped_add`.
- D6: t1 overspends (20+20 against 30) and is closed without an oracle run; the other 4 tasks run.
- D8: `paired tasks: 5`.
- D10: no profile exits 2 naming where it looked; the binary has no `tsr-cli/../..`; numbers print as `name=value`.

## New defects

**P1 (high): the whole-file restore hits the code under repair.**
- Where: run.rs:484-486 and 544-594 restore every `oracle_test_files` entry from the snapshot. mine.rs:367-389 lists the file defining the flipped test (`git grep "fn <name>"` at the parent); mutants.rs:952-978 lists every file containing `fn <name>`.
- With inline `#[cfg(test)] mod tests`, that file is the source file itself:
  - Mined tasks: the fix is reverted to the parent, so repoA "bug fixed" is rejected.
  - Mutant tasks: the mutation is reverted, so repoA's do-nothing session is accepted 5/5.
- The lane's tests use only a separate `tests/demo.rs`, so they miss this.

**P2 (high): the oracle trusts build configuration the session can write.**
- A step writing `.cargo/config.toml` with `[target.'cfg(all())'] runner = "true"` is accepted with the bug present on both repositories.
- Output with no `test result:` line counts as a pass (run.rs:636-638, corpus_cmd.rs:181). The same holds for `Cargo.toml` and `build.rs`.

**P3 (medium): the no-match guard does nothing.**
- run.rs:614-617 and corpus_cmd.rs:156-162 read the run's stdout from a CAS that nothing ever writes, so the guard returns true.
- `tsr corpus oracle --tests tests::zz_matches_nothing` on a bug-present copy prints `oracle: pass (exit 0)`.

**P4 (medium, B10): the restore writes on the host through a symlinked directory.**
- run.rs:555-563 checks only the final path component; run.rs:585-588 then follows `tests -> <host dir>`.
- A throwaway test with `copy/tests` pointing outside the copy returned `Ok` and wrote `outside-the-copy/tests.rs`.

**P5 (low):**
- run.rs:577-584 silently skips a listed file the snapshot lacks, such as a test added by the fix commit (code reading).
- mutants.rs:886 returns early on an error and leaks the `/tmp` kill-run copy (two were left after the R2 checks).

## Evidence doc

docs/m3-evidence.md is still the 0a7c6ef capture. Item 7 must fix:
1. Line 3 names `067c57b`. It must name the final tip.
2. §3 must say clippy passes on 1.90 only. On 1.97 it fails at gate.rs:411 `filter_next` (main) and driver.rs:537 `collapsible_match` (lane); both lints are 1.97-only.
3. §4 says `247 tests passed, 0 failed`, TEST_EXIT=0, and m0_forecast_default 6 passed. The binary gives:
   - 293 passed, 2 failed (both Docker-only m0 tests), 4 ignored, exit 101.
   - Binaries missing from the doc: d3_oracle_table, d3_session_copy, d3_symlink_escape, d6_tier0_rules, n2_n3_n5_sandbox, r4_n1_statistics.
   - batch.rs now passes 25, mine.rs 5, mutants.rs 26. §7 still quotes 18, 4 and 20.
4. §5's command writes `split.json` into `fixtures/m3/batch`, which dirties the tree. Its output matches.
5. §6 has `resamples: 2000`; the binary prints `resamples: 2000 used of 2000 requested`. Every number line differs: the doc has `0.600000` and `[-6.000000, -6.000000]`, the binary prints `completion_rate=0.6` and `[cost_per_accepted=-6, cost_per_accepted=-6]`.
6. §7 says mine, mutants and task run are "not runnable" and everything goes through ExecRunner; both are now false. The doc lacks:
   - a mine of a real repository;
   - record and rules `task run` provisioning real copies;
   - the oracle table, including the inline-test row;
   - mutants provisioning.

## Definition of done

| Clause | Status | Notes |
|---|---|---|
| build; fmt (1.97 and 1.90) | MET | |
| clippy -D warnings | MET on 1.90 | 1.97 fails at gate.rs:411 and driver.rs:537 |
| `cargo test --workspace --no-fail-fast` | MET | 293 passed, 2 failed (Docker-only m0_forecast_default and m0_harness; Docker CLI present, no daemon), 4 ignored |
| fixtures/m0, hook.rs, exec.rs, store_path.rs, task.rs, m0 tests unchanged; no lane 04/06 file touched | MET | `git diff d2b5f96..HEAD` on them is empty; lib.rs adds only `pub mod`; main.rs touches only the Corpus, Eval and Task Run arms; lane.diff matches the tree |
| An id-named test that failed before each fix | CAN'T TELL | They exist, but the d3 and n4 tests pass while the behaviour is wrong |
| Edits land in the copy | MET | |
| Oracle restores the tests and the table holds | NOT MET | P1–P3 |
| Mutant tasks provision; reversals apply | MET | P1 aside |
| Every exec in one copy with TSR_SESSION | NOT MET | |
| Repository never writable; clean after mine and mutants | MET | |
| Relative paths | MET | Same cwd only |
| Bootstrap resamples / INCONCLUSIVE | MET | |
| Tier 0 uses the task's files | NOT MET | |
| Evidence | NOT MET | |
| Original brief's DoD | NOT MET | mine, mutants, split, eval and the five-task record run hold. The CLI rules run over fixtures records 0 denials (m3_batch_invariants passes). The evidence is stale |

## What I ran

The build, fmt, clippy, test, mine, mutants, task-run and eval commands are quoted with their results above. In addition:
- The real `tsr exec` without TSR_SESSION: exit 2. `tsr corpus oracle` with a filter matching nothing: pass.
- A throwaway `crates/tsr-batch/tests/zz_review_restore.rs` showed P4; I deleted it.
- `tsr eval baseline` on the evidence fixtures, diffed against §6.
- Cleanup: removed `tree/target/` and `crates/tsr-cli/.tsr/` (both created by the test run). `git status --ignored` is clean at 23006f9.
